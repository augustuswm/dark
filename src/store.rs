use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use feature_flag::FeatureFlag;

pub type StoreResult<T> = Result<T, StoreError>;
pub enum StoreError {
    NotFound,
    NewerVersionFound,
}

pub trait FeatureStore: Store + Sync {}
impl<T: Store + Sync> FeatureStore for T {}

pub trait Store {
    fn get(&self, key: &str) -> Option<FeatureFlag>;
    fn get_all(&self) -> StoreResult<HashMap<String, FeatureFlag>>;
    fn delete(&self, key: &str, version: usize) -> StoreResult<()>;
    fn upsert(&self, key: &str, flag: &FeatureFlag) -> StoreResult<()>;
}

pub struct MemStore {
    data: Arc<RwLock<HashMap<String, FeatureFlag>>>,
}

impl MemStore {
    pub fn new() -> MemStore {
        MemStore { data: Arc::new(RwLock::new(HashMap::new())) }
    }

    fn get_raw(&self, key: &str) -> Option<FeatureFlag> {
        self.reader().get(key).map(|f| f.clone())
    }

    fn reader(&self) -> RwLockReadGuard<HashMap<String, FeatureFlag>> {
        match self.data.read() {
            Ok(guard) => guard,
            Err(err) => {
                error!("Read guard for store failed due to store being poisoned.");
                panic!("{:?}", err)
            }
        }
    }

    fn writer(&self) -> RwLockWriteGuard<HashMap<String, FeatureFlag>> {
        match self.data.write() {
            Ok(guard) => guard,
            Err(err) => {
                error!("Write guard for store failed due to store being poisoned.");
                panic!("{:?}", err)
            }
        }
    }
}

impl From<HashMap<String, FeatureFlag>> for MemStore {
    fn from(map: HashMap<String, FeatureFlag>) -> MemStore {
        MemStore { data: Arc::new(RwLock::new(map)) }
    }
}

impl Store for MemStore {
    fn get(&self, key: &str) -> Option<FeatureFlag> {
        self.get_raw(key).and_then(|f| if !f.deleted() {
            Some(f)
        } else {
            None
        })
    }

    fn get_all(&self) -> StoreResult<HashMap<String, FeatureFlag>> {
        let mut res = self.reader().clone();
        res.retain(|k, f| !f.deleted());
        Ok(res)
    }

    fn delete(&self, key: &str, version: usize) -> StoreResult<()> {
        if let Some(flag) = self.get_raw(key) {
            if flag.version() < version {
                let mut replacement = flag.clone();
                replacement.delete();
                replacement.update_version(version);
                self.writer().insert(key.into(), replacement);
                Ok(())
            } else {
                Err(StoreError::NewerVersionFound)
            }
        } else {
            Err(StoreError::NotFound)
        }
    }

    fn upsert(&self, key: &str, flag: &FeatureFlag) -> StoreResult<()> {
        let replacement = if let Some(e_flag) = self.get_raw(key) {
            if e_flag.version() < flag.version() {
                Ok(flag.clone())
            } else {
                warn!(
                    "Can not overwrite flag with key {:?} in store with older version",
                    key
                );
                Err(StoreError::NewerVersionFound)
            }
        } else {
            Ok(flag.clone())
        };

        replacement.map(|f| {
            self.writer().insert(key.into(), f);
            ()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_does_not_return_deleted() {
        unimplemented!()
    }

    #[test]
    fn test_delete_does_not_delete_newer_version() {
        unimplemented!()
    }

    #[test]
    fn test_upsert_does_not_replace_newer_version() {
        unimplemented!()
    }
}
