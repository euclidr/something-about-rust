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
extern crate tokio;

mod handler;
mod request;
mod sharekv;
mod store;
mod syncer;
mod ycerror;

use chrono::prelude::*;
use chrono::Utc;
use futures::prelude::*;
use futures::stream::iter_ok;
use futures::{future, Future};
use tokio::timer::Interval;
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::time::{Duration, Instant};

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

/// sync history data
/// now it's synced with future stream, it's slow
/// consider spawn a job per year simultaneously
/// it must be called inside future runtime
fn sync_history_data() {
    let mut years = vec![];
    let cur = Utc::now();
    for year in 1990..(cur.year()+1) {
        years.push(format!("{}", year));
    }

    let startup_sync = iter_ok(years).for_each(|item| {
        syncer::sync_year(&item).then(|result| match result {
            Ok(_) => future::ok(()),
            Err(err) => {
                println!("sync error: {}", err.to_string());
                future::ok(())
            }
        })
    });

    hyper::rt::spawn(startup_sync);
}

/// sync latest data periodically
fn periodic_sync_data() {
    let cron = Interval::new(Instant::now(), Duration::from_secs(3600)).for_each(|a| {
        let today = Utc::now();
        let today_str = today.format("%Y-%m-%d").to_string();
        if !syncer::is_synced(&today_str) {
            let job = syncer::sync_year(&today.year().to_string()).map_err(|e| {
                println!("sync error: {}", e.to_string());
            });
            hyper::rt::spawn(job);
        }
        Ok(())
    }).map_err(|e| {
        println!("periodic sync error: {}", e.to_string());
    });

    hyper::rt::spawn(cron);
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

        sync_history_data();

        periodic_sync_data();

        server
    }))
}
