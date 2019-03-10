#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate failure;
extern crate mongodb;
extern crate serde;
#[macro_use]
extern crate serde_derive;

pub mod error;
pub use error::{Result};
/// Provides functionality for MongoDB operations
pub mod mongo;
/// Provides structs and functionality for specific document schemas
pub mod themes;
