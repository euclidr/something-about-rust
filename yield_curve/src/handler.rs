extern crate http;

use crate::request;
use crate::sharekv;
use crate::ycerror::YCError;
use crate::{ResponseFuture, YCFuture, YCResult};

use chrono::prelude::*;
use futures::{future, Future};
use http::header::HOST;
use http::StatusCode;
use hyper::{Body, Request, Response};
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

fn get_remote_bond_yield(date: String) -> YCFuture<String> {
    let year = &date[..4];
    let r = request::yield_of_year(year).and_then(move |data| {
        for date_record in data {
            let date = date_record.get("Date").unwrap();
            let date = NaiveDate::parse_from_str(&date, "%m/%d/%y").unwrap();
            let date = date.format("%Y-%m-%d").to_string();
            sharekv::set(&date, &serde_json::to_string(&date_record).unwrap());
        }
        match sharekv::get(&date) {
            Some(val) => future::ok(val),
            None => future::err(YCError::DataNotFound(date.clone())),
        }
    });
    Box::new(r)
}

fn response_with_status(status: StatusCode, body: &str) -> ResponseFuture {
    Box::new(future::ok(
        Response::builder()
            .status(status)
            .body(Body::from(body.to_string()))
            .unwrap(),
    ))
}

/// handle path: /yield_by_date?date=<%Y-%m-%d>
///
/// returns bond yields on that day
///
pub fn handle_by_date(req: Request<Body>) -> ResponseFuture {
    let params = match get_query_params(&req) {
        Ok(value) => value,
        Err(e) => return response_with_status(StatusCode::BAD_REQUEST, &e.to_string()),
    };
    let mut date = match params.get("date") {
        Some(val) => match NaiveDate::parse_from_str(val, "%Y-%m-%d") {
            Ok(date) => date,
            Err(e) => return response_with_status(StatusCode::BAD_REQUEST, &e.to_string()),
        },
        None => return response_with_status(StatusCode::NOT_FOUND, "not found"),
    };

    date = match date.weekday() {
        Weekday::Thu => date + Duration::days(-1),
        Weekday::Sun => date + Duration::days(-2),
        _ => date,
    };

    let date = date.format("%Y-%m-%d").to_string();
    let result = {
        match sharekv::get(&date) {
            Some(val) => Some(val),
            None => None,
        }
    };

    if result.is_some() {
        let val = result.unwrap();
        return Box::new(future::ok(Response::new(Body::from(val))));
    }

    let rs = get_remote_bond_yield(date)
        .map(|val| Response::new(Body::from(val)))
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
