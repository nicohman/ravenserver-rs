use crate::routes::rendering::*;
use crate::*;
use auth::*;
use bcrypt::*;
use chrono::prelude::*;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use mongodb::to_bson;
use rocket::data::*;
use rocket::http::Status;
use rocket::request::FromFormValue;
use rocket::response::status::Custom;
use rocket::response::NamedFile;
use rocket::Outcome::*;
use rocket::Request;
use rocket_contrib::json::{Json, JsonValue};
use rocket_contrib::templates::Template;
use rocket_failure::errors::*;
use std::fs;
pub const MAX_FILE_SIZE: i64 = 50000000;
lazy_static! {
    pub static ref DOWNLOADS: HashMap<String, String> = {
        let mut map = HashMap::new();
        map.insert("raven".to_string(), "static/raven-nightly".to_string());
        map.insert("ravend".to_string(), "static/ravend-nightly".to_string());
        map.insert("eidolon".to_string(), "static/eidolon-nightly".to_string());
        map
    };
}
/// General helpers for rendering views of multiple themes
pub mod rendering {
    use crate::*;
    use mongodb::{to_bson, Bson};
    use rocket_contrib::templates::Template;
    use std::collections::HashMap;
    lazy_static! {
        pub static ref CONFIG: HashMap<String, Bson> = {
            let mut map = HashMap::new();
            map.insert("include_downloads".to_string(), bson!(true));
            map
        };
    }
    pub fn render_themes(themes: Vec<Theme>, mut context: HashMap<String, Bson>) -> Template {
        let themes = themes
            .into_iter()
            .map(|mut x| {
                if x.screen.len() > 0 {
                    x.screen = "https://images.weserv.nl/?url=".to_string()
                        + &x.screen.replace("https://", "").replace("http://", "");
                }
                if x.description.len() > 35 {
                    x.description.truncate(35);
                    x.description += "...";
                }
                x
            })
            .collect::<Vec<Theme>>();
        context.insert("themes".to_string(), to_bson(&themes).unwrap());
        Template::render("themes", &context)
    }
    pub fn render_themes_view(
        conn: DbConnection,
        filter: Option<mongodb::Document>,
        options: Option<FindOptions>,
        ptitle: impl Into<String>,
        constraints: Option<impl Into<String>>,
    ) -> Template {
        let (ptitle, constraints) = (
            ptitle.into(),
            constraints.map_or(String::default(), |x| x.into()),
        );
        let db = DataBase::from_db(conn.0.clone()).unwrap();
        let themes = db
            .find::<Theme>(filter.unwrap_or(doc!()), options)
            .unwrap()
            .into_iter()
            .filter_map(|x| x.ok())
            .collect::<Vec<Theme>>();
        let mut context = CONFIG.clone();
        context.insert("ptitle".to_string(), to_bson(&ptitle).unwrap());
        context.insert("constraints".to_string(), to_bson(&constraints).unwrap());
        render_themes(themes, context)
    }

}
#[get("/")]
pub fn index(conn: DbConnection) -> Template {
    let mut find = FindOptions::new();
    find.sort = Some(doc! {
        "installs":-1
    });
    render_themes_view(
        conn,
        None,
        Some(find),
        "All themes",
        Some("Sorted by most popular"),
    )
}
#[get("/recent")]
pub fn recent(conn: DbConnection) -> Template {
    let mut find = FindOptions::new();
    find.sort = Some(doc! {
        "updated":  -1
    });
    render_themes_view(
        conn,
        None,
        Some(find),
        "All themes",
        Some("Sorted by most recent"),
    )
}
#[get("/about")]
pub fn about() -> Template {
    Template::render("about", CONFIG.clone())
}
#[get("/downloads")]
pub fn download_redirect() -> rocket::response::Redirect {
    rocket::response::Redirect::to("https://nicohman.demenses.net/downloads")
}
#[get("/checksums")]
pub fn checksums() -> ApiResult<Template> {
    let mut md5 = crypto::md5::Md5::new();
    let mut sums = HashMap::new();
    for (key, value) in DOWNLOADS.iter() {
        let mut loaded = String::new();
        File::open(&value)?.read_to_string(&mut loaded)?;
        md5.input_str(&loaded);
        sums.insert(key.to_string(), md5.result_str());
    }
    Ok(Template::render("checksums", sums))
}
/// Routes to do with themes
pub mod themes {
    use super::*;
    #[get("/repo/<name>")]
    pub fn download_theme(conn: DbConnection, name: String) -> ApiResult<Custom<NamedFile>> {
        let db = DataBase::from_db(conn.0.clone()).unwrap();
        if let Some(theme) = db.find_one::<Theme>(doc! { "name": &name }, None)? {
            let file = NamedFile::open(
                env!("CARGO_MANIFEST_DIR").to_string() + "/public/tcdn/" + &theme.path,
            )
            .unwrap();
            if theme.reports.len() > 0 && !theme.approved {
                Ok(Custom(Status::AlreadyReported, file))
            } else {
                Ok(Custom(Status::Ok, file))
            }
        } else {
            not_found!(name)
        }
    }

