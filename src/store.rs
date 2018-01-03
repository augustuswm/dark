use redis::RedisError;

use std::collections::HashMap;

use feature_flag::FeatureFlag;

pub type StoreResult<T> = Result<T, StoreError>;

#[derive(Debug, PartialEq)]
pub enum StoreError {
    FailedToSerializeFlag,
    InvalidRedisConfig,
    NewerVersionFound,
    NotFound,
    RedisFailure(RedisError),
}

pub trait Store: Sync + Send {
    fn get(&self, key: &str) -> Option<FeatureFlag>;
    fn get_all(&self) -> StoreResult<HashMap<String, FeatureFlag>>;
    fn delete(&self, key: &str, version: usize) -> StoreResult<()>;
    fn upsert(&self, key: &str, flag: &FeatureFlag) -> StoreResult<()>;
    fn init(&self, flags: HashMap<String, FeatureFlag>) -> StoreResult<()>;
}
