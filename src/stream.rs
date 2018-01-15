use eventsource::event::Event;
use eventsource::reqwest::{Client, Error as EventSourceError};
use reqwest::Url;
use reqwest::header::{Authorization, Headers, UserAgent};
use serde_json;
use serde_json::Error as ParseError;

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::thread;

use feature_flag::FeatureFlag;
use request::{RequestError, Requestor};
use store::{Store, StoreError};
use VERSION;

#[derive(Debug)]
enum StreamError {
    EventSource(EventSourceError),
    FlagNotFound,
    ParseData(ParseError),
    ParseType,
    Request(RequestError),
    Storage(StoreError),
}

#[derive(Debug)]
enum StreamEventType {
    Put,
    Patch,
    Delete,
    IndirectPatch,
}

impl FromStr for StreamEventType {
    type Err = StreamError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "put" => Ok(StreamEventType::Put),
            "patch" => Ok(StreamEventType::Patch),
            "delete" => Ok(StreamEventType::Delete),
            "indirect/patch" => Ok(StreamEventType::IndirectPatch),
            _ => Err(StreamError::ParseType),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Patch {
    path: String,
    #[serde(rename = "data")]
    flag: FeatureFlag,
}

impl Patch {
    pub fn key(&self) -> &str {
        &self.path.as_str()[1..]
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Delete {
    path: String,
    version: usize,
}

impl Delete {
    pub fn key(&self) -> &str {
        &self.path.as_str()[1..]
    }
}

pub struct Streaming<S: Store + 'static> {
    store: Arc<S>,
    req: Arc<Requestor>,
}

impl<S: Store> Streaming<S> {
    pub fn new(store: Arc<S>, req: Arc<Requestor>) -> Streaming<S> {
        Streaming {
            store: store,
            req: req,
        }
    }

    pub fn run(self, endpoint: &str, key: &str) -> Result<thread::JoinHandle<()>, ()> {
        if let Ok(url) = Url::parse(endpoint) {
            let mut client = Client::new(url);

            let mut headers: Headers = Headers::new();
            headers.set(Authorization(key.to_string()));
            headers.set(UserAgent::new("RustTest/".to_string() + VERSION));

            client.default_headers = headers;

            Ok(thread::spawn(move || {
                for msg in client {
                    msg.map_err(StreamError::EventSource).and_then(|event| {
                        Self::get_event_type(&event).and_then(|event_type| {
                            self.process_data(&event_type, event.data.as_str())
                        })
                    });
                }
            }))
        } else {
            Err(())
        }
    }

    fn get_event_type(event: &Event) -> Result<StreamEventType, StreamError> {
        match event.event_type {
            Some(ref type_str) => type_str.parse::<StreamEventType>(),
            None => Err(StreamError::ParseType),
        }
    }

    fn process_data(&self, event_type: &StreamEventType, data: &str) -> Result<(), StreamError> {
        match *event_type {
            StreamEventType::Put => {
                let flags = serde_json::from_str::<HashMap<String, FeatureFlag>>(data)
                    .map_err(StreamError::ParseData)?;
                self.store.init(flags).map_err(StreamError::Storage)
            }
            StreamEventType::Patch => {
                let patch = serde_json::from_str::<Patch>(data).map_err(StreamError::ParseData)?;
                self.store
                    .upsert(patch.key(), &patch.flag)
                    .map_err(StreamError::Storage)
            }
            StreamEventType::Delete => {
                let delete = serde_json::from_str::<Delete>(data).map_err(StreamError::ParseData)?;
                self.store
                    .delete(delete.key(), delete.version)
                    .map_err(StreamError::Storage)
            }
            StreamEventType::IndirectPatch => match self.req.get(data) {
                Ok(Some(flag)) => self.store.upsert(data, &flag).map_err(StreamError::Storage),
                Ok(None) => Err(StreamError::FlagNotFound),
                Err(err) => Err(StreamError::Request(err)),
            },
        }
    }
}
