use std::thread;
use std::time::Duration;
use std::sync::Arc;

use feature_flag::FeatureFlag;
use request::Requestor;
use store::FeatureStore;

pub struct Polling {
    store: Arc<FeatureStore>,
    req: Arc<Requestor>,
    interval: i64,
}

impl Polling {
    pub fn new(store: Arc<FeatureStore>, req: Arc<Requestor>, interval: i64) -> Polling {
        Polling {
            store: store,
            req: req,
            interval: interval,
        }
    }

    pub fn run(self) -> thread::JoinHandle<()> {
        thread::spawn(move || loop {
            let res = self.req.get_all();

            if let Ok(flags) = res {
                self.store.init(flags);
            }

            thread::sleep(Duration::new(self.interval as u64, 0));
        })
    }
}
