#![allow(dead_code, unused_must_use, unused_variables)]

extern crate chrono;
#[macro_use]
extern crate log;
extern crate redis;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate sha1;

mod clause;
mod events;
mod feature_flag;
mod hash_cache;
mod mem_store;
mod redis_store;
mod store;
mod user;

pub use feature_flag::{FeatureFlag, VariationOrRollOut};
pub use mem_store::MemStore;
pub use redis_store::RedisStore;
pub use store::{FeatureStore, Store, StoreError, StoreResult};

#[cfg(test)]
mod tests {}
