use chrono::Utc;
use futures::Poll;
use futures::stream::Stream;

use std::sync::{Arc, RwLock};

use clause::Value;
use feature_flag::Variation;
use user::User;

pub enum Event {
    FeatureRequest(FeatureRequestEvent),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Kind {
    FeatureRequestEvent,
    CustomEvent,
    IdentifyEvent,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BaseEvent {
    creation_date: i64,
    key: String,
    user: User,
    kind: Kind,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureRequestEvent {
    base_event: BaseEvent,
    value: Option<Variation>,
    default: Option<Value>,
    version: usize,
    prereq_of: String,
}

impl<'a> FeatureRequestEvent {
    pub fn new(
        key: &str,
        user: &User,
        value: Option<Variation>,
        default: Option<Value>,
        version: usize,
        prereq_of: &str,
    ) -> FeatureRequestEvent {
        FeatureRequestEvent {
            base_event: BaseEvent {
                creation_date: Utc::now().timestamp(),
                key: key.into(),
                user: user.clone(),
                kind: Kind::FeatureRequestEvent,
            },
            value: value,
            default: default,
            version: version,
            prereq_of: prereq_of.into(),
        }
    }
}

// send_events, sampling_interval, capacity, flush_interval, events_uri

pub struct EventProcessor {
    send_events: bool,
    sampling_interval: i64,
    capacity: i64,
    flush_interval: i64,
    events_uri: String,
    events: Arc<RwLock<Vec<Event>>>,
}

impl EventProcessor {
    pub fn new(
        send_events: bool,
        sampling_interval: i64,
        capacity: i64,
        flush_interval: i64,
        events_uri: String,
    ) -> EventProcessor {
        EventProcessor {
            send_events: send_events,
            sampling_interval: sampling_interval,
            capacity: capacity,
            flush_interval: flush_interval,
            events_uri: events_uri,
            events: Arc::new(RwLock::new(vec![])),
        }
    }

    pub fn push(&self, e: Event) {
        self.events.write().unwrap().push(e)
    }
}

impl Stream for EventProcessor {
    type Item = Vec<Event>;
    type Error = &'static str;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        if !self.send_events {
            return Ok(None.into());
        }

        if let Ok(mut writer) = self.events.write() {
            if writer.len() > 0 {
                Ok(Some(writer.drain(..).collect::<Vec<Event>>()).into())
            } else {
                Ok(None.into())
            }
        } else {
            Err("Failed to mutably access events")
        }
    }
}
