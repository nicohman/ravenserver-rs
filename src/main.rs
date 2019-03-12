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
use ravenserver::mongo::*;
use ravenserver::themes::*;
use rocket_contrib::databases;
use rocket_contrib::templates::Template;
mod routes;
#[database("mongodb")]
pub struct DbConnection(databases::mongodb::db::Database);
fn main() {
    rocket::ignite()
        .attach(DbConnection::fairing())
        .attach(Template::fairing())
        .mount("/", routes![routes::index])
        .mount("/", routes![routes::recent])
        .mount("/", routes![routes::user_themes])
        .mount("/", routes![routes::about])
        .mount(
            "/",
            rocket_contrib::serve::StaticFiles::from(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/static"
            )),
        )
        .launch();
}
