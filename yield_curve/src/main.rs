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
mod ycerror;
mod sharekv;
mod client;

use futures::{future, Future, Stream};
use std::collections::{BTreeMap, HashMap};
use std::convert::From;
use std::error;
use std::error::Error;
use std::fmt;
use std::sync::{Arc, Mutex};

use hyper::client::HttpConnector;
use hyper::service::service_fn;
use hyper::{Body, Client, Method, Request, Response, Server, StatusCode};
use hyper_tls::HttpsConnector;

use select::document::Document;
use select::node::Node;
use select::predicate::{Class, Name, Predicate};

static NOTFOUND: &[u8] = b"Not Found";
static INDEX: &[u8] = b"<a href=\"bond_yield\">bond_yield</a>";
static BOND_URL: &str = "https://www.treasury.gov/resource-center/data-chart-center/interest-rates/pages/TextView.aspx?data=yield";

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type ResponseFuture = Box<Future<Item = Response<Body>, Error = GenericError> + Send>;
type HttpsClient = Client<HttpsConnector<HttpConnector>>;
type ShareDict = Arc<Mutex<BTreeMap<String, String>>>;

#[derive(Debug, Clone)]
struct XError {
    msg: String,
}

impl fmt::Display for XError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl error::Error for XError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
    fn description(&self) -> &str {
        &self.msg
    }
}

fn extract_table_keys(row: &Node) -> Vec<String> {
    let mut keys = vec![];
    for (_, th) in row.find(Name("th")).enumerate() {
        keys.push(th.text())
    }
    keys
}

fn extract_row(row: &Node, keys: &Vec<String>) -> Option<HashMap<String, String>> {
    if keys.len() == 0 {
        return None;
    }

    let cnt = row.find(Name("td")).count();
    if cnt != keys.len() {
        return None;
    }

    let mut record = HashMap::new();
    for (i, td) in row.find(Name("td")).enumerate() {
        record.insert(keys[i].clone(), td.text());
    }

    Some(record)
}

fn extract_yield_data(text: &str) -> Result<String, GenericError> {
    let document = Document::from(&text[..]);
    let mut tcharts_result = document.find(Class("t-chart"));
    let table = tcharts_result.next();
    let data = match table {
        Some(table) => {
            let trs = table.find(Name("tr"));
            let mut keys = vec![];
            let mut data = vec![];
            for (i, row) in trs.enumerate() {
                if i == 0 {
                    keys = extract_table_keys(&row);
                } else {
                    match extract_row(&row, &keys) {
                        Some(record) => data.push(record),
                        None => continue,
                    }
                }
            }
            data
        }
        None => {
            return Err(From::from(XError {
                msg: "table notfound".to_string(),
            }))
        }
    };

    let data_str = serde_json::to_string(&data)?;
    Ok(data_str)
}

// fn handle_bond_yield(_: Request<Body>, client: &HttpsClient) -> ResponseFuture {
//     let uri = BOND_URL.parse().unwrap();
//     let r = client
//         .get(uri)
//         .map_err(|e| XError { msg: e.to_string() })
//         .and_then(|resp| {
//             if !resp.status().is_success() {
//                 return future::Either::A(future::err(XError {
//                     msg: "Invalid code".to_string(),
//                 }));
//             };
//             future::Either::B(
//                 resp.into_body()
//                     .concat2()
//                     .map_err(|e| XError { msg: e.to_string() })
//                     .map(|chunk| String::from(std::str::from_utf8(&chunk).unwrap())),
//             )
//         })
//         .and_then(|text| match extract_yield_data(&text) {
//             Ok(data_str) => future::ok(data_str),
//             Err(err) => future::err(XError {
//                 msg: err.to_string(),
//             }),
//         })
//         .then(|result| {
//             match result {
//                 Ok(data) => future::ok(Response::new(Body::from(data))),
//                 Err(e) => future::ok(Response::new(Body::from(e.to_string()))),
//                 // #[warn(unreachable_patterns)]
//                 _ => future::err(XError {
//                     msg: "will not happen".to_string(),
//                 }),
//             }
//         })
//         .from_err();

//     Box::new(r)
// }

fn route(req: Request<Body>) -> ResponseFuture {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") | (&Method::GET, "/index.html") => {
            let body = Body::from(INDEX);
            Box::new(future::ok(Response::new(body)))
        }
        // (&Method::GET, "/bond_yield") => handle_bond_yield(req),
        (&Method::GET, "/bond_by_date") => handler::handle_by_date(req),
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
        let new_service = move || {
            service_fn(move |req| route(req))
        };

        let server = Server::bind(&addr)
            .serve(new_service)
            .map_err(|e| eprintln!("serve error: {}", e));

        println!("Listening on http://{}", addr);

        server
    }))
}
