#![allow(dead_code, unused_must_use, unused_imports, unused_variables)]

extern crate chrono;
extern crate eventsource;
#[macro_use]
extern crate log;
extern crate redis;
extern crate regex;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate sha1;

mod clause;
mod client;
mod config;
mod events;
mod feature_flag;
mod hash_cache;
mod mem_store;
mod poll;
mod redis_store;
mod request;
mod store;
mod stream;
mod user;

const VERSION: &'static str = "0.1.0";

pub use client::Client;
pub use config::{Config, ConfigBuilder};
pub use feature_flag::{FeatureFlag, VariationOrRollOut};
pub use mem_store::MemStore;
pub use poll::Polling;
pub use redis_store::RedisStore;
pub use request::Requestor;
pub use store::{Store, StoreError, StoreResult};
pub use stream::Streaming;
pub use user::{User, UserBuilder};

#[cfg(test)]
mod tests {}
