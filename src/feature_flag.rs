use clause::Clause;
use events::FeatureRequestEvent;
use store::FeatureStore;
use user::User;

pub type Variation = i64;

pub type FlagResult<T> = Result<T, FlagError>;
pub type FlagError = bool;

pub struct FeatureFlag {
    key: String,
    version: i64,
    on: bool,
    prerequisites: Vec<Prerequisite>,
    salt: String,
    sel: String,
    targets: Vec<Target>,
    rules: Vec<Rule>,
    fallthrough: VariationOrRollOut,
    off_variation: Option<i64>,
    variations: Vec<Variation>,
    deleted: bool,
}

pub struct Prerequisite {
    pub key: Option<String>,
    pub variation: Option<Variation>,
}

pub struct Target {
    pub value: Vec<String>,
    pub variation: Option<Variation>,
}

pub struct Rule {
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

    pub fn matches_user(&self, user: &User) -> bool {
        self.clauses.iter().fold(
            true,
            |pass, c| pass & c.matches_user(user),
        )
    }
}

pub enum VariationOrRollOut {
    Rollout(Rollout),
    Variation(Variation),
}

pub struct Rollout {
    pub weighted_variations: Vec<WeightedVariation>,
    pub bucket_by: Option<String>,
}

pub struct WeightedVariation {
    pub variation: Variation,
    pub weight: i64,
}

pub struct EvalResult {
    pub value: Option<Variation>,
    pub explanation: Explanation,
    pub events: Vec<FeatureRequestEvent>,
}

pub enum Explanation {
    Prerequisite(Prerequisite),
    Rule(Rule),
    Target(Target),
    VariationOrRollOut(VariationOrRollOut),
}

impl Explanation {
    pub fn kind(&self) -> &'static str {
        match *self {
            Explanation::Prerequisite(_) => "prerequisite",
            Explanation::Rule(_) => "rule",
            Explanation::Target(_) => "target",
            Explanation::VariationOrRollOut(_) => "fallthrough",
        }
    }
}

impl FeatureFlag {
    pub fn evalute<S: FeatureStore>(&self, user: &User, store: &S) -> EvalResult {
        let events = vec![];
        self.eval_with_explain(user, store, events)
    }

    fn eval_with_explain<S: FeatureStore>(
        &self,
        user: &User,
        store: &S,
        events: Vec<FeatureRequestEvent>,
    ) -> EvalResult {
        EvalResult {
            value: None,
            explanation: Explanation::Prerequisite(Prerequisite {
                key: None,
                variation: None,
            }),
            events: events,
        }
    }

    pub fn evalute_index() -> Option<Variation> {
        None
    }

    pub fn off_variantion(&self) -> Option<Variation> {
        self.off_variation.and_then(|off| self.variation(off))
    }

    pub fn variation(&self, i: i64) -> Option<Variation> {
        if i < self.variations.len() as i64 {
            self.variations.iter().nth(i as usize).map(|v| *v)
        } else {
            None
        }
    }

    pub fn key(&self) -> &str {
        self.key.as_str()
    }

    pub fn version(&self) -> i64 {
        self.version
    }

    pub fn on(&self) -> bool {
        self.on
    }

    pub fn deleted(&self) -> bool {
        self.deleted
    }
}

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
