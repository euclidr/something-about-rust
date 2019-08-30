use rand::random;
use std::cmp::Ordering;

struct Node<K, V> {
    nexts: Vec<*mut Node<K, V>>,
    next: Option<Box<Node<K, V>>>,
    key: K,
    value: V,
}

impl<K, V> Node<K, V> {
    fn new(levels: usize, k :K, v :V) -> Node<K, V> {
        Node {
            nexts: vec![std::ptr::null_mut(); levels],
            next: None,
            key: k,
            value: v,
        }
    }
}

struct SkipList<K, V> {
    nexts: Vec<*mut Node<K, V>>,
    next: Option<Box<Node<K,V>>>,
}

impl<K: Ord, V> SkipList<K, V> {
    fn new() -> SkipList<K, V> {
        SkipList {
            nexts: vec![],
            next: None,
        }
    }

    fn choose_level(&self, max :usize) -> usize {
        let mut num = random::<usize>();
        let mut level = 0;
        while level < max {
            if num & 1 == 1 {
                break
            }
            level += 1;
            num >>= 1;
        }
        level
    }

    fn insert(&mut self, k :K, v :V) -> Option<V> {
        let mut level = self.choose_level(self.nexts.len());
        if level == self.nexts.len() {
            self.nexts.push(std::ptr::null_mut())
        }

        let mut new_node = Box::new(Node::new(level+1, k, v));
        let p_new_node: *mut _ = &mut *new_node;

        let mut nexts = &mut self.nexts;
        let mut pre = std::ptr::null_mut();
        let mut is_equal = false;
        loop {
            if nexts[level].is_null() {
                nexts[level] = p_new_node;
            } else {
                let tmp_key = unsafe {
                    &(*nexts[level]).key
                };
                match new_node.key.cmp(tmp_key) {
                    Ordering::Greater => {
                        pre = nexts[level];
                        nexts = unsafe {
                            &mut (*nexts[level]).nexts
                        };
                        continue;
                    },
                    Ordering::Equal => {
                        new_node.nexts[level] = unsafe {
                            (*nexts[level]).nexts[level]
                        };
                        nexts[level] = p_new_node;
                        is_equal = true;
                    },
                    Ordering::Less => {
                        new_node.nexts[level] = nexts[level];
                        nexts[level] = p_new_node;
                    }
                }
            }
            if level == 0 {
                break;
            }
            level -= 1;
        }

        let mut result = None;

        if pre.is_null() {
            if is_equal{
                let mut old = self.next.take().unwrap();
                result = Some(old.value);
                new_node.next = old.next.take();
                self.next = Some(new_node);
            } else {
                self.next = match self.next.take() {
                    None => Some(new_node),
                    Some(old_node) => {
                        new_node.next = Some(old_node);
                        Some(new_node)
                    }
                }
            }
        } else {
            let pre_node = unsafe {
                &mut *pre
            };
            if is_equal {
                new_node.next = pre_node.next.as_mut().unwrap().next.take();
                let old = pre_node.next.take().unwrap();
                result = Some(old.value);
            } else {
                new_node.next = pre_node.next.take();
            }
            pre_node.next = Some(new_node);
        }

        result
    }

    fn remove(&mut self, q :&K) -> Option<V> {
        if self.nexts.len() == 0 {
            return None;
        }
        let mut level = self.nexts.len() - 1;
        let mut nexts = &mut self.nexts;
        let mut pre = std::ptr::null_mut();
        let mut cur = std::ptr::null_mut();
        let mut is_equal = false;
        loop {
            if !nexts[level].is_null() {
                let tmp_key = unsafe {
                    &(*nexts[level]).key
                };
                match q.cmp(tmp_key) {
                    Ordering::Greater => {
                        cur = nexts[level];
                        nexts = unsafe {
                            &mut (*nexts[level]).nexts
                        };
                        continue;
                    },
                    Ordering::Equal => {
                        pre = cur;
                        nexts[level] = unsafe {
                            (*nexts[level]).nexts[level]
                        };
                        is_equal = true;
                    },
                    Ordering::Less => (),
                }
            }
            if level == 0 {
                break;
            }
            level -= 1;
        }

        if !is_equal {
            return None
        }

        let result;

        if pre.is_null() {
            let mut toremove = self.next.take().unwrap();
            self.next = toremove.next.take();
            result = Some(toremove.value);
        } else {
            let pre_node = unsafe {
                &mut *pre
            };
            let mut cur = pre_node.next.take().unwrap();
            pre_node.next = cur.next.take();
            result = Some(cur.value);
        }

        result
    }

