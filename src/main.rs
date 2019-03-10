#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
extern crate mongodb;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate bson;
fn main() {
    println!("Hello, world!");
}
