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
    client: Client,
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

        Ok(RedisStore::open_with_client(client, prefix, timeout))
    }

    pub fn open_with_client(
        client: Client,
        prefix: Option<String>,
        timeout: Option<u64>,
    ) -> RedisStore {
        RedisStore {
            key: RedisStore::features_key(prefix),
            client: client,
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

    fn conn(&self) -> StoreResult<Connection> {

        // Get a single connection to group requests on
        self.client.get_connection().map_err(
            StoreError::RedisFailure,
        )
    }

    fn get_raw(&self, key: &str, conn: &Connection) -> Option<FeatureFlag> {

        conn.hget(self.key.to_string(), key.to_string()).ok()
    }

    fn put(&self, key: &str, flag: &FeatureFlag, conn: &Connection) -> StoreResult<()> {

        // Manually serialize to redis storable value to allow for failure handling
        let flag_ser = flag.to_redis_args();

        if flag_ser[0].as_slice() != FAIL {
            let res: RedisResult<u8> =
                conn.hset(self.key.to_string(), flag.key().to_string(), flag_ser);

            self.all_cache.writer().remove(ALL_CACHE);
            self.cache.writer().insert(key.to_string(), (
                flag.clone(),
                self.expiration_starting_now(),
            ));

            res.map(|_| ()).map_err(StoreError::RedisFailure)
        } else {
            Err(StoreError::FailedToSerializeFlag)
        }
    }

    fn start<T: FromRedisValue>(&self, key: &str, conn: &Connection) -> StoreResult<()> {

        let res: RedisResult<T> = cmd("WATCH").arg(key).query(conn);
        res.map(|_| ()).map_err(StoreError::RedisFailure)
    }

    fn cleanup<T: FromRedisValue>(&self, conn: &Connection) -> StoreResult<()> {

        let res: RedisResult<T> = cmd("UNWATCH").query(conn);
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

        self.conn().ok().and_then(|conn| {
            self.get_raw(key, &conn).and_then(|flag: FeatureFlag| {
                if !flag.deleted() {
                    self.cache.writer().insert(key.into(), (
                        flag.clone(),
                        self.expiration_starting_now(),
                    ));

                    Some(flag)
                } else {
                    None
                }
            })
        })
    }

    fn get_all(&self) -> StoreResult<HashMap<String, FeatureFlag>> {

        // Checks all cache

        if let Some(&(ref map, exp)) = self.all_cache.reader().get(ALL_CACHE) {
            let now = Utc::now().timestamp();

            if exp < now {
                return Ok(map.clone());
            }
        };

        self.conn()?
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

        let conn = self.conn()?;

        let _: () = self.start::<()>(key, &conn)?;

        let res = if let Some(flag) = self.get_raw(key, &conn) {
            if flag.version() < version {
                let mut replacement = flag.clone();
                replacement.delete();
                replacement.update_version(version);

                self.put(key, &replacement, &conn)
            } else {
                Err(StoreError::NewerVersionFound)
            }
        } else {
            Err(StoreError::NotFound)
        };

        self.cleanup::<()>(&conn);
        res
    }

    fn upsert(&self, key: &str, flag: &FeatureFlag) -> StoreResult<()> {

        // Ignores cache lookup

        let conn = self.conn()?;

        let _: () = self.start::<()>(key, &conn)?;

        let replacement = if let Some(e_flag) = self.get_raw(key, &conn) {
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

        let res = self.put(key, replacement, &conn);

        self.cleanup::<()>(&conn);
        res
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
