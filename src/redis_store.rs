use redis::{Client, Commands, RedisResult};

use std::collections::HashMap;

use feature_flag::FeatureFlag;
use hash_cache::HashCache;
use store::{Store, StoreResult, StoreError};

pub struct RedisStore {
    key: String,
    client: Client,
    cache: HashCache,
    timeout: u8,
}

impl RedisStore {
    pub fn open(
        host: String,
        port: u32,
        prefix: Option<String>,
        timeout: Option<u8>,
    ) -> StoreResult<RedisStore> {
        RedisStore::open_with_url(format!("redis://{}:{}", host, port), prefix, timeout)
    }

    pub fn open_with_url(
        url: String,
        prefix: Option<String>,
        timeout: Option<u8>,
    ) -> StoreResult<RedisStore> {
        let client = Client::open(url.as_str()).map_err(
            |_| StoreError::InvalidRedisConfig,
        )?;

        Ok(RedisStore::open_with_client(client, prefix, timeout))
    }

    pub fn open_with_client(
        client: Client,
        prefix: Option<String>,
        timeout: Option<u8>,
    ) -> RedisStore {
        RedisStore {
            key: RedisStore::features_key(prefix),
            client: client,
            cache: HashCache::new(),
            timeout: timeout.unwrap_or(0),
        }
    }

    fn features_key(prefix: Option<String>) -> String {
        prefix.unwrap_or("launchdarkly".into()) + ":features"
    }
}

impl Store for RedisStore {
    fn get(&self, key: &str) -> Option<FeatureFlag> {
        self.client.hget(self.key.to_string(), key.to_string()).ok()
    }

    fn get_all(&self) -> StoreResult<HashMap<String, FeatureFlag>> {
        self.client.hgetall(self.key.to_string()).map_err(
            StoreError::RedisFailure,
        )
    }

    fn delete(&self, key: &str, version: usize) -> StoreResult<()> {
        unimplemented!();
    }

    fn upsert(&self, key: &str, flag: &FeatureFlag) -> StoreResult<()> {
        let res: RedisResult<FeatureFlag> = self.client.hset(
            self.key.to_string(),
            flag.key().to_string(),
            flag,
        );

        res.map(|_| ()).map_err(StoreError::RedisFailure)
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
        let mut map: HashMap<String, FeatureFlag> = HashMap::new();
        let flags = vec![flag("f1", 5, false), flag("f2", 5, true)];

        for flag in flags.into_iter() {
            map.insert(flag.key().into(), flag);
        }

        RedisStore::open("".into(), 0, None, None).unwrap()
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
