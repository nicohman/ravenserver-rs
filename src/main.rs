#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate mongodb;
extern crate ravenserver;
#[macro_use]
extern crate rocket_contrib;
extern crate serde;
#[macro_use]
extern crate lazy_static;
extern crate serde_derive;
use mongodb::coll::options::*;
use mongodb::{to_bson, Bson};
use ravenserver::mongo::*;
use ravenserver::themes::*;
use rocket_contrib::databases;
use rocket_contrib::templates::Template;
use std::collections::HashMap;
#[database("mongodb")]
struct DbConnection(databases::mongodb::db::Database);
lazy_static! {
    static ref CONFIG: HashMap<String, Bson> = {
        let mut map = HashMap::new();
        map.insert("include_downloads".to_string(), bson!(true));
        map
    };
}
fn render_themes(themes: Vec<Theme>, mut context: HashMap<String, Bson>) -> Template {
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
fn render_themes_view(
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
#[get("/")]
fn index(conn: DbConnection) -> Template {
    let mut find = FindOptions::new();
    find.sort = Some(doc! {
        "installs":-1
    });
    render_themes_view(conn, None, Some(find), "All themes", None as Option<String>)
}
#[get("/recent")]
fn recent(conn: DbConnection) -> Template {
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
#[get("/themes/users/view/<id>")]
fn user_themes(conn: DbConnection, id: String) -> Template {
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
        format!("All themes by {}", id),
        None as Option<String>,
    )
}
fn main() {
    rocket::ignite()
        .attach(DbConnection::fairing())
        .attach(Template::fairing())
        .mount("/", routes![index])
        .mount("/", routes![recent])
        .mount(
            "/",
            rocket_contrib::serve::StaticFiles::from(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/static"
            )),
        )
        .launch();
}
