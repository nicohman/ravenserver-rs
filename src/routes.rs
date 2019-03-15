use crate::routes::rendering::*;
use crate::*;
use mongodb::to_bson;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::response::NamedFile;
use rocket_contrib::json::{Json, JsonValue};
use rocket_contrib::templates::Template;
use rocket_failure::errors::*;
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
                x.screen = "https://images.weserv.nl/?url=".to_string()
                    + &x.screen.replace("https://", "").replace("http://", "");
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
#[get("/<name>")]
pub fn theme(conn: DbConnection, name: String) -> Template {
    let db = DataBase::from_db(conn.0.clone()).unwrap();
    let theme: Theme = db.find_one_key_value("name", name).unwrap().unwrap();
    let mut context = CONFIG.clone();
    context.insert("theme".to_string(), to_bson(&theme).unwrap());
    Template::render("theme", context)
}
#[get("/downloads")]
pub fn download_redirect() -> rocket::response::Redirect {
    rocket::response::Redirect::to("https://nicohman.demenses.net/downloads")
}
#[get("/<name>")]
pub fn download_theme(conn: DbConnection, name: String) -> Custom<NamedFile> {
    let db = DataBase::from_db(conn.0.clone()).unwrap();
    let theme: Theme = db.find_one_key_value("name", name).unwrap().unwrap();
    let file =
        NamedFile::open(env!("CARGO_MANIFEST_DIR").to_string() + "/public/tcdn" + &theme.path)
            .unwrap();
    if theme.reports.len() > 0 && !theme.approved {
        Custom(Status::AlreadyReported, file)
    } else {
        Custom(Status::Ok, file)
    }
}
/// Routes to do with theme metadata
pub mod metadata {
    use super::*;
    #[get("/<name>")]
    pub fn get_metadata(conn: DbConnection, name: String) -> ApiResult<JsonValue> {
        let db = DataBase::from_db(conn.0.clone()).unwrap();
        let theme: Theme;
        if let Some(theme) = db
            .find_one::<Theme>(doc! {"name": &name}, None)
            .not_found()?
        {
            Ok(json!({
                "screen": theme.screen,
                "description":theme.description
            }))
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
/// Routes relating to user-specific pages
pub mod users {
    use super::*;
    #[get("/view/<id>")]
    pub fn user_themes(conn: DbConnection, id: String) -> Template {
        let db = DataBase::from_db(conn.0.clone()).unwrap();
        let user: User = db.find_one_key_value("id", id.as_str()).unwrap().unwrap();
        let mut find = FindOptions::new();
        find.sort = Some(doc! {
            "installs":-1
        });
        render_themes_view(
            conn,
            Some(doc! {
                "author":id.as_str()
            }),
            Some(find),
            format!("All themes by {}", user.name.as_str()),
            None as Option<String>,
        )
    }

}
