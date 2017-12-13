use clause::Clause;
use user::User;

type Variation = i64;

struct FeatureFlag {
    key: String,
    version: Variation,
    on: bool,
    prerequisites: Vec<Prerequisite>,
    salt: String,
    sel: String,
    targets: Vec<Target>,
    rules: Vec<Rule>,
    fallthrough: VariationOrRollOut,
    off_variation: Variation,
    variations: Vec<Variation>,
    deleted: bool,
}

struct Prerequisite {
    pub key: Option<String>,
    pub variation: Option<Variation>,
}

struct Target {
    pub value: Vec<String>,
    pub variation: Option<Variation>,
}

struct Rule {
    variation_or_rollout: VariationOrRollOut,
    pub clauses: Vec<Clause>,
}

impl Rule {
    pub fn variation_index_for_user(
        &self,
        user: &User,
        key: &str,
        salt: &str,
    ) -> Option<Variation> {
        match self.variation_or_rollout {
            VariationOrRollOut::Rollout(ref rollout) => {
                if rollout.weighted_variations.len() == 0 {
                    None
                } else {
                    let by = rollout.bucket_by.as_ref().map_or("key", |v| v.as_str());
                    let bucket = user.bucket(key, by, salt);

                    let mut sum: f64 = 0.0;

                    for weighted_var in &rollout.weighted_variations {
                        sum = sum + weighted_var.weight as f64 / 100000.0;

                        if bucket < sum {
                            return Some(weighted_var.variation);
                        }
                    }

                    None
                }
            }
            VariationOrRollOut::Variation(variation) => Some(variation),
        }
    }
}

enum VariationOrRollOut {
    Rollout(Rollout),
    Variation(Variation),
}

struct Rollout {
    pub weighted_variations: Vec<WeightedVariation>,
    pub bucket_by: Option<String>,
}

struct WeightedVariation {
    pub variation: Variation,
    pub weight: i64,
}

impl FeatureFlag {}

#[cfg(test)]
mod tests {
    use feature_flag::*;
    use user::*;

    #[test]
    fn test_variation_index_for_user() {
        let wv1 = WeightedVariation {
            variation: 0,
            weight: 60000,
        };
        let wv2 = WeightedVariation {
            variation: 1,
            weight: 40000,
        };
        let rollout = Rollout {
            weighted_variations: vec![wv1, wv2],
            bucket_by: None,
        };
        let rule = Rule {
            variation_or_rollout: VariationOrRollOut::Rollout(rollout),
            clauses: vec![],
        };

        let user_key_a = "userKeyA";
        let user_a = UserBuilder::new(user_key_a).build();
        let v_1 = rule.variation_index_for_user(&user_a, "hashKey", "saltyA");
        assert!(v_1.is_some(), "Variation 1 should not be None");
        assert_eq!(0, v_1.unwrap());

        let user_key_b = "userKeyB";
        let user_b = UserBuilder::new(user_key_b).build();
        let v_2 = rule.variation_index_for_user(&user_b, "hashKey", "saltyA");
        assert!(v_2.is_some(), "Variation 2 should not be None");
        assert_eq!(1, v_2.unwrap());

        let user_key_c = "userKeyC";
        let user_c = UserBuilder::new(user_key_c).build();
        let v_3 = rule.variation_index_for_user(&user_c, "hashKey", "saltyA");
        assert!(v_3.is_some(), "Variation 3 should not be None");
        assert_eq!(0, v_3.unwrap());
    }
}
