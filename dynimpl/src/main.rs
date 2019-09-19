trait Holo {
    fn relax(&self);
}

struct Concrete1 {
    value: i32,
}

impl Holo for Concrete1 {
    fn relax(&self) {
        println!("relax concrete1: {}", self.value)
    }
}

struct Concrete2 {
    value: i32,
}

impl Holo for Concrete2 {
    fn relax(&self) {
        println!("relax concrete2: {}", self.value)
    }
}

struct Array1 {
    arr: Vec<Box<dyn Holo>>,
}

fn dynamic_func<'a>(n: i32, d1: &'a dyn Holo, d2: &'a dyn Holo) -> &'a dyn Holo {
    if n < 0 {
        d1
    } else {
        d2
    }
}

fn static_func(_: impl Holo, _: impl Holo) -> impl Holo {
    Concrete1{value: 10}
}

impl Array1 {
    fn new() -> Self {
        Self {
            arr: vec![
                Box::new(Concrete1 { value: 1 }), // will be converted to a trait object
                Box::new(Concrete2 { value: 2 }),
            ],
        }
    }

    fn start(&self) {
        for item in self.arr.iter() {
            item.relax();
        }
    }
}

struct Array2<'a> {
    arr: Vec<&'a dyn Holo>,
}

impl<'a> Array2<'a> {
    fn new(c1: &'a Concrete1, c2: &'a Concrete2) -> Self {
        // c1, c2 will be convert to trait objects
        // actual type of elements in the arr will be missing
        Self { arr: vec![c1, c2] }
    }

    fn start(&self) {
        for item in self.arr.iter() {
            item.relax();
        }
    }
}

fn main() {
    let c1 = Concrete1 { value: 1 };
    let c2 = Concrete2 { value: 2 };

    println!("---------iterate Array1----------");
    let arr1 = Array1::new();
    arr1.start();

    println!("---------iterate Array2----------");
    let arr2 = Array2::new(&c1, &c2);
    arr2.start();

    println!("---------dynamic_func: n=1----------");
    let r = dynamic_func(1, &c1, &c2);
    r.relax();

    println!("---------dynamic_func: n=-1----------");
    let r = dynamic_func(-1, &c1, &c2);
    r.relax();

    println!("---------static_func----------");
    let r = static_func(c1, c2);
    r.relax();
    println!("Hello, world--");
}
