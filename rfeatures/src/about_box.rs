use std::ops::DerefMut;
use std::cell::RefCell;
pub fn x() {
    // let mut i = Box::new(1);
    // let j = i.deref_mut();
    // *j = *j + 1;
    // // *i = *i + 1;
    // println!("{}", i);

    let s = "a".to_string();
    // let rs = &s;
    // let s2 = *rs;
    // // let bs = Box::new(s);
    // let rs = RefCell::new(s);
    let bs = Box::new(s);
    let xs = &*bs;
    println!("{}", bs);
}
