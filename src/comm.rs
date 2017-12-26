// use futures::{Future, Poll, Stream};
// use futures::stream::{Map, StreamFuture};
// use hyper::{Body, Client, Method, Request, Response};
// use hyper::client::{FutureResponse, HttpConnector};
// use hyper::header::{Authorization, ContentType, UserAgent};
// use tokio_core::reactor::{Core, Handle};
//
// use events::{Event, EventProcessor};
// use VERSION;
//
// // use std::io::Read;
// //
// // let mut resp = reqwest::get("https://www.rust-lang.org")?;
// // assert!(resp.status().is_success());
// //
// // let mut content = String::new();
// // resp.read_to_string(&mut content);
//
// pub struct Comm<'a> {
//     client: Client<HttpConnector, Body>,
//     events: &'a EventProcessor,
// }
//
// impl<'a> Comm<'a> {
//     pub fn new(handle: &Handle, events: &'a EventProcessor) -> Comm<'a> {
//         Comm {
//             client: Client::new(handle),
//             events: events,
//         }
//     }
//
//     //     fn add_10<F>(f: F) -> Map<F, fn(i32) -> i32>
//     //     where F: Future<Item = i32>,
//     // {
//     //     fn do_map(i: i32) -> i32 { i + 10 }
//     //     f.map(do_map)
//     // }
//
//     // Stream::Map<Stream<Item = Vec<Event>>, fn(Vec<Event>) -> ()>
//
//     // pub fn as_stream<
//     //     S: Stream<Item = Vec<Event>, Error = &'static str>,
//     //     F: FnMut(Vec<Event>) -> Vec<Event>,
//     // >(
//     //     self,
//     //     endpoint: &str,
//     //     key: &str,
//     //     func: F,
//     // ) -> Map<EventProcessor, F> {
//     //     self.events
//     // }
//
//     fn map_stream(self, endpoint: &str, key: &str) -> Map<EventProcessor, fn(Vec<Event>) -> ()> {
//
//         self.events.map(|batch: Vec<Event>| ())
//     }
//
//     fn send_events(&self, endpoint: &str, key: &str, events: Vec<Event>) -> FutureResponse {
//
//         // let json = r#"{"library":"hyper"}"#;
//         // let uri = "http://httpbin.org/post".parse()?;
//         // let mut req = Request::new(Method::Post, uri);
//         // req.headers_mut().set(ContentType::json());
//         // req.headers_mut().set(ContentLength(json.len() as u64));
//         // req.set_body(json);
//
//         // req.Header.Add("Authorization", ep.sdkKey)
//         // req.Header.Add("Content-Type", "application/json")
//         // req.Header.Add("User-Agent", "GoClient/"+Version)
//
//         // if let Ok(uri) = endpoint.parse() {
//         let uri = endpoint.parse().unwrap();
//         let mut req = Request::new(Method::Post, uri);
//         // req.headers_mut().set("Authorization", key);
//         req.headers_mut().set(Authorization(key.to_string()));
//         req.headers_mut().set(ContentType::json());
//         // req.headers_mut().set("User-Agent", "RustTest".to_string() + VERSION);
//
//         req.set_body("");
//
//         self.client.request(req)
//         // } else {
//         //
//         // }
//     }
// }
//
// // pub fn startup() {
// //
// //     let mut core = Core::new().unwrap();
// //     let address = "0.0.0.0:12345".parse().unwrap();
// //     let listener = TcpListener::bind(&address, &core.handle()).unwrap();
// //
// //     let connections = listener.incoming();
// //     let welcomes = connections.and_then(|(socket, _peer_addr)| {
// //         tokio_io::io::write_all(socket, b"Hello, world!\n")
// //     });
// //     let server = welcomes.for_each(|(_socket, _welcome)| {
// //         Ok(())
// //     });
// //
// //     core.run(server).unwrap();
// // }
