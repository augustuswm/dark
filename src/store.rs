use std::collections::HashMap;

use feature_flag::FeatureFlag;

type StoreResult<T> = Result<T, StoreError>;
type StoreError = i64;

pub trait FeatureStore: Store + Sync {}
impl<T: Store + Sync> FeatureStore for T {}

pub trait Store {
    fn get(&self, key: &str) -> StoreResult<FeatureFlag>;
    fn get_all(&self) -> StoreResult<HashMap<String, FeatureFlag>>;
    fn delete(&self, key: &str, version: i64) -> StoreResult<()>;
    fn upsert(&self, key: &str, flag: &FeatureFlag);
}
