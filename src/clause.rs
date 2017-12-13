extern crate chrono;
extern crate regex;

use self::chrono::{DateTime, NaiveDateTime, Utc};
use self::regex::Regex;

use user::User;

pub struct Clause {
    attribute: String,
    op: Operator,
    values: Vec<Value>,
    negate: bool,
}

impl Clause {
    pub fn matches_user(&self, user: &User) -> bool {
        if let Some(val) = user.get_for_eval(self.attribute.as_str()) {

            // TODO: Add handling for non-string values coming from user data
            self.handle_negate(self.match_any(Value::String(val.into())))
        } else {
            false
        }
    }

    pub fn match_any(&self, val: Value) -> bool {
        self.values.iter().fold(false, |pass, v| {
            pass || self.op.apply(v, &val)
        })
    }

    fn handle_negate(&self, status: bool) -> bool {
        if self.negate { !status } else { status }
    }
}

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
    pub fn apply(&self, a: &Value, b: &Value) -> bool {
        match *self {
            Operator::In => {
                match (a, b) {
                    (&Value::Int(ref a_v), &Value::Float(ref b_v)) => (*a_v as f64) == *b_v as f64,
                    (&Value::Float(ref a_v), &Value::Int(ref b_v)) => (*a_v as f64) == *b_v as f64,
                    (ref a, ref b) => a == b,
                }
            }
            Operator::StartsWith => {
                match (a, b) {
                    (&Value::String(ref a_v), &Value::String(ref b_v)) => {
                        a_v.as_str().starts_with(b_v.as_str())
                    }
                    _ => false,
                }
            }
            Operator::EndsWith => {
                match (a, b) {
                    (&Value::String(ref a_v), &Value::String(ref b_v)) => {
                        a_v.as_str().ends_with(b_v.as_str())
                    }
                    _ => false,
                }
            }
            Operator::Matches => {
                match (a, b) {
                    (&Value::String(ref a_v), &Value::String(ref b_v)) => {
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
                    (&Value::String(ref a_v), &Value::String(ref b_v)) => {
                        a_v.as_str().contains(b_v.as_str())
                    }
                    _ => false,
                }
            }
            Operator::LessThan => {
                match (a, b) {
                    (&Value::Int(ref a_v), &Value::Float(ref b_v)) => (*a_v as f64) < *b_v as f64,
                    (&Value::Float(ref a_v), &Value::Int(ref b_v)) => (*a_v as f64) < *b_v as f64,
                    (&Value::Int(ref a_v), &Value::Int(ref b_v)) => a_v < b_v,
                    (&Value::Float(ref a_v), &Value::Float(ref b_v)) => a_v < b_v,
                    _ => false,
                }
            }
            Operator::LessThanOrEqual => {
                match (a, b) {
                    (&Value::Int(ref a_v), &Value::Float(ref b_v)) => (*a_v as f64) <= *b_v as f64,
                    (&Value::Float(ref a_v), &Value::Int(ref b_v)) => (*a_v as f64) <= *b_v as f64,
                    (&Value::Int(ref a_v), &Value::Int(ref b_v)) => a_v <= b_v,
                    (&Value::Float(ref a_v), &Value::Float(ref b_v)) => a_v <= b_v,
                    _ => false,
                }
            }
            Operator::GreaterThan => {
                match (a, b) {
                    (&Value::Int(ref a_v), &Value::Float(ref b_v)) => (*a_v as f64) > *b_v as f64,
                    (&Value::Float(ref a_v), &Value::Int(ref b_v)) => (*a_v as f64) > *b_v as f64,
                    (&Value::Int(ref a_v), &Value::Int(ref b_v)) => a_v > b_v,
                    (&Value::Float(ref a_v), &Value::Float(ref b_v)) => a_v > b_v,
                    _ => false,
                }
            }
            Operator::GreaterThanOrEqual => {
                match (a, b) {
                    (&Value::Int(ref a_v), &Value::Float(ref b_v)) => (*a_v as f64) >= *b_v as f64,
                    (&Value::Float(ref a_v), &Value::Int(ref b_v)) => (*a_v as f64) >= *b_v as f64,
                    (&Value::Int(ref a_v), &Value::Int(ref b_v)) => a_v >= b_v,
                    (&Value::Float(ref a_v), &Value::Float(ref b_v)) => a_v >= b_v,
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

#[derive(Debug, PartialEq)]
pub enum Value {
    String(String),
    Float(f64),
    Int(i64),
    Boolean(bool),
}

impl Value {
    pub fn is_numeric(&self) -> bool {
        match *self {
            Value::Int(_) | Value::Float(_) => true,
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

fn value_to_time(v: &Value) -> Option<DateTime<Utc>> {
    match *v {
        Value::String(ref v_v) => {
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
        Value::Int(ref v_v) => Some(f_to_time(*v_v as f64)),
        Value::Float(ref v_v) => Some(f_to_time(*v_v)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use clause::{Operator, Value};

    #[test]
    fn test_op_in() {
        assert!(Operator::In.apply(
            &Value::String("A string to match".into()),
            &Value::String("A string to match".into()),
        ));
        assert!(!Operator::In.apply(
            &Value::String("A string to match".into()),
            &Value::Boolean(true),
        ));
        assert!(Operator::In.apply(&Value::Int(34), &Value::Int(34)));
        assert!(Operator::In.apply(&Value::Int(34), &Value::Float(34.0)));
        assert!(!Operator::In.apply(&Value::Int(34), &Value::Boolean(true)));
        assert!(Operator::In.apply(
            &Value::Boolean(false),
            &Value::Boolean(false),
        ));
        assert!(Operator::In.apply(
            &Value::Boolean(true),
            &Value::Boolean(true),
        ));
        assert!(!Operator::In.apply(
            &Value::Boolean(true),
            &Value::Boolean(false),
        ));
        assert!(!Operator::In.apply(
            &Value::Boolean(false),
            &Value::Boolean(true),
        ));
    }

    #[test]
    fn test_op_ends_with() {
        assert!(Operator::EndsWith.apply(
            &Value::String("end".into()),
            &Value::String("end".into()),
        ));
        assert!(Operator::EndsWith.apply(
            &Value::String("plus more end".into()),
            &Value::String("end".into()),
        ));
        assert!(!Operator::EndsWith.apply(
            &Value::String("does not contain".into()),
            &Value::String("end".into()),
        ));
        assert!(!Operator::EndsWith.apply(
            &Value::String(
                "does not end with".into(),
            ),
            &Value::String("end".into()),
        ));
        assert!(!Operator::EndsWith.apply(
            &Value::String("end".into()),
            &Value::Float(0.0),
        ));
        assert!(!Operator::EndsWith.apply(
            &Value::String("end".into()),
            &Value::Int(0),
        ));
        assert!(!Operator::EndsWith.apply(
            &Value::String("end".into()),
            &Value::Boolean(true),
        ));
        assert!(!Operator::EndsWith.apply(
            &Value::Float(0.0),
            &Value::String("end".into()),
        ));
        assert!(!Operator::EndsWith.apply(
            &Value::Int(0),
            &Value::String("end".into()),
        ));
        assert!(!Operator::EndsWith.apply(
            &Value::Boolean(true),
            &Value::String("end".into()),
        ));
    }

    #[test]
    fn test_op_starts_with() {
        assert!(Operator::StartsWith.apply(
            &Value::String("start".into()),
            &Value::String("start".into()),
        ));
        assert!(Operator::StartsWith.apply(
            &Value::String("start plus more".into()),
            &Value::String("start".into()),
        ));
        assert!(!Operator::StartsWith.apply(
            &Value::String(
                "does not contain".into(),
            ),
            &Value::String("start".into()),
        ));
        assert!(!Operator::StartsWith.apply(
            &Value::String(
                "does not start with".into(),
            ),
            &Value::String("start".into()),
        ));
        assert!(!Operator::StartsWith.apply(
            &Value::String("start".into()),
            &Value::Float(0.0),
        ));
        assert!(!Operator::StartsWith.apply(
            &Value::String("start".into()),
            &Value::Int(0),
        ));
        assert!(!Operator::StartsWith.apply(
            &Value::String("start".into()),
            &Value::Boolean(true),
        ));
        assert!(!Operator::StartsWith.apply(
            &Value::Float(0.0),
            &Value::String("start".into()),
        ));
        assert!(!Operator::StartsWith.apply(
            &Value::Int(0),
            &Value::String("start".into()),
        ));
        assert!(!Operator::StartsWith.apply(
            &Value::Boolean(true),
            &Value::String("start".into()),
        ));
    }

    #[test]
    fn test_op_matches() {
        assert!(Operator::Matches.apply(
            &Value::String("anything".into()),
            &Value::String(".*".into()),
        ));
        assert!(Operator::Matches.apply(
            &Value::String("darn".into()),
            &Value::String(
                "(\\W|^)(baloney|darn|drat|fooey|gosh\\sdarnit|heck)(\\W|$)".into(),
            ),
        ));
        assert!(!Operator::Matches.apply(
            &Value::String("barn".into()),
            &Value::String(
                "(\\W|^)(baloney|darn|drat|fooey|gosh\\sdarnit|heck)(\\W|$)"
                    .into(),
            ),
        ));
    }

    #[test]
    fn test_op_contains() {
        assert!(Operator::Contains.apply(
            &Value::String("contain".into()),
            &Value::String("contain".into()),
        ));
        assert!(Operator::Contains.apply(
            &Value::String("contain plus more".into()),
            &Value::String("contain".into()),
        ));
        assert!(!Operator::Contains.apply(
            &Value::String("does not".into()),
            &Value::String("contain".into()),
        ));
        assert!(!Operator::Contains.apply(
            &Value::String("contain".into()),
            &Value::Float(0.0),
        ));
        assert!(!Operator::Contains.apply(
            &Value::String("contain".into()),
            &Value::Int(0),
        ));
        assert!(!Operator::Contains.apply(
            &Value::String("contain".into()),
            &Value::Boolean(true),
        ));
        assert!(!Operator::Contains.apply(
            &Value::Float(0.0),
            &Value::String("contain".into()),
        ));
        assert!(!Operator::Contains.apply(
            &Value::Int(0),
            &Value::String("contain".into()),
        ));
        assert!(!Operator::Contains.apply(
            &Value::Boolean(true),
            &Value::String("contain".into()),
        ));
    }

    #[test]
    fn test_op_less_than() {
        let tests = vec![
            (Value::Int(0), Value::Int(1), true),
            (Value::Int(1), Value::Int(0), false),
            (Value::Int(0), Value::Int(0), false),
            (Value::Float(0.0), Value::Float(1.0), true),
            (Value::Float(1.0), Value::Float(0.0), false),
            (Value::Float(0.0), Value::Float(0.0), false),
            (Value::Int(0), Value::Float(1.0), true),
            (Value::Int(1), Value::Float(0.0), false),
            (Value::Int(0), Value::Float(0.0), false),
            (Value::Float(0.0), Value::Int(1), true),
            (Value::Float(1.0), Value::Int(0), false),
            (Value::Float(0.0), Value::Int(0), false),
            (Value::String("".into()), Value::Int(0), false),
            (Value::Int(0), Value::String("".into()), false),
            (Value::Boolean(true), Value::Int(1), false),
            (Value::Int(1), Value::Boolean(true), false),
        ];

        for (a, b, res) in tests {
            assert_eq!(Operator::LessThan.apply(&a, &b), res, "{:?} > {:?}", a, b);
        }
    }

    #[test]
    fn test_op_less_than_or_equal_to() {
        let tests = vec![
            (Value::Int(0), Value::Int(1), true),
            (Value::Int(1), Value::Int(0), false),
            (Value::Int(0), Value::Int(0), true),
            (Value::Float(0.0), Value::Float(1.0), true),
            (Value::Float(1.0), Value::Float(0.0), false),
            (Value::Float(0.0), Value::Float(0.0), true),
            (Value::Int(0), Value::Float(1.0), true),
            (Value::Int(1), Value::Float(0.0), false),
            (Value::Int(0), Value::Float(0.0), true),
            (Value::Float(0.0), Value::Int(1), true),
            (Value::Float(1.0), Value::Int(0), false),
            (Value::Float(0.0), Value::Int(0), true),
            (Value::String("".into()), Value::Int(0), false),
            (Value::Int(0), Value::String("".into()), false),
            (Value::Boolean(true), Value::Int(1), false),
            (Value::Int(1), Value::Boolean(true), false),
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
            (Value::Int(0), Value::Int(1), false),
            (Value::Int(1), Value::Int(0), true),
            (Value::Int(0), Value::Int(0), false),
            (Value::Float(0.0), Value::Float(1.0), false),
            (Value::Float(1.0), Value::Float(0.0), true),
            (Value::Float(0.0), Value::Float(0.0), false),
            (Value::Int(0), Value::Float(1.0), false),
            (Value::Int(1), Value::Float(0.0), true),
            (Value::Int(0), Value::Float(0.0), false),
            (Value::Float(0.0), Value::Int(1), false),
            (Value::Float(1.0), Value::Int(0), true),
            (Value::Float(0.0), Value::Int(0), false),
            (Value::String("".into()), Value::Int(0), false),
            (Value::Int(0), Value::String("".into()), false),
            (Value::Boolean(true), Value::Int(1), false),
            (Value::Int(1), Value::Boolean(true), false),
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
            (Value::Int(0), Value::Int(1), false),
            (Value::Int(1), Value::Int(0), true),
            (Value::Int(0), Value::Int(0), true),
            (Value::Float(0.0), Value::Float(1.0), false),
            (Value::Float(1.0), Value::Float(0.0), true),
            (Value::Float(0.0), Value::Float(0.0), true),
            (Value::Int(0), Value::Float(1.0), false),
            (Value::Int(1), Value::Float(0.0), true),
            (Value::Int(0), Value::Float(0.0), true),
            (Value::Float(0.0), Value::Int(1), false),
            (Value::Float(1.0), Value::Int(0), true),
            (Value::Float(0.0), Value::Int(0), true),
            (Value::String("".into()), Value::Int(0), false),
            (Value::Int(0), Value::String("".into()), false),
            (Value::Boolean(true), Value::Int(1), false),
            (Value::Int(1), Value::Boolean(true), false),
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
            (Value::Int(0), Value::Int(1), true),
            (Value::Int(1), Value::Int(0), false),
            (Value::Int(1), Value::Int(1), false),
            (Value::String("0".into()), Value::String("1".into()), true),
            (Value::String("1".into()), Value::String("0".into()), false),
            (Value::String("1".into()), Value::String("1".into()), false),
            (Value::Float(0.0), Value::Float(1.0), true),
            (Value::Float(1.0), Value::Float(0.0), false),
            (Value::Float(1.0), Value::Float(1.0), false),
            (
                Value::String("1970-01-01T00:00:01Z".into()),
                Value::String("1970-01-01T00:00:02Z".into()),
                true
            ),
            (
                Value::String("1970-01-01T00:00:01Z".into()),
                Value::String("1970-01-01T00:00:01.0001Z".into()),
                true
            ),
            (
                Value::Int(0),
                Value::String("1970-01-01T00:00:00.0001Z".into()),
                true
            ),
            (
                Value::Float(0.0),
                Value::String("1970-01-01T00:00:00.0001Z".into()),
                true
            ),
            (
                Value::Int(0),
                Value::String("1970-01-01----00:00:00.0001Z".into()),
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
            (Value::Int(0), Value::Int(1), false),
            (Value::Int(1), Value::Int(0), true),
            (Value::Int(1), Value::Int(1), false),
            (Value::String("0".into()), Value::String("1".into()), false),
            (Value::String("1".into()), Value::String("0".into()), true),
            (Value::String("1".into()), Value::String("1".into()), false),
            (Value::Float(0.0), Value::Float(1.0), false),
            (Value::Float(1.0), Value::Float(0.0), true),
            (Value::Float(1.0), Value::Float(1.0), false),
            (
                Value::String("1970-01-01T00:00:01Z".into()),
                Value::String("1970-01-01T00:00:02Z".into()),
                false
            ),
            (
                Value::String("1970-01-01T00:00:01Z".into()),
                Value::String("1970-01-01T00:00:01.0001Z".into()),
                false
            ),
            (
                Value::Int(0),
                Value::String("1970-01-01T00:00:00.0001Z".into()),
                false
            ),
            (
                Value::Float(0.0),
                Value::String("1970-01-01T00:00:00.0001Z".into()),
                false
            ),
            (
                Value::Int(0),
                Value::String("1970-01-01----00:00:00.0001Z".into()),
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
