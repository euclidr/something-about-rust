extern crate futures;
extern crate hyper;
extern crate tokio;

use std::net::ToSocketAddrs;

use futures::future;

use hyper::rt::{self, Future};
use hyper::service::service_fn;
use hyper::upgrade::Upgraded;
use hyper::{Body, Client, Method, Request, Response, Server};

use tokio::io::{copy, shutdown};
use tokio::net::TcpStream;
use tokio::prelude::*;

// Create a TCP connection to host:port, build a tunnel between the connection and
// the upgraded connection
//
// refer to https://github.com/tokio-rs/tokio/blob/master/tokio/examples_old/proxy.rs
fn tunnel(upgraded: Upgraded, host: String, port: u16) {
    let remote_addr = format!("{}:{}", host, port);
    let remote_addr: Vec<_> = remote_addr
        .to_socket_addrs()
        .expect("Unable to resolve domain")
        .collect();

    let server = TcpStream::connect(&remote_addr[0]);

    let connector = server.and_then(move |server| {
        let (client_reader, client_writer) = upgraded.split();
        let (server_reader, server_writer) = server.split();

        let client_to_server = copy(client_reader, server_writer)
            .and_then(|(n, _, server_writer)| shutdown(server_writer).map(move |_| n));

        let server_to_client = copy(server_reader, client_writer)
            .and_then(|(n, _, client_writer)| shutdown(client_writer).map(move |_| n));

        client_to_server.join(server_to_client)
    });

    let msg = connector
        .map(move |(from_client, from_server)| {
            println!(
                "client wrote {} bytes and received {} bytes",
                from_client, from_server
            );
        })
        .map_err(|e| {
            println!("tunnel error: {}", e);
        });

    hyper::rt::spawn(msg);
}

fn main() {
    let addr = ([127, 0, 0, 1], 8100).into();
    let client_main = Client::new();

    let service = move || {
        let client = client_main.clone();
        service_fn(move |req: Request<Body>| {
            println!("req: {:?}", req);

            if Method::CONNECT == req.method() {
                // When HTTP method is CONNECT we should return an empty body
                // then we can eventually upgrade the connection and talk a new protocal.
                //
                // Note: only after client recieved an empty body with STATUS_OK can the
                // connection be upgraded, so we can't return a response inside
                // `on_upgrade` future.
                let host = req.uri().host().unwrap().to_string();
                let port = req.uri().port_u16().unwrap();
                let on_upgrade = req
                    .into_body()
                    .on_upgrade()
                    .map_err(|err| {
                        eprintln!("upgrade error: {}", err);
                    })
                    .map(move |upgraded| {
                        tunnel(upgraded, host, port);
                    });

                rt::spawn(on_upgrade);

                future::Either::A(future::ok(Response::new(Body::empty())))
            } else {
                future::Either::B(client.request(req))
            }
        })
    };

    let server = Server::bind(&addr)
        .serve(service)
        .map_err(|e| eprintln!("server error: {}", e));

    rt::run(server);
}
