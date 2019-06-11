use chrono::{Datelike, Utc};
use crate::sharekv;

pub fn yield_by_date(date: &str) -> Option<String> {
    None
}

pub fn set_yield_data(date: &str, value: &str) {

}

pub fn record_fetch_year(year: &str) {
    let key_year = format!("fetched_year:{}", year);
    let date = Utc::now();
    if date.year().to_string() != year {
        sharekv::set(&key_year, "1");
    } else {
        let date_str = date.format("%Y-%m-%d").to_string();
        sharekv::set(&key_year, &date_str);
    }

    // sharekv::set(&key_year, "1");
    // let date_str = date.format("%Y-%m-%d").to_string();

    // match sharekv::get("latest_fetch_date") {
    //     Some(latest) => {
    //         if date_str == latest {
    //             return;
    //         }
    //     }
    //     None => ()
    // };

    // sharekv::set("latest_fetch_date", &date_str);
}

pub fn is_fetched(date: &str) -> bool {
    let today = Utc::now();
    let today_str = today.format("%Y-%m-%d").to_string();
    if date >= &today_str[..] {
        return false;
    }

    let year = &date[..4];
    let key_year = format!("fetchd_year:{}", year);
    let latest = match sharekv::get(&key_year) {
        Some(latest) => latest,
        None => return false,
    };


    if date > &latest[..] {
        return false;
    }
    return true;

}
