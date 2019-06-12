use crate::sharekv;
use crate::YCError;
use chrono::prelude::*;
use chrono::{Datelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Yield {
    date: String,
    mo1: String,
    mo2: String,
    mo3: String,
    mo6: String,
    yr1: String,
    yr2: String,
    yr3: String,
    yr5: String,
    yr7: String,
    yr10: String,
    yr20: String,
    yr30: String,
}

impl Yield {
    pub fn new(raw: &HashMap<String, String>) -> Result<Yield, YCError> {
        let raw_date = match raw.get("Date") {
            Some(d) => d,
            None => return Err(YCError::InvalidData),
        };
        let date = match NaiveDate::parse_from_str(&raw_date, "%m/%d/%y") {
            Ok(d) => d,
            Err(_) => return Err(YCError::InvalidData),
        };
        let date = date.format("%Y-%m-%d").to_string();
        let default_val = "N/A".to_string();
        Ok(Yield {
            date: date,
            mo1: raw.get("1 mo").unwrap_or(&default_val).to_string(),
            mo2: raw.get("2 mo").unwrap_or(&default_val).to_string(),
            mo3: raw.get("3 mo").unwrap_or(&default_val).to_string(),
            mo6: raw.get("6 mo").unwrap_or(&default_val).to_string(),
            yr1: raw.get("1 yr").unwrap_or(&default_val).to_string(),
            yr2: raw.get("2 yr").unwrap_or(&default_val).to_string(),
            yr3: raw.get("3 yr").unwrap_or(&default_val).to_string(),
            yr5: raw.get("5 yr").unwrap_or(&default_val).to_string(),
            yr7: raw.get("7 yr").unwrap_or(&default_val).to_string(),
            yr10: raw.get("10 yr").unwrap_or(&default_val).to_string(),
            yr20: raw.get("20 yr").unwrap_or(&default_val).to_string(),
            yr30: raw.get("30 yr").unwrap_or(&default_val).to_string(),
        })
    }

    pub fn get(date: &str) -> Option<Yield> {
        let key = format!("yield:{}", date);
        match sharekv::get(&key) {
            Some(raw) => match serde_json::from_str(&raw) {
                Ok(result) => Some(result),
                Err(_) => None,
            },
            None => None,
        }
    }

    pub fn save(&self) {
        let key = format!("yield:{}", self.date);
        sharekv::set(&key, &serde_json::to_string(self).unwrap());
    }

    pub fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

/// record synced year, if it's current year mark down the date
pub fn record_synced_year(year: &str) {
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
    if date >= &today_str[..] {
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