    #[get("/view/<name>")]
    pub fn theme(conn: DbConnection, name: String) -> ApiResult<Template> {
        let db = DataBase::from_db(conn.0.clone()).unwrap();
        if let Some(theme) = db.find_one::<Theme>(doc! {"name": &name}, None)? {
            let mut context = CONFIG.clone();
            context.insert("theme".to_string(), to_bson(&theme).unwrap());
            Ok(Template::render("theme", context))
        } else {
            not_found!(name)
        }
    }

    #[post("/delete/<name>")]
    pub fn delete_theme(conn: DbConnection, name: String, token: UserToken) -> ApiResult<Status> {
        let db = DataBase::from_db(conn.0.clone()).unwrap();
        if let Some(mut theme) = db.find_one::<Theme>(doc! {"name":&name}, None)? {
            if token.id == theme.author {
                let path = format!(
                    "{}/public/tcdn/{}",
                    env!("CARGO_MANIFEST_DIR"),
                    &theme.path
                );
                fs::remove_file(path)?;
                db.delete_one(theme, None)?;
                Ok(Status::Ok)
            } else {
                Err(WebError::new("Not allowed to delete this theme")
                    .with_status(Status::Forbidden))
            }
        } else {
            not_found!(name)
        }
    }
    #[post("/upload?<name>", data = "<theme>")]
    pub fn upload_theme(
        conn: DbConnection,
        name: String,
        mut theme: UploadedTheme,
        token: UserToken,
    ) -> ApiResult<Custom<String>> {
        let db = DataBase::from_db(conn.0.clone()).unwrap();
        let mut found_theme = db.find_one::<Theme>(doc! {"name":&name}, None)?;
        let mut status = Status::Created;
        let filename = format!("{}.tar", &name);
        let now: DateTime<Local> = Local::now();
        let date = now.to_rfc2822();
        if let Some(mut found_theme) = found_theme {
            if found_theme.author != token.id {
                return Err(WebError::new("Not owned by you").with_status(Status::Forbidden));
            }
            status = Status::Ok;
            found_theme.updated = date;
            db.save::<Theme>(found_theme, None)?;
        } else {
            let tosa = doc! {
                "name":&name,
                "author":&token.id,
                "pauthor":&token.name,
                "updated":&date,
                "date":&date,
                "description":"This theme doesn't have a description yet.",
                "screen":"",
                "path":&filename
            };
            db.insert_one::<Theme>(tosa, None)?;
        }
        theme.data.stream_to_file(format!(
            "{}/public/tcdn/{}",
            env!("CARGO_MANIFEST_DIR"),
            &filename
        ))?;
        return Ok(Custom(status, filename));
    }
    pub struct UploadedTheme {
        pub data: Data,
    }
    impl FromDataSimple for UploadedTheme {
        type Error = String;
        fn from_data(req: &Request, data: Data) -> Outcome<Self, String> {
            let headers = req.headers();
            if let Some(length_one) = headers.get_one("content-length") {
                if let Ok(length) = i64::from_str_radix(length_one, 10) {
                    if length < MAX_FILE_SIZE {
                        return Success(UploadedTheme { data: data });
                    } else {
                        return Failure((Status::PayloadTooLarge, "Theme too big".to_string()));
                    }
                } else {
                    return Failure((
                        Status::UnprocessableEntity,
                        "No valid content-length header".to_string(),
                    ));
                }
            } else {
                return Failure((
                    Status::UnprocessableEntity,
                    "No content-length header".to_string(),
                ));
            }
        }
    }

}
/// Routes to do with theme metadata
pub mod metadata {
    #[derive(Serialize, Deserialize, Debug, FromFormValue)]
    pub enum MetaDataType {
        #[serde(rename = "screen")]
        Screen,
        #[serde(rename = "description")]
        Description,
    }
    use super::*;
    #[get("/<name>")]
    pub fn get_metadata(conn: DbConnection, name: String) -> ApiResult<JsonValue> {
        let db = DataBase::from_db(conn.0.clone()).unwrap();
        if let Some(theme) = db.find_one::<Theme>(doc! {"name": &name}, None)? {
            Ok(json!({
                "screen": theme.screen,
                "description":theme.description
            }))
        } else {
            not_found!(name)
        }
    }
    #[post("/<name>?<typem>&<value>")]
    pub fn post_metadata(
        conn: DbConnection,
        name: String,
        token: UserToken,
        typem: MetaDataType,
        value: String,
    ) -> ApiResult<()> {
        let db = DataBase::from_db(conn.0.clone()).unwrap();
        if let Some(mut theme) = db.find_one::<Theme>(doc! {"name":&name}, None)? {
            if token.id == theme.author {
                match typem {
                    MetaDataType::Screen => theme.screen = value,
                    MetaDataType::Description => theme.description = value,
                };
                db.save::<Theme>(theme, None).expect("Couldn't save theme");
                Ok(())
            } else {
                Err(WebError::new("Not Allowed").with_status(Status::Forbidden))
            }
        } else {
            not_found!(name)
        }
    }
}
/// Routes to do with reporting themes
pub mod report {
    use super::*;
    #[get("/")]
    pub fn report_view_default() -> Template {
        let mut context = CONFIG.clone();
        context.insert("ptitle".to_string(), bson!("Report a Theme"));
        Template::render("report", context)
    }
    #[get("/<name>")]
    pub fn report_view(name: String) -> Template {
        let mut context = CONFIG.clone();
        context.insert("ptitle".to_string(), bson!("Report a Theme"));
        context.insert("name".to_string(), bson!(name));
        Template::render("report", context)
    }

}
/// Routes relating to user-specific endpoints
pub mod users {
    use super::*;
    #[get("/view/<id>")]
    pub fn user_themes(conn: DbConnection, id: String) -> ApiResult<Template> {
        let db = DataBase::from_db(conn.0.clone()).unwrap();
        if let Some(user) = db.find_one::<User>(doc! {"id":id.as_str()}, None)? {
            let mut find = FindOptions::new();
            find.sort = Some(doc! {
                "installs":-1
            });
            Ok(render_themes_view(
                conn,
                Some(doc! {
                    "author":id.as_str()
                }),
                Some(find),
                format!("All themes by {}", user.name.as_str()),
                None as Option<String>,
            ))
        } else {
            not_found!(id)
        }
    }
    #[get("/login?<name>&<pass>")]
    pub fn login(conn: DbConnection, name: String, pass: String) -> ApiResult<JsonValue> {
        let db = DataBase::from_db(conn.0.clone()).unwrap();
        if let Some(user) = db.find_one::<User>(doc! {"name": &name}, None)? {
            if verify(&pass, user.password.as_str())? {
                let token = encode_user(UserToken {
                    name: name.clone(),
                    id: user.id,
                })?;
                Ok(json!({
                    "name":name,
                    "token":token
                }))
            } else {
                Err(WebError::new("Wrong login details").with_status(Status::Forbidden))
            }
        } else {
            not_found!(name);
        }
    }
    #[post("/create?<name>&<pass>")]
    pub fn create(conn: DbConnection, name: String, pass: String) -> ApiResult<Status> {
        let db = DataBase::from_db(conn.0.clone()).unwrap();
        if name.len() < 20 && pass.len() < 100 {
            if let Some(user) = db.find_one::<User>(doc! {"name":&name}, None)? {
                Err(WebError::new("User already exists").with_status(Status::Forbidden))
            } else {
                let mut hasher = Sha1::new();
                let hashed = hash(pass, 10)?;
                let now: DateTime<Local> = Local::now();
                let date = now.to_rfc2822();
                hasher.input_str(format!("{}{}", name, &date).as_str());
                let id = hasher.result_str();
                let nu = doc!("name": &name, "pass": &hashed, "id": &id, "date": &date);
                db.insert_one::<User>(nu, None)?;
                Ok(Status::Ok)
            }
        } else {
            Err(WebError::new("Name or password too long").with_status(Status::PayloadTooLarge))
        }
    }
    #[post("/delete/<name>?<pass>")]
    pub fn delete(
        conn: DbConnection,
        name: String,
        pass: String,
        token: UserToken,
    ) -> ApiResult<Status> {
        let db = DataBase::from_db(conn.0.clone()).unwrap();
        if name == token.name {
            if let Some(user) = db.find_one::<User>(doc! {"name":&name, "id":&token.id}, None)? {
                if verify(pass, &user.password)? {
                    db.delete_one::<User>(user, None)?;
                    db.delete::<User>(doc! {"author":&token.id}, None)?;
                    Ok(Status::Ok)
                } else {
                    Err(WebError::new("Password doesn't match").with_status(Status::Forbidden))
                }
            } else {
                not_found!(name)
            }
        } else {
            Err(WebError::new("Name and token don't match").with_status(Status::Forbidden))
        }
    }
}
