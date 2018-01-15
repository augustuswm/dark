use std::collections::HashMap;
use std::time::{Duration, Instant};

use hash_cache::HashCache;
use feature_flag::FeatureFlag;
use store::{Store, StoreError, StoreResult};

pub struct MemStore {
    data: HashCache<FeatureFlag>,
}

impl MemStore {
    pub fn new() -> MemStore {
        MemStore {
            data: HashCache::new(Duration::new(0, 0)),
        }
    }
}

impl From<HashMap<String, (FeatureFlag, Instant)>> for MemStore {
    fn from(map: HashMap<String, (FeatureFlag, Instant)>) -> MemStore {
        MemStore { data: map.into() }
    }
}

impl Store for MemStore {
    fn get(&self, key: &str) -> Option<FeatureFlag> {
        self.data
            .get(key)
            .and_then(|f| if !f.deleted() { Some(f) } else { None })
    }

    fn get_all(&self) -> StoreResult<HashMap<String, FeatureFlag>> {
        let mut map = self.data.get_all();
        map.retain(|_, flag| !flag.deleted());

        Ok(map)
    }

    fn delete(&self, key: &str, version: usize) -> StoreResult<()> {
        if let Some(flag) = self.get(key) {
            if flag.version() < version {
                let mut replacement = flag.clone();
                replacement.delete();
                replacement.update_version(version);
                self.data.insert(key, replacement);
                Ok(())
            } else {
                Err(StoreError::NewerVersionFound)
            }
        } else {
            Err(StoreError::NotFound)
        }
    }

    fn upsert(&self, key: &str, flag: &FeatureFlag) -> StoreResult<()> {
        let replacement = if let Some(e_flag) = self.get(key) {
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

        self.data.insert(key, replacement);
        Ok(())
    }

    fn init(&self, flags: HashMap<String, FeatureFlag>) -> StoreResult<()> {
        for (key, flag) in flags {
            self.upsert(key.as_str(), &flag);
        }

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
            vec![VariationValue::Integer(0), VariationValue::Integer(1)],
            deleted,
        )
    }

    fn dataset() -> MemStore {
        let mut map = HashMap::new();
        let flags = vec![flag("f1", 5, false), flag("f2", 5, true)];

        for flag in flags.into_iter() {
            map.insert(flag.key().into(), (flag, Instant::now()));
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
