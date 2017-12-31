use reqwest::{Client, Error as ReqError};
use reqwest::header::{Authorization, Headers, UserAgent};
use serde::Deserialize;

use std::collections::HashMap;

use feature_flag::FeatureFlag;
use VERSION;

const FLAG_PATH: &'static str = "/sdk/latest-flags";

pub type RequestResult<T> = Result<T, RequestError>;

#[derive(Debug)]
pub enum RequestError {
    Testing,
    HTTPFailure(ReqError),
    ParseFailure(ReqError),
}

pub struct Requestor {
    client: Client,
    base_uri: String,
    key: String,
}

// TODO: Does not implement any caching, retries, or backoff

impl Requestor {
    pub fn new<S, T>(base_uri: S, key: T) -> Requestor
    where
        S: Into<String>,
        T: Into<String>,
    {
        Requestor {
            client: Client::new(),
            base_uri: base_uri.into(),
            key: key.into(),
        }
    }

    pub fn get_all(&self) -> RequestResult<HashMap<String, FeatureFlag>> {
        // Ok(vec![])
        self.request(self.base_uri.to_string() + FLAG_PATH)
    }

    pub fn get(&self, key: &str) -> RequestResult<Option<FeatureFlag>> {
        // Ok(None)
        self.request(self.base_uri.to_string() + FLAG_PATH + "/" + key)
            .map(|flag| Some(flag))
    }

    fn request<S: Into<String>, T>(&self, endpoint: S) -> RequestResult<T>
    where
        for<'de> T: Deserialize<'de>,
    {
        let mut headers = Headers::new();
        headers.set(Authorization(self.key.clone()));
        headers.set(UserAgent::new("RustTest/".to_string() + VERSION));

        // Err(RequestError::Testing)
        let mut inter = self.client
            .get(endpoint.into().as_str())
            .headers(headers)
            .send()
            .map_err(RequestError::HTTPFailure)?;
        // panic!("{:?}", inter.text());
        // panic!("{:?}", inter.json::<HashMap<String, FeatureFlag>>());
        inter.json().map_err(RequestError::ParseFailure)
    }
}
