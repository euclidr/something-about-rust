use crate::request;
use crate::sharekv;
use crate::store::Yield;
use crate::YCFuture;

use chrono::prelude::*;
use chrono::Utc;
use futures::{future, Future};


/// sync a year's data
/// if data in that year is synced, do nothing
/// if it's current year check if latest data is synced
pub fn sync_year(year: &str) -> YCFuture<()> {
    let today = Utc::now();
    let synced;
    if year == &today.year().to_string()[..] {
        synced = is_synced(&today.format("%Y-%m-%d").to_string());
    } else {
        synced = is_synced(&format!("{}-12-31", year))
    }

    if synced {
        println!("{} has synced already", year);
        return Box::new(future::ok(()));
    }

    let year_string = year.to_string();
    let r = request::yield_of_year(year).and_then(move |data| {
        for date_record in data {
            match Yield::new(&date_record) {
                Ok(y) => y.save(),
                Err(err) => {
                    println!("new yield data error: {}", err);
                    continue;
                }
            }
        }
        record_synced_year(&year_string);
        println!("synced {} data.", year_string);
        future::ok(())
    });
    Box::new(r)
}

/// record synced year, if it's current year mark down the date
fn record_synced_year(year: &str) {
    let key_year = format!("synced_year:{}", year);
    let date = Utc::now();
    if date.year().to_string() != year {
        sharekv::set(&key_year, "1");
    } else {
        let date_str = date.format("%Y-%m-%d").to_string();
        sharekv::set(&key_year, &date_str);
    }
}

/// check if data in that day is synced 
/// if date is after today return true
pub fn is_synced(date: &str) -> bool {
    let today = Utc::now();
    let today_str = today.format("%Y-%m-%d").to_string();
    if date > &today_str[..] {
        return true;
    }

    let year = &date[..4];
    let key_year = format!("synced_year:{}", year);
    let latest = match sharekv::get(&key_year) {
        Some(latest) => latest,
        None => return false,
    };

    if &latest == "1" {
        return true;
    }

    if date > &latest[..] {
        return false;
    }
    return true;
}