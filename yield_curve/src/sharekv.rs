use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::sync::RwLock;

// type ShareKV = Mutex<BTreeMap<String, String>>;

pub enum Error {
    IO(io::Error),
    InvalidFormat,
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IO(err)
    }
}

pub struct DB {
    writer: Option<File>,
    kv: BTreeMap<String, String>,
}

lazy_static! {
    // static ref BOND_YIELD_DATA: ShareKV = { Mutex::new(BTreeMap::<String, String>::new()) };
    // static ref DBFILE: Mutex<DBFile> = { Mutex::new(DBFile { file: None }) };
    pub static ref SHARE_DB: RwLock<DB> = {
       RwLock::new(
           DB {
               writer: None,
               kv: BTreeMap::<String, String>::new()
           }
       )
    };
}

impl DB {
    fn load_from_file(&mut self, file: File) -> Result<(), Error> {
        let buf_reader = BufReader::new(file);

        for line in buf_reader.lines() {
            let line = line?;
            let trimed = line.trim();
            if trimed.len() == 0 {
                continue;
            }
            let parts: Vec<&str> = trimed.splitn(2, '\t').collect();
            if parts.len() != 2 {
                return Err(Error::InvalidFormat);
            }
            self.kv.insert(parts[0].to_string(), parts[1].to_string());
        }
        Ok(())
    }

    pub fn init(&mut self, path: &str) -> Result<(), Error> {
        let fr = File::open(path);
        match fr {
            Ok(f) => self.load_from_file(f)?,
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => (),
            Err(e) => return Err(Error::IO(e)),
        };

        let fw = OpenOptions::new().write(true).append(true).create(true).open(path)?;
        self.writer = Some(fw);
        Ok(())
    }

    fn get(&self, key: &str) -> Option<String> {
        match self.kv.get(key) {
            Some(value) => Some(value.clone()),
            None => None,
        }
    }

    fn set(&mut self, key: &str, value: &str) -> Result<(), Error> {
        self.append_file_record(key, value)?;
        self.kv.insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn append_file_record(&mut self, key: &str, value: &str) -> Result<(), Error> {
        let line = format!("\n{}\t{}", key, value);
        let mut writer = self.writer.as_ref().unwrap();
        writer.write(line.as_bytes())?;
        writer.flush()?;
        Ok(())
    }
}

pub fn get(key: &str) -> Option<String> {
    let db = SHARE_DB.read().unwrap();
    db.get(key)
}

pub fn set(key: &str, value: &str) {
    let mut db = SHARE_DB.write().unwrap();
    match db.set(key, value) {
        _ => ()
    };
}

pub fn init(path: &str) -> Result<(), Error> {
    let mut db = SHARE_DB.write().unwrap();
    db.init(path)
}
