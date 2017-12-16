

impl FromRedisValue for FeatureFlag {
    fn from_redis_value(v: &RedisValue) -> RedisResult<FeatureFlag> {
        let default = FeatureFlag::new(
            "default".into(),
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

        match *v {
            RedisValue::Data(ref data) => Ok(
                serde_json::from_str(
                    String::from_utf8(data.clone()).unwrap().as_str(),
                ).unwrap(),
            ),
            _ => Ok(default),
        }
    }
}

impl ToRedisArgs for FeatureFlag {
    fn to_redis_args(&self) -> Vec<Vec<u8>> {
        self.to_redis_args()
    }
}

impl<'a> ToRedisArgs for &'a FeatureFlag {
    fn to_redis_args(&self) -> Vec<Vec<u8>> {
        vec![serde_json::to_string(self).unwrap().as_bytes().to_vec()]
    }
}
