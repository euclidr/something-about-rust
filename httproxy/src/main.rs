extern crate futures;
extern crate hyper;

use futures::future;
use hyper::header::{HeaderValue, UPGRADE};
use hyper::rt::{self, Future};
use hyper::service::service_fn;
use hyper::upgrade::Upgraded;
use hyper::{Body, Client, Method, Request, Response, Server, StatusCode};
use std::io::{self, Read, Write};
use std::net::{Shutdown, SocketAddr, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use tokio::io::{copy, shutdown};
use tokio::net::TcpStream;
use tokio::prelude::*;

// https://github.com/tokio-rs/tokio/blob/master/tokio/examples_old/proxy.rs
fn tcp_proxy(upgraded: Upgraded, host: String, port: u16) {
    let remote_addr = format!("{}:{}", host, port);
    let remote_addr: Vec<_> = remote_addr
        .to_socket_addrs()
        .expect("Unable to resolve domain")
        .collect();
    let server = TcpStream::connect(&remote_addr[0]);
    let amounts = server.and_then(move |server| {
        println!("adfasdfasdf-----{:?}", server);
        println!("upgraded-----{:?}", upgraded);
        let client_reader = MyTcpStream(Arc::new(Mutex::new(upgraded)));
        let client_writer = client_reader.clone();
        let server_reader = MyTcpStream(Arc::new(Mutex::new(server)));
        let server_writer = server_reader.clone();

        // let (client_reader, client_writer) = upgraded.split();
        // let (server_reader, server_writer) = server.split();

        let client_to_server = copy(client_reader, server_writer)
            .and_then(|(n, _, server_writer)| shutdown(server_writer).map(move |_| n));

        let server_to_client = copy(server_reader, client_writer)
            .and_then(|(n, _, client_writer)| shutdown(client_writer).map(move |_| n));

        client_to_server.join(server_to_client)
    });

    let msg = amounts
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
                let host = req.uri().host().unwrap().to_string();
                let port = req.uri().port_u16().unwrap();
                let on_upgrade = req
                    .into_body()
                    .on_upgrade()
                    .map_err(|err| {
                        eprintln!("upgrade error: {}", err);
                    })
                    .map(move |upgraded| {
                        tcp_proxy(upgraded, host, port);
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

// This is a custom type used to have a custom implementation of the
// `AsyncWrite::shutdown` method which actually calls `TcpStream::shutdown` to
// notify the remote end that we're done writing.
struct MyTcpStream<T>(Arc<Mutex<T>>);

impl<T> Clone for MyTcpStream<T> {
    fn clone(&self) -> MyTcpStream<T> {
        MyTcpStream(self.0.clone())
    }
}

impl<T> Read for MyTcpStream<T>
where
    T: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let rs = self.0.lock().unwrap().read(buf);
        // println!("read: {:?}", buf);
        rs
    }
}

impl<T> Write for MyTcpStream<T>
where
    T: Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let rs = self.0.lock().unwrap().write(buf);
        // println!("write: {:?}", buf);
        rs
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<T> AsyncRead for MyTcpStream<T> where T: AsyncRead {}

impl<T> AsyncWrite for MyTcpStream<T>
where
    T: AsyncWrite,
{
    fn shutdown(&mut self) -> Poll<(), io::Error> {
        println!("closed");
        self.0.lock().unwrap().shutdown()?;
        Ok(().into())
    }
}
