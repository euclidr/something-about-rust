extern crate hyper;

use hyper::{Client, Server, Request, Response, Body, Method, StatusCode};
use hyper::header::{UPGRADE ,HeaderValue};
use hyper::service::service_fn;
use hyper::rt::{self, Future};

fn main() {
    let addr = ([127, 0, 0, 1], 8100).into();
    let client_main = Client::new();

    let service = move || {
        let client = client_main.clone();
        service_fn(move |mut req| {
            println!("req: {:?}", req);
            // let uri = format!("http://{}{}",req.headers().get("host").unwrap().to_str().unwrap(), req.uri().path_and_query().map(|x| x.as_str()).unwrap());
            // *req.uri_mut() = uri.parse().unwrap();
            if Method::CONNECT == req.method() {
                let on_upgrade = req.into_body().on_upgrade().map_err(|err| eprintln!("upgrade error: {}", err))
                .and_then(|updgraded| {
                    // TODO
                    unimplemented!()
                };
                rt::spawn(on_upgrade);
                let mut res = Response::new(Body::empty());
                *res.status_mut() = StatusCode::SWITCHING_PROTOCOLS;
                res.header_mut().insert(UPGRADE, HeaderValue::from_static("fooo"));
                res
            } else {
                client.request(req)
            }
        })
    };

    let server = Server::bind(&addr).serve(service).map_err(|e| eprintln!("server error: {}", e));

    rt::run(server);
}
