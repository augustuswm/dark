use std::sync::Arc;
use std::sync::mpsc::channel;
use std::thread::JoinHandle;

use config::Config;
use events::{EventProcessor, EventSender};
use feature_flag::{Eval, FeatureFlag, VariationValue};
use mem_store::MemStore;
use poll::Polling;
use request::Requestor;
use store::Store;
use stream::Streaming;
use user::User;

// pub base_uri: String, /
// pub stream_uri: String, /
// pub events_uri: String, /
// pub capacity: i64, - To do
// pub flush_interval: i64, /
// pub sampling_interval: i64, /
// pub poll_interval: i64, /
// pub timeout: i64, - Not implemented
// pub stream: bool, /
// pub use_ldd: bool, /
// pub send_events: bool, /
// pub offline: bool, /
// pub store: T, /

type FlagEvaluation = (VariationValue, Option<usize>);

type ClientResult<T> = Result<T, &'static str>;

pub struct Client<S: Store + 'static> {
    offline: bool,
    event_processor: Option<EventProcessor>,
    event_handle: Option<JoinHandle<()>>,
    update_handle: Option<JoinHandle<()>>,
    store: Arc<S>,
}

impl<S: Store> Client<S> {
    pub fn new(key: &str, config: Config<S>) -> Client<S> {
        let store = Arc::new(config.store);

        if config.use_ldd {
            Client {
                offline: config.offline,
                event_processor: None,
                event_handle: None,
                update_handle: None,
                store: store,
            }
        } else {
            let req = Arc::new(Requestor::new(config.base_uri, key));

            let update_handle = if config.stream {
                let stream = Streaming::new(store.clone(), req.clone());
                stream.run(config.stream_uri.as_str(), key).unwrap()
            } else {
                Polling::new(store.clone(), req.clone(), config.poll_interval).run()
            };

            let (tx, rx) = channel();
            let e_processor = EventProcessor::new(
                !config.offline && config.send_events,
                config.sampling_interval,
                tx,
            );
            let e_sender = EventSender::new(config.flush_interval, rx);
            let e_handle = e_sender.run(config.events_uri, key);

            Client {
                offline: config.offline,
                event_processor: Some(e_processor),
                event_handle: Some(e_handle),
                update_handle: Some(update_handle),
                store: store,
            }
        }
    }

    fn flag(&self, key: &str, user: &User) -> Option<FeatureFlag> {
        if user.key() == "" {
            warn!("_MESSAGE_USER_KEY_IS_EMPTY");
        }

        self.store.get(key)
    }

    pub fn evaluate<V: Into<VariationValue>>(
        &self,
        key: &str,
        user: &User,
        default: V,
    ) -> FlagEvaluation {
        if self.offline {
            return (default.into(), None);
        }

        if let Some(flag) = self.flag(key, user) {
            self.eval(&flag, user).unwrap_or(
                (default.into(), Some(flag.version())),
            )
        } else {
            (default.into(), None)
        }
    }

    fn eval(&self, flag: &FeatureFlag, user: &User) -> ClientResult<FlagEvaluation> {
        let Eval { result, events } = flag.evaluate(user, &self.store);

        if let Some(ref p) = self.event_processor {
            for event in events {
                p.push(event);
            }
        }

        result
            .value
            .map(|val| (val, Some(flag.version())))
            .map_err(|_| "Failed to eval")
    }

    pub fn bool_variation(&self, key: &str, user: &User, default: bool) -> bool {
        match self.variation(key, user) {
            Some(VariationValue::Boolean(val)) => val,
            _ => default,
        }
    }

    pub fn int_variation(&self, key: &str, user: &User, default: i64) -> i64 {
        match self.variation(key, user) {
            Some(VariationValue::Integer(val)) => val,
            _ => default,
        }
    }

    pub fn float_variation(&self, key: &str, user: &User, default: f64) -> f64 {
        match self.variation(key, user) {
            Some(VariationValue::Float(val)) => val,
            _ => default,
        }
    }

    pub fn string_variation<V: Into<String>>(&self, key: &str, user: &User, default: V) -> String {
        match self.variation(key, user) {
            Some(VariationValue::String(val)) => val,
            _ => default.into(),
        }
    }

