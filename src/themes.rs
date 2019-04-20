use crate::mongo::*;
use mongodb::oid::*;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub date: String,
    pub author: String,
    pub pauthor: String,
    pub path: String,
    pub screen: String,
    pub updated: String,
    pub description: String,
    #[serde(default)]
    pub installs: f64,
    #[serde(default)]
    pub votes: f64,
    #[serde(default)]
    pub reports: Vec<Report>,
    #[serde(default)]
    pub approved: bool,
    #[serde(flatten)]
    pub mongo: MongoMetadata,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Report {
    pub date: mongodb::UtcDateTime,
    pub reason: String,
    pub info: String,
}
impl MongoDocument for Theme {
    fn collection_name() -> String {
        String::from("themes")
    }
    fn get_id<'a>(&'a self) -> &'a ObjectId {
        &self.mongo.id
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub name: String,
    pub id: String,
    #[serde(rename = "pass")]
    pub password: String,
    pub date: String,
    #[serde(flatten)]
    pub mongo: MongoMetadata,
}
impl MongoDocument for User {
    fn collection_name() -> String {
        String::from("users")
    }
    fn get_id<'a>(&'a self) -> &'a ObjectId {
        &self.mongo.id
    }
}
