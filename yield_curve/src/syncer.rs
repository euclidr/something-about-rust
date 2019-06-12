use crate::request;
use crate::store::{self, Yield};
use crate::ycerror::YCError;
use crate::YCFuture;
use futures::{future, Future};

pub fn get_remote_bond_yield(date: String) -> YCFuture<Yield> {
    let year = &date[..4];
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
        store::record_synced_year(&year_string);
        match Yield::get(&date) {
            Some(val) => future::ok(val),
            None => future::err(YCError::DataNotFound(date.clone())),
        }
    });
    Box::new(r)
}

pub fn sync_year(year: &str) -> YCFuture<()> {
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
        store::record_synced_year(&year_string);
        future::ok(())
    });
    Box::new(r)
    // Box::new(get_remote_bond_yield(date).map(|_| ()).map_err(|err| {
    //     println!("error occur: {}", err.to_string());
    //     err
    // }))
}
