use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use feature_flag::FeatureFlag;

pub type StoreResult<T> = Result<T, StoreError>;
pub enum StoreError {
    NotFound,
}

pub trait FeatureStore: Store + Sync {}
impl<T: Store + Sync> FeatureStore for T {}

pub trait Store {
    fn get(&self, key: &str) -> Option<FeatureFlag>;
    fn get_all(&self) -> StoreResult<HashMap<String, FeatureFlag>>;
    fn delete(&self, key: &str, version: i64) -> StoreResult<FeatureFlag>;
    fn upsert(&self, key: &str, flag: &FeatureFlag) -> StoreResult<()>;
}

pub struct MemStore {
    data: Arc<RwLock<HashMap<String, FeatureFlag>>>,
}

impl MemStore {
    pub fn new() -> MemStore {
        MemStore { data: Arc::new(RwLock::new(HashMap::new())) }
    }
}

impl Store for MemStore {
    fn get(&self, key: &str) -> Option<FeatureFlag> {
        self.data.read().expect("Store is corrupted").get(key).map(
            |f| {
                f.clone()
            },
        )
    }

    fn get_all(&self) -> StoreResult<HashMap<String, FeatureFlag>> {
        Ok(self.data.read().expect("Store is corrupted").clone())
    }

    fn delete(&self, key: &str, version: i64) -> StoreResult<FeatureFlag> {
        self.data
            .write()
            .expect("Store is corrupted")
            .remove(key)
            .ok_or(StoreError::NotFound)
    }

    fn upsert(&self, key: &str, flag: &FeatureFlag) -> StoreResult<()> {
        self.data.write().expect("Store is corrupted").insert(
            key.to_string(),
            flag.clone(),
        );
        Ok(())
    }
}
