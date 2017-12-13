use std::collections::HashMap;

use feature_flag::FeatureFlag;

type StoreResult<T> = Result<T, StoreError>;
type StoreError = i64;

pub trait FeatureStore: Store + Sync {}
impl<T: Store + Sync> FeatureStore for T {}

pub trait Store {
    fn get(key: &str) -> StoreResult<FeatureFlag>;
    fn get_all() -> StoreResult<HashMap<String, FeatureFlag>>;
    fn delete(key: &str, version: i64) -> StoreResult<()>;
    fn upsert(key: &str, flag: &FeatureFlag);
}
