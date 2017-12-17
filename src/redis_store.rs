use chrono::Utc;
use redis::{Client, cmd, Commands, Connection, FromRedisValue, RedisResult, ToRedisArgs};

use std::collections::HashMap;

use feature_flag::FeatureFlag;
use hash_cache::HashCache;
use store::{Store, StoreResult, StoreError};

const FAIL: &'static [u8; 4] = &[102, 97, 105, 108];
const ALL_CACHE: &'static str = "$all_flags$";

pub struct RedisStore {
    key: String,
    conn: Connection,
    cache: HashCache<FeatureFlag>,
    all_cache: HashCache<HashMap<String, FeatureFlag>>,
    timeout: u64,
}

impl RedisStore {
    pub fn open(
        host: String,
        port: u32,
        prefix: Option<String>,
        timeout: Option<u64>,
    ) -> StoreResult<RedisStore> {
        RedisStore::open_with_url(format!("redis://{}:{}", host, port), prefix, timeout)
    }

    pub fn open_with_url(
        url: String,
        prefix: Option<String>,
        timeout: Option<u64>,
    ) -> StoreResult<RedisStore> {
        let client = Client::open(url.as_str()).map_err(
            |_| StoreError::InvalidRedisConfig,
        )?;

        // Get a single connection to group all of the requests on
        let c = client.get_connection().map_err(StoreError::RedisFailure)?;

        Ok(RedisStore::open_with_connection(c, prefix, timeout))
    }

    pub fn open_with_connection(
        conn: Connection,
        prefix: Option<String>,
        timeout: Option<u64>,
    ) -> RedisStore {
        RedisStore {
            key: RedisStore::features_key(prefix),
            conn: conn,
            cache: HashCache::new(),
            all_cache: HashCache::new(),
            timeout: timeout.unwrap_or(0),
        }
    }

    fn expiration_starting_now(&self) -> i64 {
        Utc::now().timestamp() + self.timeout as i64
    }

    fn features_key(prefix: Option<String>) -> String {
        prefix.unwrap_or("launchdarkly".into()) + ":features"
    }

    fn get_raw(&self, key: &str) -> Option<FeatureFlag> {
        self.conn.hget(self.key.to_string(), key.to_string()).ok()
    }

    fn start<T: FromRedisValue>(&self, key: &str) -> StoreResult<()> {
        let res: RedisResult<T> = cmd("WATCH").arg(key).query(&self.conn);
        res.map(|_| ()).map_err(StoreError::RedisFailure)
    }

    fn cleanup<T: FromRedisValue>(&self) -> StoreResult<()> {
        let res: RedisResult<T> = cmd("UNWATCH").query(&self.conn);
        res.map(|_| ()).map_err(StoreError::RedisFailure)
    }
}

impl Store for RedisStore {
    fn get(&self, key: &str) -> Option<FeatureFlag> {

        // Checks individual cache

        if let Some(&(ref flag, exp)) = self.cache.reader().get(key) {
            let now = Utc::now().timestamp();

            if exp < now {
                return Some(flag.clone());
            }
        };

        self.get_raw(key).and_then(
            |flag: FeatureFlag| if !flag.deleted() {
                self.cache.writer().insert(key.into(), (
                    flag.clone(),
                    self.expiration_starting_now(),
                ));

                Some(flag)
            } else {
                None
            },
        )
    }

    fn get_all(&self) -> StoreResult<HashMap<String, FeatureFlag>> {

        // Checks all cache

        if let Some(&(ref map, exp)) = self.all_cache.reader().get(ALL_CACHE) {
            let now = Utc::now().timestamp();

            if exp < now {
                return Ok(map.clone());
            }
        };

        self.conn
            .hgetall(self.key.to_string())
            .map(|mut map: HashMap<String, FeatureFlag>| {
                map.retain(|k, flag| !flag.deleted());

                self.all_cache.writer().insert(ALL_CACHE.into(), (
                    map.clone(),
                    self.expiration_starting_now(),
                ));

                map
            })
            .map_err(StoreError::RedisFailure)
    }

    fn delete(&self, key: &str, version: usize) -> StoreResult<()> {

        // Ignores cache lookup

        let _: () = self.start::<()>(key)?;

        if let Some(flag) = self.get_raw(key) {
            if flag.version() < version {
                let mut replacement = flag.clone();
                replacement.delete();
                replacement.update_version(version);
                self.upsert(key.into(), &replacement);

                self.cleanup::<()>()
            } else {
                self.cleanup::<()>();
                Err(StoreError::NewerVersionFound)
            }
        } else {
            self.cleanup::<()>();
            Err(StoreError::NotFound)
        }
    }

    fn upsert(&self, key: &str, flag: &FeatureFlag) -> StoreResult<()> {

        // Ignores cache lookup

        let _: () = self.start::<()>(key)?;

        let replacement = if let Some(e_flag) = self.get_raw(key) {
            if e_flag.version() < flag.version() {
                Ok(flag)
            } else {
                warn!(
                    "Can not overwrite flag with key {:?} in store with older version",
                    key
                );
                Err(StoreError::NewerVersionFound)
            }
        } else {
            Ok(flag)
        }?;

        let string_rep = replacement.to_redis_args();

        if string_rep[0].as_slice() != FAIL {
            let res: RedisResult<u8> = self.conn.hset(
                self.key.to_string(),
                replacement.key().to_string(),
                string_rep,
            );

            self.all_cache.writer().remove(ALL_CACHE);
            self.cache.writer().insert(key.to_string(), (
                replacement.clone(),
                self.expiration_starting_now(),
            ));

            self.cleanup::<()>();
            res.map(|_| ()).map_err(StoreError::RedisFailure)
        } else {

            self.cleanup::<()>();
            Err(StoreError::FailedToSerializeFlag)
        }
    }
}


#[cfg(test)]
mod tests {
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
        let store = RedisStore::open("0.0.0.0".into(), 6379, None, Some(3600)).unwrap();
        let flags = vec![flag("f1", 5, false), flag("f2", 5, true)];

        for flag in flags.into_iter() {
            store.upsert(flag.key(), &flag);
        }

        store
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
