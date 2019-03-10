use crate::Result;
use mongodb::coll::options::*;
use mongodb::coll::results::*;
use mongodb::coll::*;
use mongodb::db::*;
use mongodb::*;
use serde::de::DeserializeOwned;
use serde::ser::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
pub struct DataBase {
    pub client: mongodb::Client,
    pub collections: RefCell<HashMap<String, Rc<Collection>>>,
    db_name: String,
}
impl DataBase {
    pub fn new(host: impl Into<String>, port: u16, db_name: impl Into<String>) -> Result<DataBase> {
        let client = Client::connect(&host.into(), port)?;
        Ok(DataBase {
            collections: RefCell::new(HashMap::new()),
            client: client,
            db_name: db_name.into(),
        })
    }
    fn collection_by_name(&self, name: impl Into<String>) -> Rc<Collection> {
        let name = name.into();
        let has = self.collections.borrow().contains_key(&name);
        if has {
            self.collections.borrow().get(&name).unwrap().clone()
        } else {
            let collection = Rc::new(self.client.db(&self.db_name).collection(&name));
            self.collections
                .borrow_mut()
                .insert(name.clone(), collection.clone());
            collection
        }
    }
    fn collection_by_type<T: MongoDocument>(&self) -> Rc<Collection> {
        self.collection_by_name(T::collection_name())
    }
    pub fn find<T>(
        &self,
        filter: Document,
        options: Option<FindOptions>,
    ) -> Result<Vec<DecoderResult<T>>>
    where
        T: MongoDocument,
        T: DeserializeOwned,
    {
        let docs = self
            .collection_by_type::<T>()
            .find(Some(filter), options)?
            .drain_current_batch()?;
        Ok(docs
            .into_iter()
            .map(|x| mongodb::from_bson(mongodb::Bson::Document(x)))
            .collect())
    }
    pub fn find_one<T>(&self, filter: Document, options: Option<FindOptions>) -> Result<Option<T>>
    where
        T: MongoDocument,
        T: DeserializeOwned,
    {
        if let Some(doc) = self
            .collection_by_type::<T>()
            .find_one(Some(filter), options)?
        {
            Ok(Some(mongodb::from_bson(mongodb::Bson::Document(doc))?))
        } else {
            Ok(None)
        }
    }
    fn construct_filter_key_value(key: impl Into<String>, value: impl Into<Bson>) -> Document {
        let mut filter = mongodb::ordered::OrderedDocument::new();
        filter.insert(key, value);
        filter
    }
    pub fn find_one_key_value<T>(
        &self,
        key: impl Into<String>,
        value: impl Into<Bson>,
    ) -> Result<Option<T>>
    where
        T: MongoDocument,
        T: DeserializeOwned,
    {
        let filter = DataBase::construct_filter_key_value(key, value);
        self.find_one(filter, None)
    }
    pub fn find_key_value<T>(
        &self,
        key: impl Into<String>,
        value: impl Into<Bson>,
        options: Option<FindOptions>,
    ) -> Result<Vec<DecoderResult<T>>>
    where
        T: MongoDocument,
        T: DeserializeOwned,
    {
        let filter = DataBase::construct_filter_key_value(key, value);
        self.find(filter, options)
    }
    pub fn save<T>(&self, document: T, options: Option<UpdateOptions>) -> Result<UpdateResult>
    where
        T: Serialize,
        T: MongoDocument,
    {
        Ok(self.collection_by_type::<T>().replace_one(
            doc!{"_id": document.get_id().clone()},
            mongodb::to_bson(&document)?.as_document().unwrap().clone(),
            options,
        )?)
    }
}
pub trait MongoDocument {
    fn collection_name() -> String;
    fn get_id<'a>(&'a self) -> &'a mongodb::oid::ObjectId;
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MongoMetadata {
    #[serde(rename = "_id")]
    pub id: mongodb::oid::ObjectId,
}
