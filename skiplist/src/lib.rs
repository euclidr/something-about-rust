// use std::cmp::Ord;
// use std::cmp::Ordering;
// // use std::collections::BTreeMap;
// use std::rc::Rc;
// use std::borrow::Borrow;

// use rand::prelude::*;
// use rand::random;

// const DEFAULT_MAX_LEVEL: usize = 8;

// struct Node<K, V> {
//     nexts: Vec<Option<Rc<Box<Node<K, V>>>>>,
//     key: K,
//     value: V,
// }

// struct NodeHead<K, V> {
//     nexts: Vec<Option<Rc<Box<Node<K, V>>>>>,
// }

// struct SkipList<K, V> {
//     head: NodeHead<K, V>,
//     cnt :u64,
//     max_level :usize,
    
// }

// impl<K, V> SkipList<K, V> where K: Ord {
//     fn new() -> SkipList<K, V> {
//         let head = NodeHead::<K, V>{
//             nexts: vec![None; DEFAULT_MAX_LEVEL]
//         };
//         SkipList {
//             head,
//             cnt: 0,
//             max_level: DEFAULT_MAX_LEVEL,
//         }
//     }

//     #[allow(dead_code)]
//     fn get<Q: ?Sized>(&self, q :&Q) -> Option<&V>
//         where K: Borrow<Q>,
//               Q: Ord {
//         let mut level = self.max_level - 1;
//         let mut nexts = Some(&self.head.nexts);
//         let mut result = None;
//         while let Some(_nexts) = nexts {
//             match &_nexts[level] {
//                 Some(item) => {
//                     match q.cmp(item.key.borrow()) {
//                         Ordering::Equal => {
//                             nexts = None;
//                             result = Some(&item.value);
//                         },
//                         Ordering::Greater => {
//                             nexts = Some(&item.nexts);
//                         },
//                         Ordering::Less => {
//                             if level == 0 {
//                                 break;
//                             }
//                             level -= 1;
//                         }
//                     }
//                 },
//                 None => {
//                     if level == 0 {
//                         break
//                     }
//                     level -= 1;
//                 }
//             }
//         }
//         result
//     }

//     fn insert(&mut self, k :K, v :V) -> Option<V> {
//         None
//     }

//     fn get_mut(&mut self, k &K) -> Option<&mut V> {
//         None
//     }

//     fn take(&mut self, k &K) -> Option<V> {
//         None
//     }

//     fn remove(&mut self, k &K) -> Option<V> {
//         None
//     }

//     range
// }

mod skip;


#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::borrow::Borrow;
    #[derive(Debug)]
    struct A(i32);
    fn boo<Q: Borrow<A>+Debug>(q: &Q) {
        println!("-- {:?}", q);
    }

    #[derive(Debug)]
    struct Val(i32);

    #[derive(Debug)]
    struct Item {
        val :Val,
    }

    impl Item {
        fn f1(&self) -> &Val {
            &self.val
        }

        fn f2(&mut self) -> &mut Val {
            self.val = Val(self.val.0+1);
            &mut self.val
        }

        fn f3(&mut self) {
            self.val = Val(self.val.0+1);
        }

        fn f4(&mut self) {
            println!("x");
        }
    }

    #[test]
    fn t_f() {
        let mut item = Item{val: Val(1)};
        // let a = item.f1();
        item.f4();
        item.f3();
        // println!("a {:?}", *a);
    }

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
        let a = A(10);
        boo(&a)
    }
}
