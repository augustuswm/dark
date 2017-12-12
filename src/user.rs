extern crate sha1;

use self::sha1::{Sha1, Digest};

use std::collections::HashMap;

use feature_flag::Value;

static LONG_SCALE: u64 = 0xFFFFFFFFFFFFFFF;

pub struct UserBuilder {
    key: String,
    secondary: Option<String>,
    ip: Option<String>,
    country: Option<String>,
    email: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    avatar: Option<String>,
    name: Option<String>,
    anonymous: bool,
    custom: HashMap<String, String>,
    derived: HashMap<String, DerivedAttribute>,
    private_attributes: Vec<String>,
}

impl UserBuilder {
    pub fn new<S: Into<String>>(key: S) -> UserBuilder {
        UserBuilder {
            key: key.into(),
            secondary: None,
            ip: None,
            country: None,
            email: None,
            first_name: None,
            last_name: None,
            avatar: None,
            name: None,
            anonymous: false,
            custom: HashMap::new(),
            derived: HashMap::new(),
            private_attributes: vec![],
        }
    }

    pub fn secondary(mut self, secondary: Option<String>) -> Self {
        self.secondary = secondary;
        self
    }

    pub fn ip(mut self, ip: Option<String>) -> Self {
        self.ip = ip;
        self
    }

    pub fn country(mut self, country: Option<String>) -> Self {
        self.country = country;
        self
    }

    pub fn email(mut self, email: Option<String>) -> Self {
        self.email = email;
        self
    }

    pub fn first_name(mut self, first_name: Option<String>) -> Self {
        self.first_name = first_name;
        self
    }

    pub fn last_name(mut self, last_name: Option<String>) -> Self {
        self.last_name = last_name;
        self
    }

    pub fn avatar(mut self, avatar: Option<String>) -> Self {
        self.avatar = avatar;
        self
    }

    pub fn name(mut self, name: Option<String>) -> Self {
        self.name = name;
        self
    }

    pub fn anonymous(mut self, anonymous: bool) -> Self {
        self.anonymous = anonymous;
        self
    }

    pub fn custom(mut self, custom: HashMap<String, String>) -> Self {
        self.custom = custom;
        self
    }

    pub fn derived(mut self, derived: HashMap<String, DerivedAttribute>) -> Self {
        self.derived = derived;
        self
    }

    pub fn private_attributes(mut self, private_attributes: Vec<String>) -> Self {
        self.private_attributes = private_attributes;
        self
    }

    pub fn build(self) -> User {
        User { builder: self }
    }
}

pub struct User {
    builder: UserBuilder,
}

pub struct DerivedAttribute {
    value: Value,
    LastDerived: u64,
}

impl User {
    pub fn bucket(&self, key: &str, by: &str, salt: &str) -> f64 {
        if let Some(ref val) = self.get_for_eval(by) {
            let mut source = key.to_string();
            source.push('.');
            source.push_str(salt);
            source.push('.');
            source.push_str(val);

            if let Some(ref scnd) = self.builder.secondary {
                source.push('.');
                source.push_str(scnd);
            }

            let hash = Sha1::digest_str(source.as_str());
            let hex = hash.iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>();
            let val_res = i64::from_str_radix(&hex[..15], 16);

            val_res.ok().map_or(
                0.0,
                |val| val as f64 / LONG_SCALE as f64,
            )
        } else {
            0.0
        }
    }

    fn get_for_eval(&self, key: &str) -> Option<&str> {
        match key {
            "key" => Some(&self.builder.key),
            "ip" => self.builder.ip.as_ref(),
            "country" => self.builder.country.as_ref(),
            "email" => self.builder.email.as_ref(),
            "first_name" => self.builder.first_name.as_ref(),
            "last_name" => self.builder.last_name.as_ref(),
            "avatar" => self.builder.avatar.as_ref(),
            "name" => self.builder.name.as_ref(),
            _ => self.builder.custom.get(key),
        }.map(|ref value| value.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bucket_user() {
        let user_key_a = "userKeyA";
        let user_a = UserBuilder::new(user_key_a).build();
        let bucket_a = user_a.bucket("hashKey", "key", "saltyA");
        assert_eq!(0.4215758743392494, bucket_a);

        let user_key_b = "userKeyB";
        let user_b = UserBuilder::new(user_key_b).build();
        let bucket_b = user_b.bucket("hashKey", "key", "saltyA");
        assert_eq!(0.6708484965703435, bucket_b);

        let user_key_c = "userKeyC";
        let user_c = UserBuilder::new(user_key_c).build();
        let bucket_c = user_c.bucket("hashKey", "key", "saltyA");
        assert_eq!(0.1034310617276969, bucket_c);
    }
}
