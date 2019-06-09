use std::collections::BTreeMap;
use std::sync::Mutex;

type ShareKV = Mutex<BTreeMap<String, String>>;

lazy_static! {
    static ref BOND_YIELD_DATA: ShareKV = { Mutex::new(BTreeMap::<String, String>::new()) };
}

// TODO make it persistent

pub fn get(key: &str) -> Option<String> {
    let db = BOND_YIELD_DATA.lock().unwrap();
    match db.get(key) {
        Some(value) => Some(value.clone()),
        None => None,
    }
}

pub fn set(key: &str, value: &str) {
    let mut db = BOND_YIELD_DATA.lock().unwrap();
    db.insert(key.to_string(), value.to_string());
}
