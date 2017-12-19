use hyper::{Body, Client, Method, Request};
use hyper::client::HttpConnector;
use hyper::header::ContentType;
use tokio_core::reactor::Core;

use events::{Event, EventProcessor};
use VERSION;

// use std::io::Read;
//
// let mut resp = reqwest::get("https://www.rust-lang.org")?;
// assert!(resp.status().is_success());
//
// let mut content = String::new();
// resp.read_to_string(&mut content);

pub struct Comm {
    client: Client<HttpConnector, Body>,
    core: Core,
    events: EventProcessor,
}

impl Comm {
    pub fn new(events: EventProcessor) -> Result<Comm, ()> {
        if let Ok(core) = Core::new() {
            Ok(Comm {
                client: Client::new(&core.handle()),
                core: core,
                events: events,
            })
        } else {
            Err(())
        }
    }

    // pub fn run(&mut self) -> {
    //     self.events.and_then(|event_batch| {
    //
    //     })
    //
    //     self.core.run()
    // }

    fn send_events(endpoint: &str, key: &str, events: Vec<Event>) -> () {

        // let json = r#"{"library":"hyper"}"#;
        // let uri = "http://httpbin.org/post".parse()?;
        // let mut req = Request::new(Method::Post, uri);
        // req.headers_mut().set(ContentType::json());
        // req.headers_mut().set(ContentLength(json.len() as u64));
        // req.set_body(json);

        // req.Header.Add("Authorization", ep.sdkKey)
        // req.Header.Add("Content-Type", "application/json")
        // req.Header.Add("User-Agent", "GoClient/"+Version)

        if let Ok(uri) = endpoint.parse() {
            let mut req = Request::new(Method::Post, uri);
            // req.headers_mut().set("Authorization", key);
            req.headers_mut().set(ContentType::json());
            // req.headers_mut().set("User-Agent", "RustTest".to_string() + VERSION);
        };
    }
}


// pub fn startup() {
//
//     let mut core = Core::new().unwrap();
//     let address = "0.0.0.0:12345".parse().unwrap();
//     let listener = TcpListener::bind(&address, &core.handle()).unwrap();
//
//     let connections = listener.incoming();
//     let welcomes = connections.and_then(|(socket, _peer_addr)| {
//         tokio_io::io::write_all(socket, b"Hello, world!\n")
//     });
//     let server = welcomes.for_each(|(_socket, _welcome)| {
//         Ok(())
//     });
//
//     core.run(server).unwrap();
// }
