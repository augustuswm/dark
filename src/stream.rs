use eventsource::reqwest::Client;
use reqwest::Url;
use reqwest::header::{Authorization, Headers, UserAgent};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json;

use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::str::FromStr;

use feature_flag::FeatureFlag;
use request::Requestor;
use store::FeatureStore;
use VERSION;

#[derive(Debug)]
enum StreamEventType {
    Put,
    Patch,
    Delete,
    IndirectPatch,
}

impl FromStr for StreamEventType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "put" => Ok(StreamEventType::Put),
            "patch" => Ok(StreamEventType::Patch),
            "delete" => Ok(StreamEventType::Delete),
            "indirect/patch" => Ok(StreamEventType::IndirectPatch),
            _ => Err("Invalid event type"),
        }
    }
}

pub struct Streaming<S: 'static + FeatureStore> {
    store: S,
    req: Requestor,
}

impl<S: 'static + FeatureStore> Streaming<S> {
    pub fn new(store: S, req: Requestor) -> Streaming<S> {
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
                    match msg {
                        Ok(event) => {
                            match event.event_type {
                                Some(type_str) => {
                                    if let Ok(event_type) = type_str.parse::<StreamEventType>() {
                                        println!("{:?}", event_type);
                                        match event_type {
                                            StreamEventType::Put => {
                                                if let Ok(flags) = serde_json::from_str::<HashMap<String, FeatureFlag>>(event.data.as_str()) {
                                                    println!("{:?}", flags);
                                                    self.store.init(flags);
                                                }
                                            }
                                            StreamEventType::Patch => {
                                                println!("{:?}", event.data);
                                            }
                                            StreamEventType::Delete => {
                                                println!("{:?}", event.data);
                                            }
                                            StreamEventType::IndirectPatch => {
                                                println!("{:?}", event.data);
                                            }
                                        }
                                    };
                                }
                                None => (),
                            };
                            // match event.event_type {
                            //     Some(type_ser) => {
                            //         let e_type: StreamEventType = serde_json::from_str(type_ser.as_str()).unwrap();
                            //         panic!("{:?}", e_type);
                            //     },
                            //     None => ()
                            // }
                        }
                        Err(err) => println!("{:?}", err),
                    }
                }
                // loop {
                //     let res = self.req.get_all();
                //
                //     if let Ok(flags) = res {
                //         self.store.init(flags);
                //     }
                //
                //     thread::sleep(Duration::new(self.interval as u64, 0));
                // }
            }))
        } else {
            Err(())

        }
    }
}
