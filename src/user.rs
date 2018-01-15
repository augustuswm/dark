use sha1::{Digest, Sha1};

use std::collections::HashMap;

use feature_flag::VariationValue;

static LONG_SCALE: u64 = 0xFFFFFFFFFFFFFFF;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserBuilder {
    user: User,
}

impl UserBuilder {
    pub fn new<S: Into<String>>(key: S) -> UserBuilder {
        UserBuilder {
            user: User {
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
            },
        }
    }

    pub fn secondary(mut self, secondary: Option<String>) -> Self {
        self.user.secondary = secondary;
        self
    }

    pub fn ip(mut self, ip: Option<String>) -> Self {
        self.user.ip = ip;
        self
    }

    pub fn country(mut self, country: Option<String>) -> Self {
        self.user.country = country;
        self
    }

    pub fn email(mut self, email: Option<String>) -> Self {
        self.user.email = email;
        self
    }

    pub fn first_name(mut self, first_name: Option<String>) -> Self {
        self.user.first_name = first_name;
        self
    }

    pub fn last_name(mut self, last_name: Option<String>) -> Self {
        self.user.last_name = last_name;
        self
    }

    pub fn avatar(mut self, avatar: Option<String>) -> Self {
        self.user.avatar = avatar;
        self
    }

    pub fn name(mut self, name: Option<String>) -> Self {
        self.user.name = name;
        self
    }

    pub fn anonymous(mut self, anonymous: bool) -> Self {
        self.user.anonymous = anonymous;
        self
    }

    pub fn custom(mut self, custom: HashMap<String, String>) -> Self {
        self.user.custom = custom;
        self
    }

    pub fn derived(mut self, derived: HashMap<String, DerivedAttribute>) -> Self {
        self.user.derived = derived;
        self
    }

    pub fn private_attributes(mut self, private_attributes: Vec<String>) -> Self {
        self.user.private_attributes = private_attributes;
        self
    }

    pub fn build(self) -> User {
        self.user
    }
}

// TODO: Refactor into reverse impl. See config

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    secondary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    anonymous: bool,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    custom: HashMap<String, String>,
    #[serde(skip_serializing)]
    derived: HashMap<String, DerivedAttribute>,
    #[serde(skip_serializing)]
    private_attributes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DerivedAttribute {
    value: VariationValue,
    last_derived: u64,
}

impl User {
    pub fn key(&self) -> &str {
        self.key.as_str()
    }

    pub fn bucket(&self, key: &str, by: &str, salt: &str) -> f64 {
        if let Some(ref val) = self.get_for_eval(by) {
            let mut source = [key, salt, val].join(".");

            if let Some(ref scnd) = self.secondary {
                source.push('.');
                source.push_str(scnd);
            }

            let hash = Sha1::digest_str(source.as_str());
            let hex = hash.iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>();
            let val_res = i64::from_str_radix(&hex[..15], 16);

            val_res
                .ok()
                .map_or(0.0, |val| val as f64 / LONG_SCALE as f64)
        } else {
            0.0
        }
    }

    pub fn get_for_eval(&self, key: &str) -> Option<&str> {
        match key {
            "key" => Some(&self.key),
            "ip" => self.ip.as_ref(),
            "country" => self.country.as_ref(),
            "email" => self.email.as_ref(),
            "first_name" => self.first_name.as_ref(),
            "last_name" => self.last_name.as_ref(),
            "avatar" => self.avatar.as_ref(),
            "name" => self.name.as_ref(),
            _ => self.custom.get(key),
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