    fn get_node(&self, q :&K) -> *mut Node<K, V> {
        let mut nexts = &self.nexts;
        let mut level = self.nexts.len() - 1;
        let mut p_result = std::ptr::null_mut();
        loop {
            if !nexts[level].is_null() {
                let tmp_key = unsafe {
                    &(*nexts[level]).key
                };
                match q.cmp(tmp_key) {
                    Ordering::Greater => {
                        nexts = unsafe {
                            &(*nexts[level]).nexts
                        };
                        continue;
                    },
                    Ordering::Equal => {
                        p_result = nexts[level];
                        break;
                    },
                    Ordering::Less => (),
                }
            }
            if level == 0 {
                break;
            }
            level -= 1;
        }

        p_result
    }

    fn get(&self, q :&K) -> Option<&V> {
        let p_result = self.get_node(q);

        if !p_result.is_null() {
            let result_node = unsafe {
                &*p_result
            };
            Some(&result_node.value)
        } else {
            None
        }
    }


    fn get_mut(&mut self, q :&K) -> Option<&mut V> {
        let p_result = self.get_node(q);

        if !p_result.is_null() {
            let result_node = unsafe {
                &mut *p_result
            };
            Some(&mut result_node.value)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skiplist_basic() {
        let mut sk = SkipList::new();
        sk.insert("aa".to_string(), "aa1".to_string());
        sk.insert("ab".to_string(), "ab1".to_string());
        sk.insert("dd".to_string(), "dd1".to_string());
        sk.insert("aa".to_string(), "aa2".to_string());
        sk.insert("a".to_string(), "a1".to_string());

        let aa = "aa".to_string();
        let ab = "ab".to_string();
        let dd = "dd".to_string();
        let cc = "cc".to_string();
        println!("aa: {:?}", sk.get(&aa));
        println!("ab: {:?}", sk.get(&ab));
        println!("dd: {:?}", sk.get(&dd));
        println!("cc: {:?}", sk.get(&cc));
        println!("aa: {:?}", sk.get_mut(&aa));
        println!("ab: {:?}", sk.get_mut(&ab));
        println!("dd: {:?}", sk.get_mut(&dd));
        println!("cc: {:?}", sk.get_mut(&cc));

        let r_ab = sk.remove(&ab);
        println!("r_ab: {:?}", r_ab);
        let r_ab = sk.remove(&ab);
        println!("r_ab: {:?}", r_ab);
        let r_dd = sk.remove(&dd);
        println!("r_dd: {:?}", r_dd);
        println!("dd: {:?}", sk.get(&dd));

        match sk.get_mut(&dd) {
            None => (),
            Some(ddv) => *ddv = "dd2".to_string(),
        };

        let a = sk.get_mut(&dd);
        match a {
            None => (),
            Some(ddv) => *ddv = "dd3".to_string(),
        };

        let b = sk.get_mut(&dd);
        match b {
            None => (),
            Some(ddv) => *ddv = "dd4".to_string(),
        };

        // println!("{}", b.unwrap());

        println!("dd: {:?}", sk.get_mut(&dd));
    }

    // #[derive(Debug)]
    // struct Class {
    //     a :String,
    // }

    // impl Class {
    //     fn get_a(&mut self) -> &mut String {
    //         &mut self.a
    //     }
    // }

    // #[test]
    // fn t_class() {
    //     let mut cl = Class{a: "a".to_string()};
    //     let a = cl.get_a();
    //     let b = cl.get_a();

    //     *b = "a1".to_string();
    //     *a = "a2".to_string();

    //     println!("{}", b);

    // }
}