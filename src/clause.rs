pub struct Clause {
    attribute: String,
    op: Operator,
    values: Vec<Value>,
    negate: bool,
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
            Operator::Matches => false,
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
            _ => false,
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
        unimplemented!()
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
        unimplemented!()
    }

    #[test]
    fn test_op_after() {
        unimplemented!()
    }
}
