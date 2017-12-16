#![allow(dead_code, unused_variables)]

extern crate chrono;
#[macro_use]
extern crate log;
extern crate redis;
extern crate regex;
extern crate sha1;

mod clause;
mod events;
mod feature_flag;
mod hash_cache;
mod mem_store;
mod redis_store;
mod store;
mod user;

#[cfg(test)]
mod tests {}
