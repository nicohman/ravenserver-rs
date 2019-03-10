use mongodb::coll::options::*;
use mongodb::coll::*;
use mongodb::cursor::*;
use mongodb::db::*;
use mongodb::*;
use serde::de::DeserializeOwned;
use crate::themes::*;
pub struct DataBase {
    pub client: mongodb::Client,
    pub themes_collection: Collection,
    pub users_collection: Collection,
}
impl DataBase {
    pub fn new(host: impl Into<String>, port: u16) -> Result<DataBase> {
        let client = Client::connect(&host.into(), port)?;
        Ok(DataBase{
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
pub trait MongoDocument {
    fn collection_name() -> String;
}
