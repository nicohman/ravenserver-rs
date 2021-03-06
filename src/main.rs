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
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use mongodb::ThreadedClient;
mod auth;
mod routes;
pub fn rocket() -> rocket::Rocket {
    let client = mongodb::Client::connect("localhost", 27017).expect("Couldn't initialize mongodb");
    rocket::ignite()
        .manage(client)
        .attach(Template::fairing())
        .attach(AdHoc::on_request("Download Counter", |req, data| {
            if req.uri().path().contains("nightly") {
                let mut st = String::new();
                File::open(format!("{}/downloads.json", env!("CARGO_MANIFEST_DIR")))
                    .expect("Couldn't open downloads storage file")
                    .read_to_string(&mut st)
                    .expect("Couldn't read download counting file");
                let mut downloads: HashMap<String, i64> = serde_json::from_str(&st).unwrap();
                let fname = req.uri().segments().last().unwrap().to_string();
                if downloads.contains_key(&fname) {
                    *downloads.get_mut(&fname).unwrap() += 1;
                } else {
                    downloads.insert(fname, 1);
                }
                let path = format!("{}/downloads.json", env!("CARGO_MANIFEST_DIR"));
                                   println!("{}", path);
                File::open(path)
                    .unwrap();
                    //.write_all(serde_json::to_string(&downloads).unwrap().as_bytes())
                    //.unwrap();
            }
        }))
        .mount(
            "/",
            routes![
                routes::index,
                routes::recent,
                routes::about,
                routes::download_redirect,
                routes::checksums
            ],
        )
        .mount(
            "/themes/users/",
            routes![
                routes::users::user_themes,
                routes::users::login,
                routes::users::create,
                routes::users::delete
            ],
        )
        .mount(
            "/themes/user/",
            routes![routes::users::login, routes::users::create],
        )
        .mount(
            "/themes/report/",
            routes![
                routes::report::report_view,
                routes::report::report_view_default
            ],
        )
        .mount(
            "/",
            rocket_contrib::serve::StaticFiles::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/static"
            ), rocket_contrib::serve::Options::DotFiles),
        )
        .mount(
            "/themes/meta",
            routes![
                routes::metadata::get_metadata,
                routes::metadata::post_metadata
            ],
        )
        .mount(
            "/themes",
            routes![
                routes::themes::download_theme,
                routes::themes::theme,
                routes::themes::upload_theme,
                routes::themes::delete_theme
            ],
        )
}
fn main() {
    rocket().launch();
}
