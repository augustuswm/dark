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
    creation_date: u64,
    key: String,
    user: &'a User,
    kind: Kind,
}

#[derive(Debug)]
pub struct FeatureRequestEvent<'a> {
    base_event: BaseEvent<'a>,
    value: Variation,
    default: Value,
    version: i64,
    prereq_of: String,
}
