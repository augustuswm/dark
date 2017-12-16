use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use feature_flag::FeatureFlag;

pub struct HashCache {
    cache: Arc<RwLock<HashMap<String, FeatureFlag>>>,
}

impl From<HashMap<String, FeatureFlag>> for HashCache {
    fn from(map: HashMap<String, FeatureFlag>) -> HashCache {
        HashCache { cache: Arc::new(RwLock::new(map)) }
    }
}

impl HashCache {
    pub fn new() -> HashCache {
        HashCache { cache: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub fn reader(&self) -> RwLockReadGuard<HashMap<String, FeatureFlag>> {
        match self.cache.read() {
            Ok(guard) => guard,
            Err(err) => {
                error!("Read guard for cache failed due to poisoning");
                panic!("{:?}", err)
            }
        }
    }

    pub fn writer(&self) -> RwLockWriteGuard<HashMap<String, FeatureFlag>> {
        match self.cache.write() {
            Ok(guard) => guard,
            Err(err) => {
                error!("Write guard for cache failed due to poisoning");
                panic!("{:?}", err)
            }
        }
    }
}
