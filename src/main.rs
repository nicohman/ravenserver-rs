#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate mongodb;
extern crate ravenserver;
#[macro_use]
extern crate rocket_contrib;
extern crate crypto;
extern crate serde;
#[macro_use]
extern crate lazy_static;
extern crate bcrypt;
extern crate chrono;
extern crate jsonwebtoken as jwt;
extern crate reqwest;
extern crate serde_json;
#[cfg(test)]
mod tests;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate rocket_failure;
use mongodb::coll::options::*;
use ravenserver::mongo::*;
use ravenserver::themes::*;
use rocket::fairing::AdHoc;
use rocket_contrib::databases;
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
mod auth;
mod routes;
#[database("mongodb")]
pub struct DbConnection(databases::mongodb::db::Database);
pub fn rocket() -> rocket::Rocket {
    let mut st = String::new();
    File::open(format!("{}/downloads.json", env!("CARGO_MANIFEST_DIR")))
        .expect("Couldn't open downloads storage file")
        .read_to_string(&mut st)
        .expect("Couldn't read download counting file");
    let mut downloads: HashMap<String, i64> = serde_json::from_str(&st).unwrap();
    rocket::ignite()
        .attach(DbConnection::fairing())
        .attach(Template::fairing())
        .attach(AdHoc::on_request("Download Counter", |req, data| {
            if req.uri().path().contains("nightly") {
                let fname = req.uri().segments().last().unwrap().to_string();
                if downloads.contains_key(&fname) {
                    *downloads.get_mut(&fname).unwrap() += 1;
                } else {
                    downloads.insert(fname, 1);
                }
                File::open(format!("{}/downloads.json", env!("CARGO_MANIFEST_DIR")))
                    .unwrap()
                    .write_all(serde_json::to_string(&downloads).unwrap().as_bytes())
                    .unwrap();
            }
        }))
        .mount(
            "/",
            routes![
                routes::index,
                routes::recent,
                routes::about,
                routes::download_redirect
            ],
        )
        .mount(
            "/themes/users/",
            routes![
                routes::users::user_themes,
                routes::users::login,
                routes::users::create
            ],
        )
        .mount(
            "/themes/report/",
            routes![
                routes::report::report_view,
                routes::report::report_view_default
            ],
        )
        .mount("/themes/view/", routes![routes::theme])
        .mount("/themes/repo", routes![routes::download_theme])
        .mount(
            "/",
            rocket_contrib::serve::StaticFiles::from(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/static"
            )),
        )
        .mount(
            "/themes/meta",
            routes![
                routes::metadata::get_metadata,
                routes::metadata::post_metadata
            ],
        )
        .mount("/themes", routes![routes::upload_theme])
}
fn main() {
    rocket().launch();
}
