extern crate http;

use crate::client;
use crate::sharekv;
use crate::ycerror;

use ycerror::YCError;

use chrono::prelude::*;
use futures::{future, Future, Stream};
use http::header::HOST;
use hyper::{Body, Request, Response};
use std::collections::HashMap;
use std::convert::From;
use time::Duration;
use url::Url;

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type ResponseFuture = Box<Future<Item = Response<Body>, Error = GenericError> + Send>;

fn get_query_params(req: &Request<Body>) -> Result<HashMap<String, String>, GenericError> {
    let uri_string = req.uri().to_string();
    let host = match req.headers().get(HOST) {
        Some(host_header) => host_header.to_str()?,
        None => "localhost",
    };
    let url_string = format!("http://{}{}", host, uri_string);
    println!("{}", &url_string);
    let request_url = Url::parse(&url_string).unwrap();
    Ok(request_url.query_pairs().into_owned().collect())
}

fn get_remote_bond_yield(date: String) -> Box<Future<Item = String, Error = YCError> + Send> {
    let year = &date[..4];
    let r = client::yield_of_year(year).and_then(move |data| {
        for dateRecord in data {
            let date = dateRecord.get("Date").unwrap();
            let date = NaiveDate::parse_from_str(&date, "%m/%d/%y").unwrap();
            let date = date.format("%Y-%m-%d").to_string();
            sharekv::set(&date, &serde_json::to_string(&dateRecord).unwrap());
        }
        match sharekv::get(&date) {
            Some(val) => future::ok(val),
            None => future::err(YCError::DataNotFound(date.clone())),
        }
    });
    Box::new(r)
}

pub fn handle_by_date(req: Request<Body>) -> ResponseFuture {
    let params = match get_query_params(&req) {
        Ok(value) => value,
        Err(e) => return Box::new(future::ok(Response::new(Body::from(e.to_string())))),
    };
    let mut date = match params.get("date") {
        Some(val) => match NaiveDate::parse_from_str(val, "%Y-%m-%d") {
            Ok(date) => date,
            Err(e) => return Box::new(future::err(From::from(e))),
        },
        None => return Box::new(future::err(From::from("not found"))),
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

pub fn handle_by_term() {}

#[cfg(test)]
mod tests {
    use super::*;
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
