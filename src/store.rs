use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use feature_flag::FeatureFlag;

pub type StoreResult<T> = Result<T, StoreError>;

#[derive(Debug, PartialEq)]
pub enum StoreError {
    NotFound,
    NewerVersionFound,
}

pub trait FeatureStore: Store + Sync {}
impl<T: Store + Sync> FeatureStore for T {}

pub trait Store {
    fn get(&self, key: &str) -> Option<FeatureFlag>;
    fn get_all(&self) -> HashMap<String, FeatureFlag>;
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

    fn get_all(&self) -> HashMap<String, FeatureFlag> {
        let mut res = self.reader().clone();
        res.retain(|k, f| !f.deleted());
        res
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
    use std::collections::HashMap;

    use feature_flag::*;
    use store::*;

    fn flag<S: Into<String>>(key: S, version: usize, deleted: bool) -> FeatureFlag {
        FeatureFlag::new(
            key.into(),
            version,
            true,
            vec![],
            "".into(),
            "".into(),
            vec![],
            vec![],
            VariationOrRollOut::Variation(0),
            None,
            vec![0, 1],
            deleted,
        )
    }

    fn dataset() -> MemStore {
        let mut map = HashMap::new();
        let flags = vec![flag("f1", 5, false), flag("f2", 5, true)];

        for flag in flags.into_iter() {
            map.insert(flag.key().into(), flag);
        }

        map.into()
    }

    #[test]
    fn test_does_not_return_deleted() {
        assert!(dataset().get("f2").is_none())
    }

    #[test]
    fn test_all_returns_only_not_deleted() {
        let all = dataset().get_all();
        assert!(all.get("f1".into()).is_some());
        assert!(all.get("f2".into()).is_none());
    }

    #[test]
    fn test_delete_does_not_delete_newer_version() {
        assert_eq!(
            dataset().delete("f1", 3),
            Err(StoreError::NewerVersionFound)
        )
    }

    #[test]
    fn test_upsert_does_not_replace_newer_version() {
        assert_eq!(
            dataset().upsert("f1", &flag("f1", 3, false)),
            Err(StoreError::NewerVersionFound)
        )
    }
}
