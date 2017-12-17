#![allow(dead_code, unused_must_use, unused_variables)]

extern crate dark;

pub fn main() {}

#[cfg(test)]
mod tests {

    use dark::{FeatureFlag, RedisStore, Store, VariationOrRollOut};

    fn flag(key: &str, deleted: bool) -> FeatureFlag {
        FeatureFlag::new(
            key.into(),
            1,
            true,
            vec![],
            "salt".into(),
            "sel".into(),
            vec![],
            vec![],
            VariationOrRollOut::Variation(0),
            Some(0),
            vec![],
            deleted,
        )
    }

    #[test]
    fn test_redis_store() {
        let r = RedisStore::open("0.0.0.0".into(), 6379, Some("example_flags".into()), None)
            .unwrap();

        let f1 = flag("ex_1", false);
        let f2 = flag("ex_2", true);
        let f3 = flag("ex_3", false);

        r.upsert(f1.key(), &f1);
        r.upsert(f2.key(), &f2);
        r.upsert(f3.key(), &f3);

        let all = r.get_all().unwrap();

        assert_eq!(all.len(), 2);
        assert_eq!(all.get("ex_1").unwrap(), &f1);
        assert!(all.get("ex_2").is_none());
        assert_eq!(all.get("ex_3").unwrap(), &f3);
    }
}
