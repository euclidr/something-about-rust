extern crate chrono;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate select;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate url;
#[macro_use]
extern crate lazy_static;
extern crate futures_timer;

mod handler;
mod request;
mod sharekv;
mod store;
mod syncer;
mod ycerror;

use chrono::prelude::*;
use futures::prelude::*;
use futures::{future, Future};
use futures_timer::Interval;
use futures::stream::iter_ok;
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::time::Duration;

use ycerror::YCError;

static NOTFOUND: &[u8] = b"Not Found";
static INDEX: &[u8] = b"<a href=\"bond_yield\">bond_yield</a>";

pub type GenericError = Box<dyn std::error::Error + Send + Sync>;
pub type ResponseFuture = Box<Future<Item = Response<Body>, Error = GenericError> + Send>;
pub type YCFuture<T> = Box<Future<Item = T, Error = YCError> + Send>;
pub type YCResult<T> = Result<T, YCError>;

fn route(req: Request<Body>) -> ResponseFuture {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") | (&Method::GET, "/index.html") => {
            let body = Body::from(INDEX);
            Box::new(future::ok(Response::new(body)))
        }
        (&Method::GET, "/yield_by_date") => handler::handle_by_date(req),
        _ => {
            let body = Body::from(NOTFOUND);
            Box::new(future::ok(
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(body)
                    .unwrap(),
            ))
        }
    }
}

fn main() {
    let addr = "127.0.0.1:2008".parse().unwrap();
    sharekv::init("kv.db");

    hyper::rt::run(future::lazy(move || {
        let new_service = move || service_fn(move |req| route(req));

        let server = Server::bind(&addr)
            .serve(new_service)
            .map_err(|e| eprintln!("serve error: {}", e));

        println!("Listening on http://{}", addr);

        let mut years = vec![];
        let cur = Utc::now();
        for year in 1991..cur.year() {
            years.push(format!("{}", year));
        }

        let startup_sync = iter_ok(years).for_each(|item| {
            syncer::sync_year(&item).then(|result| match result {
                Ok(_) => {
                    println!("synced");
                    future::ok(())
                }
                Err(err) => {
                    println!("error: {}", err.to_string());
                    future::ok(())
                }
            })
        });

        hyper::rt::spawn(startup_sync);

        // let cron = future::ok(1).and_then(|_| {
        //     Interval::new(Duration::from_secs(5))
        //         .for_each(|()| {
        //             println!("hahah");
        //             Ok(())
        //         })
        //         .wait()
        //         .unwrap_or(()); // ignore the error;
        //     future::ok(())
        // });

        // hyper::rt::spawn(cron);

        server
    }))
}
