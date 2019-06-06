fn main() {
    println!("Hello, world!");
}

fn fn_once_example() {
    let x = 'a'.to_string();
    let mut cnt = 0;
    let upper_x = move || {
        if cnt == 2 {
            x
        } else {
            cnt = cnt + 1;
            x.to_uppercase()
        }
    };
    // 这不能编译
    // note: closure cannot be invoked more than once because it moves the variable `x` out of its environment
    // 因为 upper_x 将 x 返回了
    for _ in 0..2 {
        let y = upper_x();
        println!("{}", y)
    }
    upper_x();
}

fn move_used_with_closure() {
    let cnt = "1".to_string();
    // let fn_cnt = move || { // 如果用 move，会编译不过，因为即使用了 & 也会 take ownership
    let fn_cnt = || {
        &cnt // 如果没有 & 的话会编译不过，因为没有 & 表示take ownership了
    };
    for _ in 0..2 {
        let y = fn_cnt();
        println!("return: {}", y)
    }

    println!("cnt: {}", cnt)
}