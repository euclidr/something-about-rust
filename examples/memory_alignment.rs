
// https://doc.rust-lang.org/edition-guide/rust-2018/data-types/choosing-alignment-with-the-repr-attribute.html

#[derive(Default)]
#[repr(align(16))]
struct Align16 {
    member1: i8,
    member2: i16,
    member3: i32,
    member4: i64,
}

#[derive(Default)]
struct AlignAuto {
    member1: i8,
    member2: i16,
    member3: i32,
    member4: i64,
}

#[derive(Default)]
#[repr(align(16))]
struct AlignSmall16 {
    member1: i8
}

#[derive(Default)]
struct AlignSmallAuto {
    member1: i8
}

fn main() {
    for _ in 0..10 {
        let a16 = Box::new(Align16::default());
        println!("===a16===: align_of: {}", std::mem::align_of::<Align16>());
        println!("a16: {}", &*a16 as *const Align16 as usize & 0xFF);
        println!("a16->member1: {}", &(a16.member1) as *const i8 as usize & 0xF);
        println!("a16->member2: {}", &(a16.member2) as *const i16 as usize & 0xF);
        println!("a16->member3: {}", &(a16.member3) as *const i32 as usize & 0xF);
        println!("a16->member4: {}", &(a16.member4) as *const i64 as usize & 0xF);

        let aauto = Box::new(AlignAuto::default());
        println!("===aauto===: align_of: {}", std::mem::align_of::<AlignAuto>());
        println!("aauto: {}", &*aauto as *const AlignAuto as usize & 0xFF);
        println!("aauto->member1: {}", &(aauto.member1) as *const i8 as usize & 0xF);
        println!("aauto->member2: {}", &(aauto.member2) as *const i16 as usize & 0xF);
        println!("aauto->member3: {}", &(aauto.member3) as *const i32 as usize & 0xF);
        println!("aauto->member4: {}", &(aauto.member4) as *const i64 as usize & 0xF);

        let as16 = Box::new(AlignSmall16::default());
        println!("===as16===: align_of: {}", std::mem::align_of::<AlignSmall16>());
        println!("as16: {}", &*as16 as *const AlignSmall16 as usize & 0xFF);
        println!("as16->member1: {}", &(as16.member1) as *const i8 as usize & 0xF);

        let asauto = Box::new(AlignSmallAuto::default());
        println!("===asauto===: align_of: {}", std::mem::align_of::<AlignSmallAuto>());
        println!("asauto: {}", &*asauto as *const AlignSmallAuto as usize & 0xFF);
        println!("asauto->member1: {}", &(asauto.member1) as *const i8 as usize & 0xF);
    }
}

// Results
// ===a16===: align_of: 16
// a16: 64
// a16->member1: 14
// a16->member2: 12
// a16->member3: 8
// a16->member4: 0
// ===aauto===: align_of: 8
// aauto: 160
// aauto->member1: 14
// aauto->member2: 12
// aauto->member3: 8
// aauto->member4: 0
// ===as16===: align_of: 16
// as16: 192
// as16->member1: 0
// ===asauto===: align_of: 1
// asauto: 224
// asauto->member1: 0
