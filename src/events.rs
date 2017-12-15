use clause::Value;
use feature_flag::Variation;
use user::User;

pub enum Kind {
    FeatureRequestEvent,
    CustomEvent,
    IdentifyEvent,
}

pub struct BaseEvent<'a> {
    creation_date: u64,
    key: String,
    user: &'a User,
    kind: Kind,
}

pub struct FeatureRequestEvent<'a> {
    base_event: BaseEvent<'a>,
    value: Variation,
    default: Value,
    version: i64,
    prereq_of: String,
}