    fn variation(&self, key: &str, user: &User) -> Option<VariationValue> {
        self.flag(key, user).and_then(|flag| {
            self.eval(&flag, user).map(|(variation, _)| variation).ok()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offline_returns_default() {
        unimplemented!()
    }

    #[test]
    fn test_toggle() {
        unimplemented!()
    }

    #[test]
    fn test_int() {
        unimplemented!()
    }

    #[test]
    fn test_float() {
        unimplemented!()
    }

    #[test]
    fn test_json() {
        unimplemented!()
    }
}

// package ldclient
//
// import (
// 	"encoding/json"
// 	"fmt"
// 	"log"
// 	"os"
// 	"testing"
// 	"time"
// )
//
// type TestUpdateProcessor struct{}
//
// func (u TestUpdateProcessor) initialized() bool { return true }
// func (u TestUpdateProcessor) close()            {}
// func (u TestUpdateProcessor) start(chan<- struct{}) {}
//
// func TestOfflineModeAlwaysReturnsDefaultValue(t *testing.T) {
// 	config := Config{
// 		BaseUri:       "https://localhost:3000",
// 		Capacity:      1000,
// 		FlushInterval: 5 * time.Second,
// 		Logger:        log.New(os.Stderr, "[LaunchDarkly]", log.LstdFlags),
// 		Timeout:       1500 * time.Millisecond,
// 		Stream:        true,
// 		Offline:       true,
// 	}
// 	client, _ := MakeCustomClient("api_key", config, 0)
// 	defer client.Close()
// 	client.config.Offline = true
// 	key := "foo"
// 	user := User{Key: &key}
//
// 	//Toggle
// 	expected := true
// 	actual, err := client.Toggle("featureKey", user, expected)
// 	if err != nil {
// 		t.Errorf("Unexpected error in Toggle: %+v", err)
// 	}
// 	if actual != expected {
// 		t.Errorf("Offline mode should return default value, but doesn't")
// 	}
//
// 	//IntVariation
// 	expectedInt := 100
// 	actualInt, err := client.IntVariation("featureKey", user, expectedInt)
// 	if err != nil {
// 		t.Errorf("Unexpected error in IntVariation: %+v", err)
// 	}
// 	if actualInt != expectedInt {
// 		t.Errorf("Offline mode should return default value: %+v, instead returned: %+v", expectedInt, actualInt)
// 	}
//
// 	//Float64Variation
// 	expectedFloat64 := 100.0
// 	actualFloat64, err := client.Float64Variation("featureKey", user, expectedFloat64)
// 	if err != nil {
// 		t.Errorf("Unexpected error in Float64Variation: %+v", err)
// 	}
// 	if actualFloat64 != expectedFloat64 {
// 		t.Errorf("Offline mode should return default value, but doesn't")
// 	}
//
// 	//StringVariation
// 	expectedString := "expected"
// 	actualString, err := client.StringVariation("featureKey", user, expectedString)
// 	if err != nil {
// 		t.Errorf("Unexpected error in StringVariation: %+v", err)
// 	}
// 	if actualString != expectedString {
// 		t.Errorf("Offline mode should return default value, but doesn't")
// 	}
//
// 	//JsonVariation
// 	expectedJsonString := `{"fieldName":"fieldValue"}`
// 	expectedJson := json.RawMessage([]byte(expectedJsonString))
// 	actualJson, err := client.JsonVariation("featureKey", user, expectedJson)
// 	if err != nil {
// 		t.Errorf("Unexpected error in JsonVariation: %+v", err)
// 	}
// 	if string([]byte(actualJson)) != string([]byte(expectedJson)) {
// 		t.Errorf("Offline mode should return default value (%+v), instead got: %+v", expectedJson, actualJson)
// 	}
//
// 	client.Close()
// }
//
// func TestToggle(t *testing.T) {
// 	expected := true
//
// 	variations := make([]interface{}, 2)
// 	variations[0] = false
// 	variations[1] = expected
//
// 	client := makeClientWithFeatureFlag(variations)
// 	defer client.Close()
//
// 	userKey := "userKey"
// 	actual, err := client.Toggle("validFeatureKey", User{Key: &userKey}, false)
//
// 	if err != nil {
// 		t.Errorf("Unexpected error when calling Toggle: %+v", err)
// 	}
// 	if actual != expected {
// 		t.Errorf("Got unexpected result when calling Toggle: %+v but expected: %+v", actual, expected)
// 	}
// }
//
// func TestIntVariation(t *testing.T) {
// 	expected := float64(100)
//
// 	variations := make([]interface{}, 2)
// 	variations[0] = float64(-1)
// 	variations[1] = expected
//
// 	client := makeClientWithFeatureFlag(variations)
// 	defer client.Close()
//
// 	userKey := "userKey"
// 	actual, err := client.IntVariation("validFeatureKey", User{Key: &userKey}, 10000)
//
// 	if err != nil {
// 		t.Errorf("Unexpected error when calling IntVariation: %+v", err)
// 	}
// 	if actual != int(expected) {
// 		t.Errorf("Got unexpected result when calling IntVariation: %+v but expected: %+v", actual, expected)
// 	}
// }
//
// func TestFloat64Variation(t *testing.T) {
// 	expected := 100.01
//
// 	variations := make([]interface{}, 2)
// 	variations[0] = -1.0
// 	variations[1] = expected
//
// 	client := makeClientWithFeatureFlag(variations)
// 	defer client.Close()
//
// 	userKey := "userKey"
// 	actual, err := client.Float64Variation("validFeatureKey", User{Key: &userKey}, 0.0)
//
// 	if err != nil {
// 		t.Errorf("Unexpected error when calling Float64Variation: %+v", err)
// 	}
// 	if actual != expected {
// 		t.Errorf("Got unexpected result when calling Float64Variation: %+v but expected: %+v", actual, expected)
// 	}
// }
//
// func TestJsonVariation(t *testing.T) {
// 	expectedJsonString := `{"jsonFieldName2":"fallthroughValue"}`
//
// 	var variations []interface{}
// 	json.Unmarshal([]byte(fmt.Sprintf(`[{"jsonFieldName1" : "jsonFieldValue"},%s]`, expectedJsonString)), &variations)
//
// 	client := makeClientWithFeatureFlag(variations)
// 	defer client.Close()
//
// 	userKey := "userKey"
// 	var actual json.RawMessage
// 	actual, err := client.JsonVariation("validFeatureKey", User{Key: &userKey}, []byte(`{"default":"default"}`))
//
// 	if err != nil {
// 		t.Errorf("Unexpected error when calling JsonVariation: %+v", err)
// 	}
// 	if string(actual) != expectedJsonString {
// 		t.Errorf("Got unexpected result when calling JsonVariation: %+v but expected: %+v", string(actual), expectedJsonString)
// 	}
// }
//
// func TestSecureModeHash(t *testing.T) {
// 	expected := "aa747c502a898200f9e4fa21bac68136f886a0e27aec70ba06daf2e2a5cb5597"
// 	key := "Message"
// 	config := DefaultConfig
// 	config.Offline = true
//
// 	client, _ := MakeCustomClient("secret", config, 0*time.Second)
//
// 	hash := client.SecureModeHash(User{Key: &key})
//
// 	if hash != expected {
// 		t.Errorf("Got unexpected result when calling SecureModeHash: %s but expected %s", hash, expected)
// 	}
// }
//
// // Creates LdClient loaded with one feature flag with key: "validFeatureKey".
// // Variations param should have at least 2 items with variations[1] being the expected
// // fallthrough value when passing in a valid user
// func makeClientWithFeatureFlag(variations []interface{}) *LDClient {
// 	config := Config{
// 		BaseUri:       "https://localhost:3000",
// 		Capacity:      1000,
// 		FlushInterval: 5 * time.Second,
// 		Logger:        log.New(os.Stderr, "[LaunchDarkly]", log.LstdFlags),
// 		Timeout:       1500 * time.Millisecond,
// 		Stream:        true,
// 		Offline:       false,
// 		SendEvents:    false,
// 	}
//
// 	client := LDClient{
// 		sdkKey:          "sdkKey",
// 		config:          config,
// 		eventProcessor:  newEventProcessor("sdkKey", config),
// 		updateProcessor: TestUpdateProcessor{},
// 		store:           NewInMemoryFeatureStore(nil),
// 	}
// 	featureFlag := featureFlagWithVariations(variations)
//
// 	client.store.Upsert(featureFlag.Key, featureFlag)
// 	return &client
// }
//
// func featureFlagWithVariations(variations []interface{}) FeatureFlag {
// 	fallThroughVariation := 1
//
// 	return FeatureFlag{
// 		Key:         "validFeatureKey",
// 		Version:     1,
// 		On:          true,
// 		Fallthrough: VariationOrRollout{Variation: &fallThroughVariation},
// 		Variations:  variations,
// 	}
// }
