use chrono::Utc;
use reqwest::Client;
use reqwest::header::{Authorization, ContentType, Headers, UserAgent};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json;

use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use clause::Value;
use feature_flag::VariationValue;
use user::User;
use VERSION;

#[derive(Serialize)]
#[serde(untagged)]
pub enum Event {
    FeatureRequest(FeatureRequestEvent),
}

#[derive(Debug)]
pub enum EventError {
    FailedToParseEndpoint,
}

#[derive(Debug)]
pub enum Kind {
    FeatureRequestEvent,
    CustomEvent,
    IdentifyEvent,
}

impl Serialize for Kind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(match *self {
            Kind::FeatureRequestEvent => "feature",
            Kind::CustomEvent => "custom",
            Kind::IdentifyEvent => "indentify",
        })
    }
}

impl<'de> Deserialize<'de> for Kind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "feature" => Ok(Kind::FeatureRequestEvent),
            "custom" => Ok(Kind::CustomEvent),
            "indentify" => Ok(Kind::IdentifyEvent),
            _ => Err(::serde::de::Error::custom("Invalid event kind")),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureRequestEvent {
    #[serde(rename = "creationDate")]
    creation_date: i64,
    key: String,
    user: User,
    kind: Kind,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<VariationValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default: Option<Value>,
    version: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    prereq_of: Option<String>,
}

impl<'a> FeatureRequestEvent {
    pub fn new(
        key: &str,
        user: &User,
        value: Option<VariationValue>,
        default: Option<Value>,
        version: usize,
        prereq_of: Option<String>,
    ) -> FeatureRequestEvent {
        FeatureRequestEvent {
            creation_date: Utc::now().timestamp() * 1000,
            key: key.into(),
            user: user.clone(),
            kind: Kind::FeatureRequestEvent,
            value: value,
            default: default,
            version: version,
            prereq_of: prereq_of,
        }
    }
}

pub struct EventProcessor {
    send_events: bool,
    sampling_interval: i64,
    channel: Sender<Event>,
}

impl EventProcessor {
    pub fn new(active: bool, sampling_interval: i64, channel: Sender<Event>) -> EventProcessor {
        EventProcessor {
            send_events: active,
            sampling_interval: sampling_interval,
            channel: channel,
        }
    }

    pub fn push(&self, e: Event) {
        if self.send_events && self.sampling_interval == 0 {
            self.channel.send(e);
        }
    }
}

pub struct EventSender {
    flush_interval: i64,
    stream: Receiver<Event>,
}

impl EventSender {
    pub fn new(flush_interval: i64, stream: Receiver<Event>) -> EventSender {
        EventSender {
            flush_interval: flush_interval,
            stream: stream,
        }
    }

    pub fn run<S: Into<String>, T: Into<String>>(
        self,
        endpoint: S,
        key: T,
    ) -> Result<thread::JoinHandle<()>, EventError> {
        let k = key.into();
        let e = endpoint.into();

        Ok(thread::spawn(move || {
            let client = Client::new();

            let flush_interval = self.flush_interval;
            let mut start = Utc::now().timestamp();
            let mut batch = vec![];

            let mut headers = Headers::new();
            headers.set(Authorization(k.clone()));
            headers.set(ContentType::json());
            headers.set(UserAgent::new("RustTest/".to_string() + VERSION));

            loop {
                if let Ok(event) = self.stream.try_recv() {
                    batch.push(event);
                }

                if start + flush_interval < Utc::now().timestamp() {
                    if batch.len() > 0 {
                        if let Ok(data) = serde_json::to_string(&batch) {
                            client.post(e.as_str()).headers(headers.clone()).body(data);
                        }

                        // Always reset, if a batch fails to serialize once,
                        // the next attempt will fail as well
                        batch = vec![];
                    }

                    start = Utc::now().timestamp();
                }
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;

    use events::*;
    use user::*;

    #[test]
    fn test_recieves_event() {
        let u = UserBuilder::new("user_key").build();

        let (tx, rx) = channel();

        let processor = EventProcessor::new(true, 0, tx);
        let sender = EventSender::new(0, rx);

        let handle = sender.run("https://0.0.0.0", "").unwrap();

        processor.push(Event::FeatureRequest(
            FeatureRequestEvent::new("level-1", &u, None, None, 1, None),
        ));

        // Wait to make sure the sender ticks
        ::std::thread::sleep(::std::time::Duration::new(2, 0));

        drop(handle);
    }
}
