use std::collections::HashMap;

use hash_cache::HashCache;
use feature_flag::FeatureFlag;
use store::{Store, StoreResult, StoreError};

pub struct MemStore {
    data: HashCache,
}

impl MemStore {
    pub fn new() -> MemStore {
        MemStore { data: HashCache::new() }
    }

    fn get_raw(&self, key: &str) -> Option<FeatureFlag> {
        self.data.reader().get(key).map(|&(ref f, exp)| f.clone())
    }
}

impl From<HashMap<String, (FeatureFlag, i64)>> for MemStore {
    fn from(map: HashMap<String, (FeatureFlag, i64)>) -> MemStore {
        MemStore { data: map.into() }
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
        let data = self.data.reader();
        let mut res: HashMap<String, FeatureFlag> = HashMap::new();

        for (k, &(ref f, exp)) in data.iter() {
            if !f.deleted() {
                res.insert(k.clone(), f.clone());
            }
        }

        Ok(res)
    }

    fn delete(&self, key: &str, version: usize) -> StoreResult<()> {
        if let Some(flag) = self.get_raw(key) {
            if flag.version() < version {
                let mut replacement = flag.clone();
                replacement.delete();
                replacement.update_version(version);
                self.data.writer().insert(key.into(), (replacement, 0));
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
        }?;

        self.data.writer().insert(key.into(), (replacement, 0));
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use feature_flag::*;
    use mem_store::*;

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
            map.insert(flag.key().into(), (flag, 0));
        }

        map.into()
    }

    #[test]
    fn test_does_not_return_deleted() {
        assert!(dataset().get("f2").is_none())
    }

    #[test]
    fn test_all_returns_only_not_deleted() {
        let all = dataset().get_all().unwrap();
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
