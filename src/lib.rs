#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
extern crate mongodb;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate bson;
/// Provides structs and functionality for specific document schemas
pub mod themes;
/// Provides functionality for MongoDB operations
pub mod mongo;
