#![allow(dead_code, unused_must_use, unused_variables)]

extern crate dark;

pub fn main() {}

#[cfg(test)]
mod tests {

    use dark::{FeatureFlag, Polling, RedisStore, Requestor, Store, VariationOrRollOut};

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
    fn test_polling() {
        let store = RedisStore::open("0.0.0.0".into(), 6379, Some("example_flags".into()), None)
            .unwrap();

        let req = Requestor::new(
            "https://app.launchdarkly.com",
            "sdk-00617963-388b-4ad4-b3c0-a49d1027ab7e",
        );

        let poll = Polling::new(store.clone(), req, 5);

        poll.run();

        ::std::thread::sleep(::std::time::Duration::new(2, 0));

        panic!("{:?}", store.get_all());
    }

    #[test]
    fn test_redis_store() {
        unimplemented!();
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
