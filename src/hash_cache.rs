use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Debug)]
pub struct HashCache<T> {
    cache: Arc<RwLock<HashMap<String, (T, i64)>>>,
}

impl<T> From<HashMap<String, (T, i64)>> for HashCache<T> {
    fn from(map: HashMap<String, (T, i64)>) -> HashCache<T> {
        HashCache { cache: Arc::new(RwLock::new(map)) }
    }
}

impl<T> HashCache<T> {
    pub fn new() -> HashCache<T> {
        HashCache { cache: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub fn reader(&self) -> RwLockReadGuard<HashMap<String, (T, i64)>> {
        match self.cache.read() {
            Ok(guard) => guard,
            Err(err) => {
                error!("Read guard for cache failed due to poisoning");
                panic!("{:?}", err)
            }
        }
    }

    pub fn writer(&self) -> RwLockWriteGuard<HashMap<String, (T, i64)>> {
        match self.cache.write() {
            Ok(guard) => guard,
            Err(err) => {
                error!("Write guard for cache failed due to poisoning");
                panic!("{:?}", err)
            }
        }
    }
}
