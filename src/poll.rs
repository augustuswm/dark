use std::thread;
use std::time::Duration;

use feature_flag::FeatureFlag;
use request::Requestor;
use store::FeatureStore;

pub struct Polling<S: 'static + FeatureStore> {
    store: S,
    req: Requestor,
    interval: i64,
}

impl<S: 'static + FeatureStore> Polling<S> {
    pub fn new(store: S, req: Requestor, interval: i64) -> Polling<S> {
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
