extern crate dark;

use dark::{RedisStore, Store};

pub fn main() {}

#[cfg(test)]
mod tests {

    use dark::{FeatureFlag, RedisStore, Store, VariationOrRollOut};

    #[test]
    fn test_redis_store() {
        let r = RedisStore::open("0.0.0.0".into(), 6379, None, None).unwrap();

        let f = FeatureFlag::new(
            "f1".into(),
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
            false,
        );

        print!("\n");
        println!("{:?}", r.upsert(f.key(), &f));
        print!("\n");
        println!("{:?}", r.get(f.key()));

        assert!(false);
    }
}
