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
#[macro_use]
extern crate serde_derive;
use databases::mongodb::db::ThreadedDatabase;
use mongodb::{to_bson, Bson};
use ravenserver::mongo::*;
use ravenserver::themes::*;
use rocket_contrib::databases;
use rocket_contrib::templates::Template;
use std::collections::HashMap;
#[database("mongodb")]
struct DbConnection(databases::mongodb::db::Database);
use mongodb::pool::*;
lazy_static! {
    static ref CONFIG: HashMap<String, Bson> = {
        let mut map = HashMap::new();
        map.insert("include_downloads".to_string(), bson!(true));
        map
    };
}
#[get("/")]
fn index(conn: DbConnection) -> Template {
    let db = DataBase::from_db(conn.0.clone()).unwrap();
    let themes = db
        .find::<Theme>(doc!(), None)
        .unwrap()
        .into_iter()
        .filter_map(|x| x.ok())
        .map(|mut x| {
            x.screen = "https://images.weserv.nl/?url=".to_string() + &x.screen.replace("https://", "").replace("http://", "");
            if x.description.len() > 35 {
                x.description.truncate(35);
                x.description += "...";
            }
            x
        })
        .collect::<Vec<Theme>>();
    let mut context = CONFIG.clone();
    context.insert("themes".to_string(), to_bson(&themes).unwrap());
    println!("{:?}", context);
    Template::render("themes", &context)
}
fn main() {
    rocket::ignite()
        .attach(DbConnection::fairing())
        .attach(Template::fairing())
        .mount("/", routes![index])
        .mount(
            "/",
            rocket_contrib::serve::StaticFiles::from(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/static"
            )),
        )
        .launch();
}
