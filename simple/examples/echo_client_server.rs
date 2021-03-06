//! Using tokio-proto to get a request / response oriented client and server
//!
//! This example illustrates how to implement a client and server for a request
//! / response oriented protocol. The protocol is a simple line-based string
//! protocol. Each line represents a request or response.
//!
//! A transport is setup to handle framing the protocl to `String` messages.
//! Then the transport is passed to `tokio-proto` which handles the details of
//! managing the necessary state to implemnet `Service`.

extern crate tokio_line as line;

extern crate futures;
extern crate tokio_core;
extern crate tokio_service;
extern crate service_fn;

use futures::Future;
use tokio_core::reactor::Core;
use tokio_service::Service;
use service_fn::service_fn;
use std::thread;
use std::time::Duration;
use std::io;

pub fn main() {
    let mut core = Core::new().unwrap();

    // This brings up our server.
    let addr = "127.0.0.1:12345".parse().unwrap();

    thread::spawn(move || {
        line::serve(
            addr,
            || {
                Ok(service_fn(|msg| {
                    println!("SERVER: {:?}", msg);
                    //Ok(msg)
                    Err(io::Error::new(io::ErrorKind::Other, "service error"))
                }))
            });
    });

    // A bit annoying, but we need to wait for the server to connect
    thread::sleep(Duration::from_millis(100));

    let handle = core.handle();

    core.run(
        line::Client::connect(&addr, &handle)
            .and_then(|client| {
                client.call("Hello".to_string())
                    .and_then(move |response| {
                        println!("CLIENT: {:?}", response);
                        client.call("Goodbye".to_string())
                    })
                    .and_then(|response| {
                        println!("CLIENT: {:?}", response);
                        Ok(())
                    })
            })
    ).unwrap();
}
