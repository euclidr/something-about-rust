use futures::{future, Future, Stream};
use hyper::client::HttpConnector;
use hyper::{Body, Client};
use hyper_tls::HttpsConnector;
use std::collections::HashMap;

use select::document::Document;
use select::node::Node;
use select::predicate::{Class, Name};

use std::fs::File;
use std::io::prelude::*;

use crate::ycerror::YCError;
use crate::YCFuture;

type YCClient = Client<HttpsConnector<HttpConnector>>;

lazy_static! {
    static ref CLIENT: YCClient = {
        let https = HttpsConnector::new(4).unwrap();
        Client::builder().build::<_, Body>(https)
    };
}

/// Get an HTTPS client
pub fn cli() -> YCClient {
    CLIENT.clone()
}

/// Make a GET HTTP request, return response body as String
///
/// # Errors
///
/// Returns future error when:
///     url_str is not valid
///     error occurs in connecting the server
///     responsed status code is not StatusCode::Ok
///     invalid response text
///     
pub fn get(url_str: &str) -> YCFuture<String> {
    const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/75.0.3770.80 Safari/537.36";

    let mut req = hyper::Request::builder();
    req.uri(url_str).header("User-Agent", USER_AGENT);
    let req = req.body(Body::from("")).unwrap();

    let r = cli()
        .request(req)
        .map_err(|err| YCError::Hyper(err))
        .and_then(|resp| {
            if !resp.status().is_success() {
                return future::Either::A(future::err(YCError::InvalidResponse(format!(
                    "status code: {}",
                    resp.status()
                ))));
            };
            future::Either::B(
                resp.into_body()
                    .concat2()
                    .map_err(|err| YCError::Hyper(err))
                    .map(|chunk| String::from(std::str::from_utf8(&chunk).unwrap())),
            )
        });

    Box::new(r)
}

/// Get bond yield data from website.
///
/// The result is an array of HashMap, contents in HashMap looks like bellow:
///     {
///         "date": "02/01/17",
///         "1 mo": "N/A",
///         "3 mo": "1.7",
///         "1 yr": "2.3",
///         ...
///     }
///
pub fn yield_of_year(
    year: &str,
) -> Box<Future<Item = Vec<HashMap<String, String>>, Error = YCError> + Send> {
    const SUB_URL: &str = "https://www.treasury.gov/resource-center/data-chart-center/interest-rates/pages/TextView.aspx?data=yieldYear";

    let url_str = format!("{}&year={}", SUB_URL, year);
    let year_str = year.to_string();
    let r = get(&url_str).and_then(move |text| match extract_yield_data(&text) {
        Ok(data) => future::ok(data),
        Err(err) => {
            // write down the result for later analysing
            let mut file = File::create(&format!("error_page_{}.html", &year_str)).unwrap();
            file.write_all(text.as_bytes()).unwrap();
            future::err(err)
        }
    });
    Box::new(r)
}

fn extract_column_names(row: &Node) -> Vec<String> {
    let mut keys = vec![];
    for (_, th) in row.find(Name("th")).enumerate() {
        keys.push(th.text().trim().to_string())
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
        record.insert(keys[i].clone(), td.text().trim().to_string());
    }

    Some(record)
}

fn extract_yield_data(text: &str) -> Result<Vec<HashMap<String, String>>, YCError> {
    let document = Document::from(&text[..]);
    let table = document.find(Class("t-chart")).next();
    let data = match table {
        Some(table) => {
            let mut keys = vec![];
            let mut data = vec![];
            let trs = table.find(Name("tr"));
            for (i, row) in trs.enumerate() {
                if i == 0 {
                    keys = extract_column_names(&row);
                } else {
                    match extract_row(&row, &keys) {
                        Some(record) => data.push(record),
                        None => continue,
                    }
                }
            }
            data
        }
        None => return Err(YCError::DataNotFound("t-chart".to_string())),
    };

    Ok(data)
}
