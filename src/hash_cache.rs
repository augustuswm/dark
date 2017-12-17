use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use feature_flag::FeatureFlag;

#[derive(Debug)]
pub struct HashCache {
    cache: Arc<RwLock<HashMap<String, (FeatureFlag, i64)>>>,
}

impl From<HashMap<String, (FeatureFlag, i64)>> for HashCache {
    fn from(map: HashMap<String, (FeatureFlag, i64)>) -> HashCache {
        HashCache { cache: Arc::new(RwLock::new(map)) }
    }
}

impl HashCache {
    pub fn new() -> HashCache {
        HashCache { cache: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub fn reader(&self) -> RwLockReadGuard<HashMap<String, (FeatureFlag, i64)>> {
        match self.cache.read() {
            Ok(guard) => guard,
            Err(err) => {
                error!("Read guard for cache failed due to poisoning");
                panic!("{:?}", err)
            }
        }
    }

    pub fn writer(&self) -> RwLockWriteGuard<HashMap<String, (FeatureFlag, i64)>> {
        match self.cache.write() {
            Ok(guard) => guard,
            Err(err) => {
                error!("Write guard for cache failed due to poisoning");
                panic!("{:?}", err)
            }
        }
    }
}
