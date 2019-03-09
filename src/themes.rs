use mongodb::coll::options::*;
use mongodb::coll::*;
use mongodb::cursor::*;
use mongodb::db::*;
use mongodb::*;
use serde::de::DeserializeOwned;
pub struct Themes {
    pub client: mongodb::Client,
    pub themes_collection: Collection,
    pub users_collection: Collection,
}
impl Themes {
    pub fn new(host: impl Into<String>, port: u16) -> Result<Themes> {
        let client = Client::connect(&host.into(), port)?;
        Ok(Themes {
            users_collection: client.db("themes").collection("users"),
            themes_collection: client.db("themes").collection("themes"),
            client: client,
        })
    }
    fn collection_by_name<'a>(&'a self, name: impl Into<String>) -> &'a Collection {
        match name.into().as_ref() {
            "themes" => &self.themes_collection,
            _ => panic!("wtf"),
        }
    }
    pub fn find_documents<T>(
        &self,
        filter: Document,
        options: Option<FindOptions>,
    ) -> Result<Vec<DecoderResult<T>>>
    where
        T: MongoDocument,
        T: DeserializeOwned,
    {
        let docs = self
            .collection_by_name(T::collection_name().as_ref())
            .find(Some(filter), options)?
            .drain_current_batch()?;
        Ok(docs.into_iter()
            .map(|x| mongodb::from_bson(mongodb::Bson::Document(x)))
            .collect())
    }
}
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
pub trait MongoDocument {
    fn collection_name() -> String;
}
impl MongoDocument for Theme {
    fn collection_name() -> String {
        String::from("themes")
    }
}
