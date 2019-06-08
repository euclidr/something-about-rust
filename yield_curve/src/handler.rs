extern crate http;

use crate::yc_error;

use chrono::prelude::*;
use chrono::prelude::*;
use futures::{future, Future, Stream};
use http::header::HOST;
use hyper::{Body, Request, Response};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::convert::From;
use std::sync::{Arc, Mutex};
use std::fmt;
use std::error::Error;
use std::error;
use time::Duration;
use url::Url;

use hyper::client::HttpConnector;
use hyper::Client;
use hyper_tls::HttpsConnector;

use select::document::Document;
use select::node::Node;
use select::predicate::{Class, Name, Predicate}

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

static BOND_SUB_URL: &str = "https://www.treasury.gov/resource-center/data-chart-center/interest-rates/pages/TextView.aspx?data=yield";

fn get_query_params(req: &Request<Body>) -> Result<HashMap<String, String>, GenericError> {
    let uri_string = req.uri().to_string();
    let host = match req.headers().get(HOST) {
        Some(host_header) => host_header.to_str()?,
        None => "localhost",
    };
    let url_string = format!("http://{}{}", host, uri_string);
    let request_url = Url::parse(&url_string).unwrap();
    Ok(request_url.query_pairs().into_owned().collect())
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

fn extract_yield_data(text: &str) -> Result<Vec<HashMap<String, String>>, GenericError> {
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

    Ok(data)
}

fn get_remote_bond_yield(client: &HttpsClient, bondData: &ShareDict, date: &str) -> Box<Future<Item=String, Error = GenericError> + Send> {
    let year = &date[..4];
    let urlString = format!("{}&year={}", BOND_SUB_URL, year);
    let uri = urlString.parse().unwrap();
    let r = client
        .get(uri)
        .map_err(|e| XError { msg: e.to_string() })
        .and_then(|resp| {
            if !resp.status().is_success() {
                return future::Either::A(future::err(XError {
                    msg: "Invalid code".to_string(),
                }));
            };
            future::Either::B(
                resp.into_body()
                    .concat2()
                    .map_err(|e| XError { msg: e.to_string() })
                    .map(|chunk| String::from(std::str::from_utf8(&chunk).unwrap())),
            )
        })
        .and_then(|text| match extract_yield_data(&text) {
            Ok(data_str) => future::ok(data_str),
            Err(err) => future::err(XError {
                msg: err.to_string(),
            }),
        })
        .then(|result| {
            match result {
                Ok(data) => future::ok(Response::new(Body::from(data))),
                Err(e) => future::ok(Response::new(Body::from(e.to_string()))),
                // #[warn(unreachable_patterns)]
                _ => future::err(XError {
                    msg: "will not happen".to_string(),
                }),
            }
        })
        .from_err();

    Box::new(r)

} 

pub fn handle_by_date(
    req: Request<Body>,
    client: &HttpsClient,
    bondData: &ShareDict,
) -> ResponseFuture {
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
    // let mut date = date.format("%Y-%m-%d").to_string();
    date = match date.weekday() {
        Weekday::Thu => date + Duration::days(-1),
        Weekday::Sun => date + Duration::days(-2),
        _ => date,
    };

    let date = date.format("%Y-%m-%d").to_string();
    let result = {
        let bd = bondData.lock().unwrap();
        match bd.get(&date) {
            Some(val) => Some(val.clone()),
            None => None,
        }
    };
    if result.is_some() {
        let val = result.unwrap();
        return Box::new(future::ok(Response::new(Body::from(val))));
    }

    let rs = get_remote_bond_yield(client, bondData, &date).map(|val| {
        Response::new(Body::from(val))
    }).from_err();

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
