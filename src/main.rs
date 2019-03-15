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
#[macro_use]
extern crate rocket_failure;
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
        .mount("/", routes![routes::index, routes::recent, routes::about, routes::download_redirect])
        .mount("/themes/users/", routes![routes::users::user_themes])
        .mount("/themes/report/", routes![routes::report::report_view, routes::report::report_view_default])
        .mount("/themes/view/", routes![routes::theme])
        .mount("/themes/repo", routes![routes::download_theme])
        .mount(
            "/",
            rocket_contrib::serve::StaticFiles::from(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/static"
            )),
        )
        .launch();
}
