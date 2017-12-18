use mem_store::MemStore;
use redis_store::RedisStore;
use store::FeatureStore;

pub struct Config<T: FeatureStore> {
    pub base_uri: String,
    pub stream_uri: String,
    pub events_uri: String,
    pub capacity: i64,
    pub flush_interval: i64,
    pub poll_interval: i64,
    pub timeout: i64,
    pub stream: bool,
    pub use_ldd: bool,
    pub send_events: bool,
    pub offline: bool,
    pub store: T,
}

pub struct ConfigBuilder<T: FeatureStore> {
    config: Config<T>,
}

impl<T: FeatureStore> From<Config<T>> for ConfigBuilder<T> {
    fn from(config: Config<T>) -> ConfigBuilder<T> {
        ConfigBuilder { config: config }
    }
}

impl<T: FeatureStore> From<ConfigBuilder<T>> for Config<T> {
    fn from(builder: ConfigBuilder<T>) -> Config<T> {
        builder.build()
    }
}

impl ConfigBuilder<MemStore> {
    pub fn new() -> ConfigBuilder<MemStore> {
        ConfigBuilder {
            config: Config {
                base_uri: "https://app.launchdarkly.com".into(),
                stream_uri: "https://stream.launchdarkly.com".into(),
                events_uri: "https://events.launchdarkly.com".into(),
                capacity: 1000,
                flush_interval: 5,
                poll_interval: 1,
                timeout: 3,
                stream: true,
                use_ldd: false,
                send_events: true,
                offline: false,
                store: MemStore::new(),
            },
        }
    }
}

impl<T: FeatureStore> ConfigBuilder<T> {
    pub fn base_uri<S: Into<String>>(mut self, uri: S) -> Self {
        self.config.base_uri = uri.into();
        self
    }

    pub fn stream_uri<S: Into<String>>(mut self, uri: S) -> Self {
        self.config.stream_uri = uri.into();
        self
    }

    pub fn events_uri<S: Into<String>>(mut self, uri: S) -> Self {
        self.config.events_uri = uri.into();
        self
    }

    pub fn capacity(mut self, capacity: i64) -> Self {
        self.config.capacity = capacity;
        self
    }

    pub fn flush_interval(mut self, flush_interval: i64) -> Self {
        self.config.flush_interval = flush_interval;
        self
    }

    pub fn poll_interval(mut self, poll_interval: i64) -> Self {
        self.config.poll_interval = poll_interval;
        self
    }

    pub fn timeout(mut self, timeout: i64) -> Self {
        self.config.timeout = timeout;
        self
    }

    pub fn stream(mut self, stream: bool) -> Self {
        self.config.stream = stream;
        self
    }

    pub fn use_ldd(mut self, use_ldd: bool) -> Self {
        self.config.use_ldd = use_ldd;
        self
    }

    pub fn send_events(mut self, send_events: bool) -> Self {
        self.config.send_events = send_events;
        self
    }

    pub fn offline(mut self, offline: bool) -> Self {
        self.config.offline = offline;
        self
    }

    pub fn store<S: FeatureStore>(self, store: S) -> ConfigBuilder<S> {
        let config = self.build();

        ConfigBuilder {
            config: Config {
                base_uri: config.base_uri,
                stream_uri: config.stream_uri,
                events_uri: config.events_uri,
                capacity: config.capacity,
                flush_interval: config.flush_interval,
                poll_interval: config.poll_interval,
                timeout: config.timeout,
                stream: config.stream,
                use_ldd: config.use_ldd,
                send_events: config.send_events,
                offline: config.offline,
                store: store,
            },
        }
    }

    pub fn build(self) -> Config<T> {
        self.config
    }
}
