extern crate http;

use crate::store::Yield;
use crate::syncer::{is_synced, sync_year};
use crate::{ResponseFuture, YCResult};

use chrono::prelude::*;
use futures::{future, Future};
use http::header::HOST;
use http::StatusCode;
use hyper::{Body, Request, Response};
use serde::Serialize;
use serde_json::{self, json};
use std::collections::HashMap;
use std::convert::From;
use time::Duration;
use url::Url;

/// get query params from request
fn get_query_params(req: &Request<Body>) -> YCResult<HashMap<String, String>> {
    let uri_string = req.uri().to_string();
    // uri_string does not contain http://a.b.c
    // so we need to complete it into full URL
    let host = match req.headers().get(HOST) {
        Some(host_header) => host_header.to_str()?,
        None => "localhost",
    };
    let url_string = format!("http://{}{}", host, uri_string);
    let request_url = Url::parse(&url_string)?;
    Ok(request_url.query_pairs().into_owned().collect())
}

fn abort(status: StatusCode) -> Response<hyper::Body> {
    let status_text = status.canonical_reason().unwrap_or("unknown status");
    let resp = Response::builder()
        .status(status)
        .body(Body::from(status_text))
        .unwrap();
    resp
}

fn ok_response<T: ?Sized>(data: &T) -> Response<hyper::Body>
where
    T: Serialize,
{
    let result = json!({
        "code": 0,
        "data": data
    });
    api_response(&result)
}

fn err_response(code: i32, msg: &str) -> Response<hyper::Body> {
    let result = json!({
        "code": code,
        "msg": msg
    });
    api_response(&result)
}

fn api_response<T: ?Sized>(data: &T) -> Response<hyper::Body>
where
    T: Serialize,
{
    let result = serde_json::to_string(&data).unwrap_or("{\"code\": 100}".to_string());
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(result.to_string()))
        .unwrap()
}

fn future_abort(status: StatusCode) -> ResponseFuture {
    Box::new(future::ok(abort(status)))
}

fn future_ok_response<T: ?Sized>(data: &T) -> ResponseFuture
where
    T: Serialize,
{
    Box::new(future::ok(ok_response(data)))
}

fn future_err_response(code: i32, msg: &str) -> ResponseFuture {
    Box::new(future::ok(err_response(code, msg)))
}

/// handle path: /yield_by_date?date=<%Y-%m-%d>
///
/// returns bond yields on that day
///
pub fn handle_by_date(req: Request<Body>) -> ResponseFuture {
    let params = match get_query_params(&req) {
        Ok(value) => value,
        Err(_) => return future_abort(StatusCode::BAD_REQUEST),
    };
    let mut date = match params.get("date") {
        Some(val) => match NaiveDate::parse_from_str(val, "%Y-%m-%d") {
            Ok(date) => date,
            Err(_) => return future_abort(StatusCode::BAD_REQUEST),
        },
        None => return future_abort(StatusCode::BAD_REQUEST),
    };

    date = match date.weekday() {
        Weekday::Thu => date + Duration::days(-1),
        Weekday::Sun => date + Duration::days(-2),
        _ => date,
    };

    let date = date.format("%Y-%m-%d").to_string();
    match Yield::get(&date) {
        Some(val) => return future_ok_response(&val),
        None => (),
    };

    if is_synced(&date) {
        return future_err_response(1, "no data");
    }

    let rs = sync_year(&date[..4])
        .then(move |result| match result {
            Ok(_) => match Yield::get(&date) {
                Some(val) => future::ok(ok_response(&val)),
                None => future::ok(err_response(1, "no data")),
            },
            Err(_) => future::ok(abort(StatusCode::INTERNAL_SERVER_ERROR)),
            _ => future::err("not exist"),
        })
        .from_err();

    Box::new(rs)
}

#[allow(dead_code)]
pub fn handle_by_term() {}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use chrono::prelude::*;
    use std::collections::BTreeMap;
    use std::ops::Bound::{Excluded, Included};

    #[test]
    fn parse_date() {
        let result = NaiveDate::parse_from_str("2007-01-03", "%Y-%m-%d");
        let expect = NaiveDate::from_ymd(2007, 1, 3);
        assert_eq!(result, Ok(expect));
    }

    #[test]
    fn parse_short_date() {
        let result = NaiveDate::parse_from_str("03/01/07", "%d/%m/%y");
        let expect = NaiveDate::from_ymd(2007, 1, 3);
        assert_eq!(result, Ok(expect));
    }

    #[test]
    fn range_search() {
        let mut map = BTreeMap::new();
        map.insert("1".to_string(), "1".to_string());
        map.insert("2".to_string(), "2".to_string());
        map.insert("3".to_string(), "3".to_string());
        map.insert("4".to_string(), "4".to_string());
        map.insert("5".to_string(), "5".to_string());
        map.insert("6".to_string(), "6".to_string());
        for (key, value) in
            map.range::<String, _>((Included(&"2".to_string()), Excluded(&"5".to_string())))
        {
            println!("{}: {}", key, value);
        }
    }
}
