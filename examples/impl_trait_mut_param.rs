// trait 只是对方法参数的类型作了限制，对参数的可变性是没有限制的
// 冒号后是参数类型 String 是一个类型 &String 也是一个类型
// &mut String 也是一个类型， 三者是不同的
// 冒号前就是这个类型对象的名字，名字不是trait限制的
// trait 也不规定名字指向的对象是否可变，因为名字对应的对象实际已经move到方法里
// 比如 a: &String, 那个引用已经被 move 到方法里（不是 String 被 move） 
// 要怎么处置 a，是具体实现的自由，具体实现可以写为 mut a: &String, 
// 那么 a 在方法中可以改引用别的 String。
trait Generic {
    fn f1(mut a: String);
    fn f2(a: String);
    fn f3(&self, a: String);
    fn f4(&mut self, a: String);
    fn f5(self);
}

#[derive(Debug)]
struct Concrete {
    a: String,
}

impl Generic for Concrete {
    // tait 的方法标记不包括参数名和参数的可变性
    fn f1(a1: String) {
        println!("f1: {}", a1);
    }

    // tait 的方法标记不包括参数名和参数的可变性
    fn f2(mut a: String) {
        a.push_str(" anything");
        println!("f2: {}", a);
    }

    // 引用 self 的可变性则要严格遵循 trait
    fn f3(&self, mut _a: String) {
        println!("f3");
    }

    // 引用 self 的可变性则要严格遵循 trait
    fn f4(&mut self, a: String) {
        self.a = a;
        println!("f4");
    }

    // self 的可变性也是不限制的
    fn f5(mut self) {
        self.a = String::from("none");
        println!("f5: {:?}", self);
    }
}

fn main() {
    Concrete::f1(String::from("data"));
    Concrete::f2(String::from("data"));
    let mut c = Concrete { a: String::from("a") };
    c.f3(String::from("data"));
    c.f4(String::from("data"));
    c.f5();
}