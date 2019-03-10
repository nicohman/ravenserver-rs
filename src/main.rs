#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
extern crate mongodb;
extern crate ravenserver;
#[macro_use]
extern crate rocket_contrib;
extern crate serde;
#[macro_use]
extern crate serde_derive;
use ravenserver::mongo::*;
use ravenserver::themes::*;
use rocket_contrib::databases;
use databases::mongodb::db::ThreadedDatabase;
#[database("mongodb")]
struct DbConnection(databases::mongodb::db::Database);
use mongodb::pool::*;
#[get("/")]
fn index(conn: DbConnection) ->  String {
    println!("{:?}", conn.0.list_collections(None).unwrap().next());
    let db = DataBase::from_db(conn.0.clone()).unwrap();
    let theme : Theme = db.find_one_key_value("name", "fall").unwrap().unwrap();
    format!("{:?}", theme)
}
fn main() {
    rocket::ignite()
        .attach(DbConnection::fairing())
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
