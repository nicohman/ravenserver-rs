use crate::mongo::*;
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
    pub installs: i32,
    #[serde(default)]
    pub votes: i32,
    #[serde(default)]
    pub reports: Vec<Report>,
    #[serde(default)]
    pub approved: bool,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Report {
    pub date: String,
    pub reason: String,
    pub info: String,
}
impl MongoDocument for Theme {
    fn collection_name() -> String {
        String::from("themes")
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub name: String,
    pub id: String,
    #[serde(rename = "pass")]
    pub password: String,
    pub date: String
}
impl MongoDocument for User {
    fn collection_name() -> String {
        String::from("users")
    }
}
