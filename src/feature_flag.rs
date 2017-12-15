use clause::Clause;
use events::FeatureRequestEvent;
use store::{FeatureStore, MemStore};
use user::User;

pub type Variation = usize;

pub type FlagResult<T> = Result<T, FlagError>;

#[derive(Debug, PartialEq)]
pub enum FlagError {
    FailedToEvalIndex,
    FailedToSatisfyPrereq,
    InvalidVariationIndex,
}

#[derive(Clone, Debug)]
pub struct FeatureFlag {
    key: String,
    version: usize,
    on: bool,
    prerequisites: Vec<Prerequisite>,
    salt: String,
    sel: String,
    targets: Vec<Target>,
    rules: Vec<Rule>,
    fallthrough: VariationOrRollOut,
    off_variation: Option<usize>,
    variations: Vec<Variation>,
    deleted: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Prerequisite {
    pub key: String,
    pub variation: Variation,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Target {
    pub values: Vec<String>,
    pub variation: Option<Variation>,
}

#[derive(Clone, Debug, PartialEq)]
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
        self.variation_or_rollout.variation_index_for_user(
            user,
            key,
            salt,
        )
    }

    pub fn matches_user(&self, user: &User) -> bool {
        self.clauses.iter().fold(
            true,
            |pass, c| pass & c.matches_user(user),
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum VariationOrRollOut {
    Rollout(Rollout),
    Variation(Variation),
}

impl VariationOrRollOut {
    pub fn variation_index_for_user(
        &self,
        user: &User,
        key: &str,
        salt: &str,
    ) -> Option<Variation> {
        match *self {
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

#[derive(Clone, Debug, PartialEq)]
pub struct Rollout {
    pub weighted_variations: Vec<WeightedVariation>,
    pub bucket_by: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WeightedVariation {
    pub variation: Variation,
    pub weight: usize,
}

#[derive(Debug)]
pub struct Eval<'a> {
    pub result: VariationResult,
    pub events: Vec<FeatureRequestEvent<'a>>,
}

#[derive(Debug)]
pub struct VariationResult {
    pub value: FlagResult<Variation>,
    pub explanation: Explanation,
}

pub struct IndexResult {
    pub value: Option<usize>,
    pub explanation: Explanation,
}

#[derive(Debug, PartialEq)]
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
    pub fn new(
        key: String,
        version: usize,
        on: bool,
        prerequisites: Vec<Prerequisite>,
        salt: String,
        sel: String,
        targets: Vec<Target>,
        rules: Vec<Rule>,
        fallthrough: VariationOrRollOut,
        off_variation: Option<usize>,
        variations: Vec<Variation>,
        deleted: bool,
    ) -> FeatureFlag {
        FeatureFlag {
            key: key,
            version: version,
            on: on,
            prerequisites: prerequisites,
            salt: salt,
            sel: sel,
            targets: targets,
            rules: rules,
            fallthrough: fallthrough,
            off_variation: off_variation,
            variations: variations,
            deleted: deleted,
        }
    }

    pub fn evaluate<S: FeatureStore>(&self, user: &User, store: &S) -> Eval {
        let mut events = vec![];

        Eval {
            result: self.eval(user, store, &mut events),
            events: events,
        }
    }

    fn eval<S: FeatureStore>(
        &self,
        user: &User,
        store: &S,
        events: &mut Vec<FeatureRequestEvent>,
    ) -> VariationResult {
        let mut failed_prereq = None;

        for prereq in self.prerequisites.iter() {
            if failed_prereq.is_none() {
                failed_prereq = if let Some(p_flag) = store.get(prereq.key.as_str()) {
                    if p_flag.on() {
                        let p_flag_eval = p_flag.eval(user, store, events);
                        let p_flag_var = p_flag.variation(prereq.variation);

                        // NOTE: SIDE EFFECT
                        // Add event tracking

                        if let Ok(val) = p_flag_eval.value {
                            if let Ok(var) = p_flag_var {
                                if val == var { None } else { Some(prereq) }
                            } else {
                                Some(prereq)
                            }
                        } else {
                            Some(prereq)
                        }
                    } else {
                        Some(prereq)
                    }
                } else {
                    Some(prereq)
                }
            }
        }

        match failed_prereq {
            Some(failure) => VariationResult {
                value: Err(FlagError::FailedToSatisfyPrereq),
                explanation: Explanation::Prerequisite(failure.clone()),
            },
            None => {
                let index = self.eval_index(user);

                VariationResult {
                    value: index.value.ok_or(FlagError::FailedToEvalIndex).and_then(
                        |value| {
                            self.variation(value)
                        },
                    ),
                    explanation: index.explanation,
                }
            }
        }
    }

    pub fn eval_index(&self, user: &User) -> IndexResult {
        for target in self.targets.iter() {
            for value in target.values.iter() {
                if value == user.key() {
                    return IndexResult {
                        value: target.variation,
                        explanation: Explanation::Target(target.clone()),
                    };
                }
            }
        }

        for rule in self.rules.iter() {
            if rule.matches_user(user) {
                let variation = rule.variation_index_for_user(user, self.key(), self.salt());

                return IndexResult {
                    value: variation,
                    explanation: Explanation::Rule(rule.clone()),
                };
            }
        }

        IndexResult {
            value: self.fallthrough.variation_index_for_user(
                user,
                self.key(),
                self.salt(),
            ),
            explanation: Explanation::VariationOrRollOut(self.fallthrough.clone()),
        }
    }

    pub fn off_variation(&self) -> Option<FlagResult<Variation>> {
        self.off_variation.map(|off| self.variation(off))
    }

    pub fn variation(&self, i: usize) -> FlagResult<Variation> {
        self.variations.iter().nth(i).map(|v| *v).ok_or(
            FlagError::InvalidVariationIndex,
        )
    }

    pub fn key(&self) -> &str {
        self.key.as_str()
    }

    pub fn salt(&self) -> &str {
        self.salt.as_str()
    }

    pub fn version(&self) -> usize {
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
    use store::*;
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

    fn flag_with_prereq(a: String, b: String) -> FeatureFlag {
        FeatureFlag::new(
            a,
            0,
            true,
            vec![
                Prerequisite {
                    key: b,
                    variation: 0,
                },
            ],
            "".into(),
            "".into(),
            vec![],
            vec![],
            VariationOrRollOut::Variation(0),
            None,
            vec![0, 1],
            false,
        )
    }

    fn flag_off(a: String) -> FeatureFlag {
        FeatureFlag::new(
            a,
            0,
            false,
            vec![],
            "".into(),
            "".into(),
            vec![],
            vec![],
            VariationOrRollOut::Variation(0),
            None,
            vec![0, 1],
            false,
        )
    }

    #[test]
    fn test_prereq_does_not_exist() {
        let f1 = flag_with_prereq("keyA".into(), "keyB".into());
        let store = MemStore::new();
        let user = UserBuilder::new("userKey").build();

        store.upsert(f1.key(), &f1);

        let eval = f1.evaluate(&user, &store);
        let explanation = Explanation::Prerequisite(Prerequisite {
            key: "keyB".into(),
            variation: 0,
        });

        assert_eq!(eval.result.value, Err(FlagError::FailedToSatisfyPrereq));
        assert_eq!(eval.result.explanation, explanation);
        assert_eq!(eval.events.len(), 0);
    }

    // @Test
    // public void testPrereqCollectsEventsForPrereqs() throws EvaluationException {
    // String keyA = "keyA";
    // String keyB = "keyB";
    // String keyC = "keyC";
    // FeatureFlag flagA = newFlagWithPrereq(keyA, keyB);
    // FeatureFlag flagB = newFlagWithPrereq(keyB, keyC);
    // FeatureFlag flagC = newFlagOff(keyC);
    //
    // featureStore.upsert(flagA.getKey(), flagA);
    // featureStore.upsert(flagB.getKey(), flagB);
    // featureStore.upsert(flagC.getKey(), flagC);
    //
    // LDUser user = new LDUser.Builder("userKey").build();
    //
    // FeatureFlag.EvalResult flagAResult = flagA.evaluate(user, featureStore);
    // Assert.assertNotNull(flagAResult);
    // Assert.assertNull(flagAResult.getValue());
    // Assert.assertEquals(2, flagAResult.getPrerequisiteEvents().size());
    //
    // FeatureFlag.EvalResult flagBResult = flagB.evaluate(user, featureStore);
    // Assert.assertNotNull(flagBResult);
    // Assert.assertNull(flagBResult.getValue());
    // Assert.assertEquals(1, flagBResult.getPrerequisiteEvents().size());
    //
    // FeatureFlag.EvalResult flagCResult = flagC.evaluate(user, featureStore);
    // Assert.assertNotNull(flagCResult);
    // Assert.assertEquals(null, flagCResult.getValue());
    // Assert.assertEquals(0, flagCResult.getPrerequisiteEvents().size());
    // }

    #[test]
    fn test_prereq_collects_events() {
        let f1 = flag_with_prereq("key1".into(), "key2".into());
        let f2 = flag_with_prereq("key2".into(), "key3".into());
        let f3 = flag_off("key3".into());
        let store = MemStore::new();
        let user = UserBuilder::new("userKey").build();

        store.upsert(f1.key(), &f1);
        store.upsert(f2.key(), &f3);
        store.upsert(f3.key(), &f3);

        let f1_eval = f1.evaluate(&user, &store);

        panic!("{:?}", f1_eval);
    }
}
