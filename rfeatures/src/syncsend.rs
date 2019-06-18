use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::thread;
use std::time::Duration;

#[derive(Default)]
struct Animal {
    leg: i32,
}

struct Book {
    animal: RefCell<Animal>,
}

unsafe impl Sync for Book {}

pub fn discover() {
    let tiger = RefCell::new(Animal::default());
    tiger.borrow_mut().deref_mut().leg = 4;
    let book = Book { animal: tiger };

    // 当不是 move 时，会提示 may outlive 错误
    // 当不是 move 且没有 impl Sync 时 提示
    // `std::cell::RefCell<syncsend::Animal>` cannot be shared between threads safely
    thread::spawn(move || {
        let b = &book;
        println!("found a {}", &b.animal.borrow().deref().leg);
    });

    // `std::cell::RefCell<syncsend::Animal>` cannot be shared between threads safely
    // thread::spawn(|| {
    //     println!("found a {}", &tiger.borrow().deref().leg);
    // });

    for i in 1..5 {
        println!("hi number {} from the main thread!", i);
        thread::sleep(Duration::from_millis(1));
    }
}
