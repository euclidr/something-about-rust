fn main() {
    let num: i32 = 5;

    let r2 = &num as *const i32 as *mut i32;
    // the non-mutable value can be mutated
    unsafe {
        *r2 = 6;
    }
    println!("{:?} {:?}", r2, num);
    
    const N: i32 = 5;
    let r2 = &N as *const i32;
    println!("{:?} {:?}", r2, N);
    let r2 = r2 as *mut i32;
    // Segmentation fault
    // unsafe {
    //    *r2 = 6
    // }
    println!("{:?} {:?}", r2, N);
}