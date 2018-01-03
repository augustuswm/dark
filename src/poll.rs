use std::thread;
use std::time::Duration;
use std::sync::Arc;

use request::Requestor;
use store::Store;

pub struct Polling<S: Store + 'static> {
    store: Arc<S>,
    req: Arc<Requestor>,
    interval: i64,
}

impl<S: Store> Polling<S> {
    pub fn new(store: Arc<S>, req: Arc<Requestor>, interval: i64) -> Polling<S> {
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
