use std::collections::HashMap;

use feature_flag::FeatureFlag;
use store::{Store, StoreResult, StoreError};

pub struct RedisStore {}

impl RedisStore {
    pub fn new() -> RedisStore {
        RedisStore {}
    }
}

impl From<HashMap<String, FeatureFlag>> for RedisStore {
    fn from(map: HashMap<String, FeatureFlag>) -> RedisStore {
        RedisStore {}
    }
}

impl Store for RedisStore {
    fn get(&self, key: &str) -> Option<FeatureFlag> {
        unimplemented!();
    }

    fn get_all(&self) -> HashMap<String, FeatureFlag> {
        unimplemented!();
    }

    fn delete(&self, key: &str, version: usize) -> StoreResult<()> {
        unimplemented!();
    }

    fn upsert(&self, key: &str, flag: &FeatureFlag) -> StoreResult<()> {
        unimplemented!();
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use feature_flag::*;
    use redis_store::*;

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

    fn dataset() -> RedisStore {
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
