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
extern crate docopt;
extern crate rustc_serialize;

#[macro_use]
extern crate log;
extern crate env_logger;

use line::{Client, Line, LineStream};

use futures::{future, Future, Stream, Sink};
use tokio_core::reactor::{Core, Timeout, Handle, Remote};
use tokio_service::{Service, NewService};

use std::{io, thread};
use std::io::{Error, ErrorKind};
use std::time::Duration;
use std::env;

use docopt::Docopt;


const USAGE: &'static str = "
Debug.

Usage:
  error_handling_debug [--server]

Options:
  -h --help     Show this screen.
  --server      Start as a server.
";

    
#[derive(Debug, RustcDecodable)]
struct Args {
    flag_server: bool,
}

pub fn main() {
    env_logger::init().unwrap();

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    let addr = "127.0.0.1:12345";
    if args.flag_server {
        server(addr);
    } else {
        client(addr);
    }
}

struct ServerService {
    remote: Remote,
}

impl ServerService {
    pub fn new(remote: Remote) -> ServerService {
        ServerService {
            remote: remote
        }
    }
}

impl Service for ServerService {
    type Request = Line;
    type Response = Line;
    type Error = io::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    /// Process the request and return the response asynchronously.
    fn call(&self, req: Self::Request) -> Self::Future {
        if let &Some(ref handle) = &self.remote.handle() {
            return Timeout::new(Duration::from_secs(10), handle)
                .unwrap()
                .and_then(|()| {
                    //future::done(Ok(Line::Once("Ok".to_string())))
                    trace!("intentionally failing");

                    future::failed(io::Error::new(io::ErrorKind::Other, "intentional fail"))
                })
                .boxed()
        } else {
            panic!("cannot unwrap handle");
        }
    }
}

struct ServerServiceFactory {
    remote: Remote,
}

impl ServerServiceFactory {
    pub fn new(handle: &Handle) -> ServerServiceFactory {
        ServerServiceFactory {
            remote: handle.remote().clone()
        }
    }
}

impl NewService for ServerServiceFactory {
    /// Requests handled by the service
    type Request = Line;
    type Response = Line;
    type Error = io::Error;
    type Instance = ServerService;

    /// Create and return a new service value.
    fn new_service(&self) -> io::Result<Self::Instance> {
        Ok(ServerService::new(self.remote.clone()))
    }
}

fn server(addr: &str) {
    // This brings up our server.
    let addr = addr.parse().unwrap();

    println!("starting server");
    
    line::with_handle(
        addr,
        |handle: &Handle| {
            ServerServiceFactory::new(handle)
        });
}

fn client(addr: &str) {
    let addr = addr.parse().unwrap();

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    println!("starting client");

    core.run(
        Client::connect(&addr, &handle)
            .and_then(|client| {
                client.call(Line::Once("Hello".to_string()))
                    .then(|res| {
                        println!("CLIENT: result: {:?}", res);

                        Ok(())
                    })
            })
    ).unwrap();
}
