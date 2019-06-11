extern crate chrono;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate select;
extern crate serde_json;
extern crate url;
#[macro_use]
extern crate lazy_static;

mod handler;
mod request;
mod sharekv;
mod ycerror;
mod store;

use futures::{future, Future};
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};

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

        server
    }))
}
