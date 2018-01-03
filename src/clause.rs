use chrono::{DateTime, NaiveDateTime, Utc};
use regex::Regex;

use feature_flag::VariationValue;
use user::User;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Clause {
    attribute: String,
    op: Operator,
    values: Vec<VariationValue>,
    negate: bool,
}

impl Clause {
    pub fn matches_user(&self, user: &User) -> bool {
        if let Some(val) = user.get_for_eval(self.attribute.as_str()) {

            // TODO: Add handling for non-string values coming from user data
            self.handle_negate(self.match_any(VariationValue::String(val.into())))
        } else {
            false
        }
    }

    pub fn match_any(&self, val: VariationValue) -> bool {
        self.values.iter().fold(false, |pass, v| {
            pass || self.op.apply(v, &val)
        })
    }

    fn handle_negate(&self, status: bool) -> bool {
        if self.negate { !status } else { status }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Operator {
    In,
    EndsWith,
    StartsWith,
    Matches,
    Contains,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Before,
    After,
}

impl Operator {
    pub fn apply(&self, a: &VariationValue, b: &VariationValue) -> bool {
        match *self {
            Operator::In => {
                match (a, b) {
                    (&VariationValue::Integer(ref a_v), &VariationValue::Float(ref b_v)) => {
                        (*a_v as f64) == *b_v as f64
                    }
                    (&VariationValue::Float(ref a_v), &VariationValue::Integer(ref b_v)) => {
                        (*a_v as f64) == *b_v as f64
                    }
                    (ref a, ref b) => a == b,
                }
            }
            Operator::StartsWith => {
                match (a, b) {
                    (&VariationValue::String(ref a_v), &VariationValue::String(ref b_v)) => {
                        a_v.as_str().starts_with(b_v.as_str())
                    }
                    _ => false,
                }
            }
            Operator::EndsWith => {
                match (a, b) {
                    (&VariationValue::String(ref a_v), &VariationValue::String(ref b_v)) => {
                        a_v.as_str().ends_with(b_v.as_str())
                    }
                    _ => false,
                }
            }
            Operator::Matches => {
                match (a, b) {
                    (&VariationValue::String(ref a_v), &VariationValue::String(ref b_v)) => {
                        if let Ok(re) = Regex::new(b_v) {
                            re.is_match(a_v)
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            }
            Operator::Contains => {
                match (a, b) {
                    (&VariationValue::String(ref a_v), &VariationValue::String(ref b_v)) => {
                        a_v.as_str().contains(b_v.as_str())
                    }
                    _ => false,
                }
            }
            Operator::LessThan => {
                match (a, b) {
                    (&VariationValue::Integer(ref a_v), &VariationValue::Float(ref b_v)) => {
                        (*a_v as f64) < *b_v as f64
                    }
                    (&VariationValue::Float(ref a_v), &VariationValue::Integer(ref b_v)) => {
                        (*a_v as f64) < *b_v as f64
                    }
                    (&VariationValue::Integer(ref a_v), &VariationValue::Integer(ref b_v)) => {
                        a_v < b_v
                    }
                    (&VariationValue::Float(ref a_v), &VariationValue::Float(ref b_v)) => a_v < b_v,
                    _ => false,
                }
            }
            Operator::LessThanOrEqual => {
                match (a, b) {
                    (&VariationValue::Integer(ref a_v), &VariationValue::Float(ref b_v)) => {
                        (*a_v as f64) <= *b_v as f64
                    }
                    (&VariationValue::Float(ref a_v), &VariationValue::Integer(ref b_v)) => {
                        (*a_v as f64) <= *b_v as f64
                    }
                    (&VariationValue::Integer(ref a_v), &VariationValue::Integer(ref b_v)) => {
                        a_v <= b_v
                    }
                    (&VariationValue::Float(ref a_v), &VariationValue::Float(ref b_v)) => {
                        a_v <= b_v
                    }
                    _ => false,
                }
            }
            Operator::GreaterThan => {
                match (a, b) {
                    (&VariationValue::Integer(ref a_v), &VariationValue::Float(ref b_v)) => {
                        (*a_v as f64) > *b_v as f64
                    }
                    (&VariationValue::Float(ref a_v), &VariationValue::Integer(ref b_v)) => {
                        (*a_v as f64) > *b_v as f64
                    }
                    (&VariationValue::Integer(ref a_v), &VariationValue::Integer(ref b_v)) => {
                        a_v > b_v
                    }
                    (&VariationValue::Float(ref a_v), &VariationValue::Float(ref b_v)) => a_v > b_v,
                    _ => false,
                }
            }
            Operator::GreaterThanOrEqual => {
                match (a, b) {
                    (&VariationValue::Integer(ref a_v), &VariationValue::Float(ref b_v)) => {
                        (*a_v as f64) >= *b_v as f64
                    }
                    (&VariationValue::Float(ref a_v), &VariationValue::Integer(ref b_v)) => {
                        (*a_v as f64) >= *b_v as f64
                    }
                    (&VariationValue::Integer(ref a_v), &VariationValue::Integer(ref b_v)) => {
                        a_v >= b_v
                    }
                    (&VariationValue::Float(ref a_v), &VariationValue::Float(ref b_v)) => {
                        a_v >= b_v
                    }
                    _ => false,
                }
            }
            Operator::Before => {
                match (value_to_time(a), value_to_time(b)) {
                    (Some(date_a), Some(date_b)) => date_a < date_b,
                    _ => false,
                }
            }
            Operator::After => {
                match (value_to_time(a), value_to_time(b)) {
                    (Some(date_a), Some(date_b)) => date_a > date_b,
                    _ => false,
                }
            }
        }
    }
}

// #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
// pub enum VariationValue {
//     String(String),
//     Float(f64),
//     Integer(i64),
//     Boolean(bool),
// }

impl VariationValue {
    pub fn is_numeric(&self) -> bool {
        match *self {
            VariationValue::Integer(_) |
            VariationValue::Float(_) => true,
            _ => false,
        }
    }
}

fn f_to_time(f: f64) -> DateTime<Utc> {
    let sec = (f / 1000.0).trunc();
    let remain = (f / 1000.0) - sec;
    let nano = (remain * 1000.0 * 1000000.0).trunc();

    DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(sec as i64, nano as u32), Utc)
}

fn value_to_time(v: &VariationValue) -> Option<DateTime<Utc>> {
    match *v {
        VariationValue::String(ref v_v) => {
            if let Ok(date) = v_v.parse::<DateTime<Utc>>() {
                Some(date)
            } else {
                if let Ok(v_f) = v_v.parse::<f64>() {
                    Some(f_to_time(v_f))
                } else {
                    None
                }
            }
        }
        VariationValue::Integer(ref v_v) => Some(f_to_time(*v_v as f64)),
        VariationValue::Float(ref v_v) => Some(f_to_time(*v_v)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use clause::{Operator, VariationValue};

    #[test]
    fn test_op_in() {
        assert!(Operator::In.apply(
            &VariationValue::String(
                "A string to match".into(),
            ),
            &VariationValue::String(
                "A string to match".into(),
            ),
        ));
        assert!(!Operator::In.apply(
            &VariationValue::String(
                "A string to match".into(),
            ),
            &VariationValue::Boolean(true),
        ));
        assert!(Operator::In.apply(
            &VariationValue::Integer(34),
            &VariationValue::Integer(34),
        ));
        assert!(Operator::In.apply(
            &VariationValue::Integer(34),
            &VariationValue::Float(34.0),
        ));
        assert!(!Operator::In.apply(
            &VariationValue::Integer(34),
            &VariationValue::Boolean(true),
        ));
        assert!(Operator::In.apply(
            &VariationValue::Boolean(false),
            &VariationValue::Boolean(false),
        ));
        assert!(Operator::In.apply(
            &VariationValue::Boolean(true),
            &VariationValue::Boolean(true),
        ));
        assert!(!Operator::In.apply(
            &VariationValue::Boolean(true),
            &VariationValue::Boolean(false),
        ));
        assert!(!Operator::In.apply(
            &VariationValue::Boolean(false),
            &VariationValue::Boolean(true),
        ));
    }

    #[test]
    fn test_op_ends_with() {
        assert!(Operator::EndsWith.apply(
            &VariationValue::String("end".into()),
            &VariationValue::String("end".into()),
        ));
        assert!(Operator::EndsWith.apply(
            &VariationValue::String(
                "plus more end".into(),
            ),
            &VariationValue::String("end".into()),
        ));
        assert!(!Operator::EndsWith.apply(
            &VariationValue::String(
                "does not contain".into(),
            ),
            &VariationValue::String("end".into()),
        ));
        assert!(!Operator::EndsWith.apply(
            &VariationValue::String(
                "does not end with".into(),
            ),
            &VariationValue::String("end".into()),
        ));
        assert!(!Operator::EndsWith.apply(
            &VariationValue::String("end".into()),
            &VariationValue::Float(0.0),
        ));
        assert!(!Operator::EndsWith.apply(
            &VariationValue::String("end".into()),
            &VariationValue::Integer(0),
        ));
        assert!(!Operator::EndsWith.apply(
            &VariationValue::String("end".into()),
            &VariationValue::Boolean(true),
        ));
        assert!(!Operator::EndsWith.apply(
            &VariationValue::Float(0.0),
            &VariationValue::String("end".into()),
        ));
        assert!(!Operator::EndsWith.apply(
            &VariationValue::Integer(0),
            &VariationValue::String("end".into()),
        ));
        assert!(!Operator::EndsWith.apply(
            &VariationValue::Boolean(true),
            &VariationValue::String("end".into()),
        ));
    }

    #[test]
    fn test_op_starts_with() {
        assert!(Operator::StartsWith.apply(
            &VariationValue::String("start".into()),
            &VariationValue::String("start".into()),
        ));
        assert!(Operator::StartsWith.apply(
            &VariationValue::String(
                "start plus more".into(),
            ),
            &VariationValue::String("start".into()),
        ));
        assert!(!Operator::StartsWith.apply(
            &VariationValue::String(
                "does not contain".into(),
            ),
            &VariationValue::String("start".into()),
        ));
        assert!(!Operator::StartsWith.apply(
            &VariationValue::String(
                "does not start with".into(),
            ),
            &VariationValue::String("start".into()),
        ));
        assert!(!Operator::StartsWith.apply(
            &VariationValue::String("start".into()),
            &VariationValue::Float(0.0),
        ));
        assert!(!Operator::StartsWith.apply(
            &VariationValue::String("start".into()),
            &VariationValue::Integer(0),
        ));
        assert!(!Operator::StartsWith.apply(
            &VariationValue::String("start".into()),
            &VariationValue::Boolean(true),
        ));
        assert!(!Operator::StartsWith.apply(
            &VariationValue::Float(0.0),
            &VariationValue::String("start".into()),
        ));
        assert!(!Operator::StartsWith.apply(
            &VariationValue::Integer(0),
            &VariationValue::String("start".into()),
        ));
        assert!(!Operator::StartsWith.apply(
            &VariationValue::Boolean(true),
            &VariationValue::String("start".into()),
        ));
    }

    #[test]
    fn test_op_matches() {
        assert!(Operator::Matches.apply(
            &VariationValue::String("anything".into()),
            &VariationValue::String(".*".into()),
        ));
        assert!(Operator::Matches.apply(
            &VariationValue::String("darn".into()),
            &VariationValue::String(
                "(\\W|^)(baloney|darn|drat|fooey|gosh\\sdarnit|heck)(\\W|$)".into(),
            ),
        ));
        assert!(!Operator::Matches.apply(
            &VariationValue::String("barn".into()),
            &VariationValue::String(
                "(\\W|^)(baloney|darn|drat|fooey|gosh\\sdarnit|heck)(\\W|$)"
                    .into(),
            ),
        ));
    }

    #[test]
    fn test_op_contains() {
        assert!(Operator::Contains.apply(
            &VariationValue::String("contain".into()),
            &VariationValue::String("contain".into()),
        ));
        assert!(Operator::Contains.apply(
            &VariationValue::String(
                "contain plus more".into(),
            ),
            &VariationValue::String("contain".into()),
        ));
        assert!(!Operator::Contains.apply(
            &VariationValue::String(
                "does not".into(),
            ),
            &VariationValue::String("contain".into()),
        ));
        assert!(!Operator::Contains.apply(
            &VariationValue::String("contain".into()),
            &VariationValue::Float(0.0),
        ));
        assert!(!Operator::Contains.apply(
            &VariationValue::String("contain".into()),
            &VariationValue::Integer(0),
        ));
        assert!(!Operator::Contains.apply(
            &VariationValue::String("contain".into()),
            &VariationValue::Boolean(true),
        ));
        assert!(!Operator::Contains.apply(
            &VariationValue::Float(0.0),
            &VariationValue::String("contain".into()),
        ));
        assert!(!Operator::Contains.apply(
            &VariationValue::Integer(0),
            &VariationValue::String("contain".into()),
        ));
        assert!(!Operator::Contains.apply(
            &VariationValue::Boolean(true),
            &VariationValue::String("contain".into()),
        ));
    }

    #[test]
    fn test_op_less_than() {
        let tests = vec![
            (VariationValue::Integer(0), VariationValue::Integer(1), true),
            (
                VariationValue::Integer(1),
                VariationValue::Integer(0),
                false
            ),
            (
                VariationValue::Integer(0),
                VariationValue::Integer(0),
                false
            ),
            (VariationValue::Float(0.0), VariationValue::Float(1.0), true),
            (
                VariationValue::Float(1.0),
                VariationValue::Float(0.0),
                false
            ),
            (
                VariationValue::Float(0.0),
                VariationValue::Float(0.0),
                false
            ),
            (VariationValue::Integer(0), VariationValue::Float(1.0), true),
            (
                VariationValue::Integer(1),
                VariationValue::Float(0.0),
                false
            ),
            (
                VariationValue::Integer(0),
                VariationValue::Float(0.0),
                false
            ),
            (VariationValue::Float(0.0), VariationValue::Integer(1), true),
            (
                VariationValue::Float(1.0),
                VariationValue::Integer(0),
                false
            ),
            (
                VariationValue::Float(0.0),
                VariationValue::Integer(0),
                false
            ),
            (
                VariationValue::String("".into()),
                VariationValue::Integer(0),
                false
            ),
            (
                VariationValue::Integer(0),
                VariationValue::String("".into()),
                false
            ),
            (
                VariationValue::Boolean(true),
                VariationValue::Integer(1),
                false
            ),
            (
                VariationValue::Integer(1),
                VariationValue::Boolean(true),
                false
            ),
        ];

        for (a, b, res) in tests {
            assert_eq!(Operator::LessThan.apply(&a, &b), res, "{:?} > {:?}", a, b);
        }
    }

    #[test]
    fn test_op_less_than_or_equal_to() {
        let tests = vec![
            (VariationValue::Integer(0), VariationValue::Integer(1), true),
            (
                VariationValue::Integer(1),
                VariationValue::Integer(0),
                false
            ),
            (VariationValue::Integer(0), VariationValue::Integer(0), true),
            (VariationValue::Float(0.0), VariationValue::Float(1.0), true),
            (
                VariationValue::Float(1.0),
                VariationValue::Float(0.0),
                false
            ),
            (VariationValue::Float(0.0), VariationValue::Float(0.0), true),
            (VariationValue::Integer(0), VariationValue::Float(1.0), true),
            (
                VariationValue::Integer(1),
                VariationValue::Float(0.0),
                false
            ),
            (VariationValue::Integer(0), VariationValue::Float(0.0), true),
            (VariationValue::Float(0.0), VariationValue::Integer(1), true),
            (
                VariationValue::Float(1.0),
                VariationValue::Integer(0),
                false
            ),
            (VariationValue::Float(0.0), VariationValue::Integer(0), true),
            (
                VariationValue::String("".into()),
                VariationValue::Integer(0),
                false
            ),
            (
                VariationValue::Integer(0),
                VariationValue::String("".into()),
                false
            ),
            (
                VariationValue::Boolean(true),
                VariationValue::Integer(1),
                false
            ),
            (
                VariationValue::Integer(1),
                VariationValue::Boolean(true),
                false
            ),
        ];

        for (a, b, res) in tests {
            assert_eq!(
                Operator::LessThanOrEqual.apply(&a, &b),
                res,
                "{:?} > {:?}",
                a,
                b
            );
        }
    }

    #[test]
    fn test_op_greater_than() {
        let tests = vec![
            (
                VariationValue::Integer(0),
                VariationValue::Integer(1),
                false
            ),
            (VariationValue::Integer(1), VariationValue::Integer(0), true),
            (
                VariationValue::Integer(0),
                VariationValue::Integer(0),
                false
            ),
            (
                VariationValue::Float(0.0),
                VariationValue::Float(1.0),
                false
            ),
            (VariationValue::Float(1.0), VariationValue::Float(0.0), true),
            (
                VariationValue::Float(0.0),
                VariationValue::Float(0.0),
                false
            ),
            (
                VariationValue::Integer(0),
                VariationValue::Float(1.0),
                false
            ),
            (VariationValue::Integer(1), VariationValue::Float(0.0), true),
            (
                VariationValue::Integer(0),
                VariationValue::Float(0.0),
                false
            ),
            (
                VariationValue::Float(0.0),
                VariationValue::Integer(1),
                false
            ),
            (VariationValue::Float(1.0), VariationValue::Integer(0), true),
            (
                VariationValue::Float(0.0),
                VariationValue::Integer(0),
                false
            ),
            (
                VariationValue::String("".into()),
                VariationValue::Integer(0),
                false
            ),
            (
                VariationValue::Integer(0),
                VariationValue::String("".into()),
                false
            ),
            (
                VariationValue::Boolean(true),
                VariationValue::Integer(1),
                false
            ),
            (
                VariationValue::Integer(1),
                VariationValue::Boolean(true),
                false
            ),
        ];

        for (a, b, res) in tests {
            assert_eq!(
                Operator::GreaterThan.apply(&a, &b),
                res,
                "{:?} > {:?}",
                a,
                b
            );
        }
    }

    #[test]
    fn test_op_greater_than_or_equal_to() {
        let tests = vec![
            (
                VariationValue::Integer(0),
                VariationValue::Integer(1),
                false
            ),
            (VariationValue::Integer(1), VariationValue::Integer(0), true),
            (VariationValue::Integer(0), VariationValue::Integer(0), true),
            (
                VariationValue::Float(0.0),
                VariationValue::Float(1.0),
                false
            ),
            (VariationValue::Float(1.0), VariationValue::Float(0.0), true),
            (VariationValue::Float(0.0), VariationValue::Float(0.0), true),
            (
                VariationValue::Integer(0),
                VariationValue::Float(1.0),
                false
            ),
            (VariationValue::Integer(1), VariationValue::Float(0.0), true),
            (VariationValue::Integer(0), VariationValue::Float(0.0), true),
            (
                VariationValue::Float(0.0),
                VariationValue::Integer(1),
                false
            ),
            (VariationValue::Float(1.0), VariationValue::Integer(0), true),
            (VariationValue::Float(0.0), VariationValue::Integer(0), true),
            (
                VariationValue::String("".into()),
                VariationValue::Integer(0),
                false
            ),
            (
                VariationValue::Integer(0),
                VariationValue::String("".into()),
                false
            ),
            (
                VariationValue::Boolean(true),
                VariationValue::Integer(1),
                false
            ),
            (
                VariationValue::Integer(1),
                VariationValue::Boolean(true),
                false
            ),
        ];

        for (a, b, res) in tests {
            assert_eq!(
                Operator::GreaterThanOrEqual.apply(&a, &b),
                res,
                "{:?} > {:?}",
                a,
                b
            );
        }
    }

    #[test]
    fn test_op_before() {
        let tests = vec![
            (VariationValue::Integer(0), VariationValue::Integer(1), true),
            (
                VariationValue::Integer(1),
                VariationValue::Integer(0),
                false
            ),
            (
                VariationValue::Integer(1),
                VariationValue::Integer(1),
                false
            ),
            (
                VariationValue::String("0".into()),
                VariationValue::String("1".into()),
                true
            ),
            (
                VariationValue::String("1".into()),
                VariationValue::String("0".into()),
                false
            ),
            (
                VariationValue::String("1".into()),
                VariationValue::String("1".into()),
                false
            ),
            (VariationValue::Float(0.0), VariationValue::Float(1.0), true),
            (
                VariationValue::Float(1.0),
                VariationValue::Float(0.0),
                false
            ),
            (
                VariationValue::Float(1.0),
                VariationValue::Float(1.0),
                false
            ),
            (
                VariationValue::String("1970-01-01T00:00:01Z".into()),
                VariationValue::String("1970-01-01T00:00:02Z".into()),
                true
            ),
            (
                VariationValue::String("1970-01-01T00:00:01Z".into()),
                VariationValue::String("1970-01-01T00:00:01.0001Z".into()),
                true
            ),
            (
                VariationValue::Integer(0),
                VariationValue::String("1970-01-01T00:00:00.0001Z".into()),
                true
            ),
            (
                VariationValue::Float(0.0),
                VariationValue::String("1970-01-01T00:00:00.0001Z".into()),
                true
            ),
            (
                VariationValue::Integer(0),
                VariationValue::String("1970-01-01----00:00:00.0001Z".into()),
                false
            ),
        ];

        for (a, b, res) in tests {
            assert_eq!(
                Operator::Before.apply(&a, &b),
                res,
                "{:?} is before {:?}",
                a,
                b
            );
        }
    }

    #[test]
    fn test_op_after() {
        let tests = vec![
            (
                VariationValue::Integer(0),
                VariationValue::Integer(1),
                false
            ),
            (VariationValue::Integer(1), VariationValue::Integer(0), true),
            (
                VariationValue::Integer(1),
                VariationValue::Integer(1),
                false
            ),
            (
                VariationValue::String("0".into()),
                VariationValue::String("1".into()),
                false
            ),
            (
                VariationValue::String("1".into()),
                VariationValue::String("0".into()),
                true
            ),
            (
                VariationValue::String("1".into()),
                VariationValue::String("1".into()),
                false
            ),
            (
                VariationValue::Float(0.0),
                VariationValue::Float(1.0),
                false
            ),
            (VariationValue::Float(1.0), VariationValue::Float(0.0), true),
            (
                VariationValue::Float(1.0),
                VariationValue::Float(1.0),
                false
            ),
            (
                VariationValue::String("1970-01-01T00:00:01Z".into()),
                VariationValue::String("1970-01-01T00:00:02Z".into()),
                false
            ),
            (
                VariationValue::String("1970-01-01T00:00:01Z".into()),
                VariationValue::String("1970-01-01T00:00:01.0001Z".into()),
                false
            ),
            (
                VariationValue::Integer(0),
                VariationValue::String("1970-01-01T00:00:00.0001Z".into()),
                false
            ),
            (
                VariationValue::Float(0.0),
                VariationValue::String("1970-01-01T00:00:00.0001Z".into()),
                false
            ),
            (
                VariationValue::Integer(0),
                VariationValue::String("1970-01-01----00:00:00.0001Z".into()),
                false
            ),
        ];

        for (a, b, res) in tests {
            assert_eq!(
                Operator::After.apply(&a, &b),
                res,
                "{:?} is after {:?}",
                a,
                b
            );
        }
    }
}
