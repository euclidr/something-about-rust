extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate select;
extern crate serde_json;

use futures::{future, Future, Stream};
use std::collections::HashMap;

use hyper::service::service_fn;
use hyper::{Body, Client, Method, Request, Response, Server, StatusCode};
use hyper_tls::HttpsConnector;

use select::document::Document;
use select::predicate::{Class, Name, Predicate};

static NOTFOUND: &[u8] = b"Not Found";
static INDEX: &[u8] = b"<a href=\"bond_yield\">bond_yield</a>";
static BOND_URL: &str = "https://www.treasury.gov/resource-center/data-chart-center/interest-rates/pages/TextView.aspx?data=yield";

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type ResponseFuture = Box<Future<Item = Response<Body>, Error = GenericError> + Send>;

fn handle_bond_yield(_: Request<Body>) -> ResponseFuture {
    let https = HttpsConnector::new(4).unwrap();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let uri = BOND_URL.parse().unwrap();
    let r = client
        .get(uri)
        .map_err(|e| {
            println!("{}", e);
            e
        })
        .from_err()
        .and_then(|res| {
            res.into_body()
                .concat2()
                .map_err(|err| {
                    println!("{}", err);
                    err
                })
                .map(|chunk| {
                    // println!("dddhsdslsdfs----{}", std::str::from_utf8(&chunk));
                    // TODO handle error
                    String::from(std::str::from_utf8(&chunk).unwrap())
                })
                .from_err()
        })
        .map(|text| {
            let document = Document::from(&text[..]);
            let mut tcharts_result = document.find(Class("t-chart"));
            let table = tcharts_result.next();
            let mut cates = HashMap::new();
            match table {
                Some(table) => {
                    for th in table.find(Name("th")) {
                        cates.insert(th.text(), 1);
                    }
                }
                None => (),
            }
            let fin = serde_json::to_string(&cates).unwrap();
            Response::new(Body::from(fin))
        });

    Box::new(r)
}

fn route(req: Request<Body>) -> ResponseFuture {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") | (&Method::GET, "/index.html") => {
            let body = Body::from(INDEX);
            Box::new(future::ok(Response::new(body)))
        }
        (&Method::GET, "/bond_yield") => handle_bond_yield(req),
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

    hyper::rt::run(future::lazy(move || {
        let new_service = move || service_fn(move |req| route(req));

        let server = Server::bind(&addr)
            .serve(new_service)
            .map_err(|e| eprintln!("serve error: {}", e));

        println!("Listening on http://{}", addr);

        server
    }))
}
