use redis::RedisError;

use std::collections::HashMap;

use feature_flag::FeatureFlag;

pub type StoreResult<T> = Result<T, StoreError>;

#[derive(Debug, PartialEq)]
pub enum StoreError {
    InvalidRedisConfig,
    NewerVersionFound,
    NotFound,
    RedisFailure(RedisError),
}

pub trait FeatureStore: Store + Sync {}
impl<T: Store + Sync> FeatureStore for T {}

pub trait Store {
    fn get(&self, key: &str) -> Option<FeatureFlag>;
    fn get_all(&self) -> StoreResult<HashMap<String, FeatureFlag>>;
    fn delete(&self, key: &str, version: usize) -> StoreResult<()>;
    fn upsert(&self, key: &str, flag: &FeatureFlag) -> StoreResult<()>;
}
