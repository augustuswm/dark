extern crate chrono;
use self::chrono::Utc;

use clause::Value;
use feature_flag::Variation;
use user::User;

#[derive(Debug)]
pub enum Kind {
    FeatureRequestEvent,
    CustomEvent,
    IdentifyEvent,
}

#[derive(Debug)]
pub struct BaseEvent<'a> {
    creation_date: i64,
    key: String,
    user: &'a User,
    kind: Kind,
}

#[derive(Debug)]
pub struct FeatureRequestEvent<'a> {
    base_event: BaseEvent<'a>,
    value: Option<Variation>,
    default: Option<Value>,
    version: usize,
    prereq_of: String,
}

impl<'a> FeatureRequestEvent<'a> {
    pub fn new(
        key: &str,
        user: &'a User,
        value: Option<Variation>,
        default: Option<Value>,
        version: usize,
        prereq_of: &str,
    ) -> FeatureRequestEvent<'a> {
        FeatureRequestEvent {
            base_event: BaseEvent {
                creation_date: Utc::now().timestamp(),
                key: key.into(),
                user: user,
                kind: Kind::FeatureRequestEvent,
            },
            value: value,
            default: default,
            version: version,
            prereq_of: prereq_of.into(),
        }
    }
}
