use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct HashCache<T> {
    cache: Arc<RwLock<HashMap<String, (T, Instant)>>>,
    duration: Duration,
}

impl<T> From<HashMap<String, (T, Instant)>> for HashCache<T> {
    fn from(map: HashMap<String, (T, Instant)>) -> HashCache<T> {
        HashCache {
            cache: Arc::new(RwLock::new(map)),
            duration: Duration::new(0, 0),
        }
    }
}

impl<T> HashCache<T> {
    pub fn new(duration: Duration) -> HashCache<T> {
        HashCache {
            cache: Arc::new(RwLock::new(HashMap::new())),
            duration: duration,
        }
    }

    pub fn reader(&self) -> RwLockReadGuard<HashMap<String, (T, Instant)>> {
        match self.cache.read() {
            Ok(guard) => guard,
            Err(err) => {
                error!("Read guard for cache failed due to poisoning");
                panic!("{:?}", err)
            }
        }
    }

    pub fn writer(&self) -> RwLockWriteGuard<HashMap<String, (T, Instant)>> {
        match self.cache.write() {
            Ok(guard) => guard,
            Err(err) => {
                error!("Write guard for cache failed due to poisoning");
                panic!("{:?}", err)
            }
        }
    }

    fn ignore_dur(&self) -> bool {
        self.duration.as_secs() as f64 + self.duration.subsec_nanos() as f64 == 0.0
    }
}

impl<T: Clone> HashCache<T> {
    pub fn get<'a, S: Into<&'a str>>(&self, key: S) -> Option<T> {
        let map = self.reader();
        let entry = map.get(key.into());

        match entry {
            Some(&(ref val, created)) => {
                if self.ignore_dur() || created.elapsed() <= self.duration {
                    Some(val.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn get_all(&self) -> HashMap<String, T> {
        let mut res: HashMap<String, T> = HashMap::new();

        for (k, &(ref f, created)) in self.reader().iter() {
            if self.ignore_dur() || created.elapsed() <= self.duration {
                res.insert(k.clone(), f.clone());
            }
        }

        res
    }

    pub fn insert<S: Into<String>>(&self, key: S, val: T) -> Option<T> {
        self.writer()
            .insert(key.into(), (val, Instant::now()))
            .map(|(v, _)| v)
    }

    pub fn remove<'a, S: Into<&'a str>>(&self, key: S) -> Option<T> {
        self.writer().remove(key.into()).map(|(v, _)| v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get() {
        let cache = HashCache::new(Duration::new(5, 0));
        cache
            .writer()
            .insert("3".into(), (vec![1, 2, 3], Instant::now()));
        assert_eq!(Some(vec![1, 2, 3]), cache.get("3"));
    }
}
