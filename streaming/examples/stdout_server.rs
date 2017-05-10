//! Using tokio-proto to get a request / response oriented client and server
//!
//! This example illustrates how to implement a client and server for a request
//! / response oriented protocol. The protocol is a simple line-based string
//! protocol. Each line represents a request or response.
//!
//! A transport is setup to handle framing the protocl to `String` messages.
//! Then the transport is passed to `tokio-proto` which handles the details of
//! managing the necessary state to implemnet `Service`.

extern crate tokio_line_streaming as line;

extern crate futures;
extern crate tokio_core;
extern crate tokio_service;
extern crate service_fn;

#[macro_use]
extern crate log;
extern crate env_logger;

use line::{Client, Line, LineStream};

use futures::{future, Future, Stream, Sink};
use tokio_core::reactor::Core;
use tokio_service::Service;
use service_fn::service_fn;

use std::{io, thread};
use std::io::{Error, ErrorKind};
use std::time::Duration;

pub fn main() {
    env_logger::init().unwrap();

    let mut core = Core::new().unwrap();

    // This brings up our server.
    let addr = "127.0.0.1:12345".parse().unwrap();

    thread::spawn(move || {
        line::serve(
            addr,
            || {
                Ok(service_fn(|msg| {
                    match msg {
                        Line::Once(line) => {
                            println!("{}", line);
                            //Box::new(future::done(Ok(Line::Once("Ok".to_string()))))
                            Box::new(future::result(Err(Error::new(ErrorKind::Other, "service error"))))
                        }
                        Line::Stream(body) => {
                            let resp = body
                                .for_each(|line| {
                                    println!(" + {}", line);
                                    Ok(())
                                })
                                .map(|_| Line::Once("Ok".to_string()));

                            Box::new(resp) as Box<Future<Item = Line, Error = io::Error>>
                        }
                    }
                }))
            });
    });

    // A bit annoying, but we need to wait for the server to connect
    thread::sleep(Duration::from_millis(100));

    let handle = core.handle();

    core.run(
        Client::connect(&addr, &handle)
            .and_then(|client| {
                client.call(Line::Once("Hello".to_string()))
                    .then(|res| {
                        println!("got some result");

                        res
                    })
                    .and_then(move |response| {
                        println!("CLIENT: {:?}", response);

                        // Now, spawn a thread that streams to the server

                        let (mut tx, rx) = LineStream::pair();

                        thread::spawn(move || {
                            for msg in &["one", "two", "three", "four"] {
                                thread::sleep(Duration::from_millis(500));
                                tx = tx.send(Ok(msg.to_string())).wait().unwrap();
                            }
                        });

                        client.call(Line::Stream(rx))
                    })
                    .and_then(|response| {
                        println!("CLIENT: {:?}", response);
                        Ok(())
                    })
            })
    ).unwrap();
}

