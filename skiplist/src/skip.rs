use rand::random;
use std::borrow::Borrow;
use std::cmp::Ordering;

struct Node<K, V> {
    nexts: Vec<*mut Node<K, V>>,
    next: Option<Box<Node<K, V>>>,
    key: K,
    value: V,
}

impl<K, V> Node<K, V> {
    fn new(levels: usize, k: K, v: V) -> Node<K, V> {
        Node {
            nexts: vec![std::ptr::null_mut(); levels],
            next: None,
            key: k,
            value: v,
        }
    }

    // Remove next node, leaving nexts untouched.
    // Returns contents that was removed.
    // Caller must sort out the `nexts` before removing next node.
    fn _remove_next(&mut self) -> Option<(K, V)> {
        match self.next.take() {
            Some(mut node) => {
                self.next = node.next.take();
                Some((node.key, node.value))
            }
            None => None,
        }
    }

    // Replace next node with new node.
    // Returns contents that was replaced.
    // Caller must sort out the `nexts` before replacing next node.
    // It will be pannic if there is no next node.
    fn _replace_next(&mut self, mut node: Box<Self>) -> Option<(K, V)> {
        match self.next.take() {
            Some(old) => {
                node.next = old.next;
                self.next = Some(node);
                Some((old.key, old.value))
            }
            None => unreachable!(),
        }
    }

    // Insert a node right after.
    // Caller must sort out the `nexts` before inserting.
    fn _insert_next(&mut self, mut node: Box<Self>) {
        match self.next.take() {
            Some(old) => {
                node.next = Some(old);
                self.next = Some(node);
            }
            None => {
                node.next = None;
                self.next = Some(node);
            }
        }
    }
}

struct SkipList<K, V> {
    nexts: Vec<*mut Node<K, V>>,
    next: Option<Box<Node<K, V>>>,
}

impl<K: Ord, V> SkipList<K, V> {
    pub fn new() -> SkipList<K, V> {
        SkipList {
            nexts: vec![],
            next: None,
        }
    }
    // TODO
    // range
    // iter
    // split
    // contains
    // clears

    // Remove next node, leaving nexts untouched.
    // Returns contents that was removed.
    // Caller must sort out the `nexts` before removing next node.
    fn _remove_next(&mut self) -> Option<(K, V)> {
        match self.next.take() {
            Some(mut node) => {
                self.next = node.next.take();
                Some((node.key, node.value))
            }
            None => None,
        }
    }

    // Replace next node with new node.
    // Returns contents that was replaced.
    // Caller must sort out the `nexts` before replacing next node.
    // It will be pannic if there is no next node.
    fn _replace_next(&mut self, mut node: Box<Node<K, V>>) -> Option<(K, V)> {
        match self.next.take() {
            Some(old) => {
                node.next = old.next;
                self.next = Some(node);
                Some((old.key, old.value))
            }
            None => unreachable!(),
        }
    }

    // Insert a node right after.
    // Caller must sort out the `nexts` before inserting.
    fn _insert_next(&mut self, mut node: Box<Node<K, V>>) {
        match self.next.take() {
            Some(old) => {
                node.next = Some(old);
                self.next = Some(node);
            }
            None => {
                node.next = None;
                self.next = Some(node);
            }
        }
    }

    fn choose_level(&self, max: usize) -> usize {
        let mut num = random::<usize>();
        let mut level = 0;
        while level < max {
            if num & 1 == 1 {
                break;
            }
            level += 1;
            num >>= 1;
        }
        level
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        let mut level = self.choose_level(self.nexts.len());
        if level == self.nexts.len() {
            self.nexts.push(std::ptr::null_mut())
        }

        let mut new_node = Box::new(Node::new(level + 1, k, v));
        let p_new_node: *mut _ = &mut *new_node;

        let mut nexts = &mut self.nexts;
        let mut pre = std::ptr::null_mut();
        let mut equal = false;
        loop {
            if nexts[level].is_null() {
                // If reach the end of the level, append the new node
                // and go to the next level
                nexts[level] = p_new_node;
            } else {
                // Current key to be compared
                let tmp_key = unsafe { &(*nexts[level]).key };

                match new_node.key.cmp(tmp_key) {
                    Ordering::Greater => {
                        // Record pre node for later use.
                        pre = nexts[level];

                        nexts = unsafe { &mut (*nexts[level]).nexts };

                        continue;
                    }
                    Ordering::Equal => {
                        // Replace old node with new node
                        new_node.nexts[level] = unsafe { (*nexts[level]).nexts[level] };
                        nexts[level] = p_new_node;

                        equal = true;
                    }
                    Ordering::Less => {
                        // Insert new node
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
            if equal {
                result = self._replace_next(new_node);
            } else {
                self._insert_next(new_node);
            }
        } else {
            let pre_node = unsafe { &mut *pre };

            if equal {
                pre_node._replace_next(new_node);
            } else {
                pre_node._insert_next(new_node);
            }
        }

        result.map(|(_, v)| v)
    }

    #[allow(dead_code)]
    pub fn remove<Q: ?Sized>(&mut self, q: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        if self.nexts.len() == 0 {
            return None;
        }

        let mut level = self.nexts.len() - 1;
        let mut nexts = &mut self.nexts;
        let mut pre = std::ptr::null_mut();
        let mut equal = false;
        loop {
            if !nexts[level].is_null() {
                let tmp_key = unsafe { &(*nexts[level]).key };

                match q.cmp(tmp_key.borrow()) {
                    Ordering::Greater => {
                        // Update pre node ptr, as the level goes down it will
                        // finally reach the node just before the node to search.
                        pre = nexts[level];

                        nexts = unsafe { &mut (*nexts[level]).nexts };
                        continue;
                    }
                    Ordering::Equal => {
                        nexts[level] = unsafe { (*nexts[level]).nexts[level] };

                        equal = true;
                    }
                    Ordering::Less => (),
                }
            }
            if level == 0 {
                break;
            }
            level -= 1;
        }

        if !equal {
            return None;
        }

        let result;
        if pre.is_null() {
            result = self._remove_next();
        } else {
            let pre_node = unsafe { &mut *pre };
            result = pre_node._remove_next();
        }

        result.map(|(_, v)| v)
    }

    #[allow(dead_code)]
    fn get_node<Q: ?Sized>(&self, q: &Q) -> *mut Node<K, V>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        let mut nexts = &self.nexts;
        let mut level = self.nexts.len() - 1;
        let mut p_result = std::ptr::null_mut();
        loop {
            if !nexts[level].is_null() {
                let tmp_key = unsafe { &(*nexts[level]).key };
                match q.cmp(tmp_key.borrow()) {
                    Ordering::Greater => {
                        nexts = unsafe { &(*nexts[level]).nexts };
                        continue;
                    }
                    Ordering::Equal => {
                        p_result = nexts[level];
                        break;
                    }
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

    #[allow(dead_code)]
    pub fn get<Q: ?Sized>(&self, q: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        let p_result = self.get_node(q);

        if !p_result.is_null() {
            let result_node = unsafe { &*p_result };
            Some(&result_node.value)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn get_mut<Q: ?Sized>(&mut self, q: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        let p_result = self.get_node(q);

        if !p_result.is_null() {
            let result_node = unsafe { &mut *p_result };
            Some(&mut result_node.value)
        } else {
            None
        }
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, K, V> {
        Iter {
            next: self.next.as_ref().map(|node| &**node),
        }
    }

    pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a, K, V> {
        IterMut {
            next: self.next.as_mut().map(|node| &mut **node),
        }
    }
}

struct Iter<'a, K, V> {
    next: Option<&'a Node<K, V>>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_ref().map(|node| &**node);
            (&node.key, &node.value)
        })
    }
}

struct IterMut<'a, K, V> {
    next: Option<&'a mut Node<K, V>>,
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node| {
            self.next = node.next.as_mut().map(|node| &mut **node);
            (&node.key, &mut node.value)
        })
    }
}
// pub enum Entry<'a, K: 'a, V: 'a> {
//     /// A vacant entry.
//     Vacant(VacantEntry<'a, K, V>),

//     /// An occupied entry.
//     Occupied(OccupiedEntry<'a, K, V>),
// }

// pub struct Range<'a, K: 'a, V: 'a> {
//     front: Handle<NodeRef<marker::Immut<'a>, K, V, marker::Leaf>, marker::Edge>,
//     back: Handle<NodeRef<marker::Immut<'a>, K, V, marker::Leaf>, marker::Edge>,
// }

// pub struct Iter<'a, K: 'a, V: 'a> {
//     range: Range<'a, K, V>,
//     length: usize,
// }

// impl<K, V> Iterator for SkipList<K, V> {

// }

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    fn skiplist_basic() {
        let mut sk = SkipList::new();
        sk.insert("aa".to_string(), "aa1".to_string());
        sk.insert("ab".to_string(), "ab1".to_string());
        sk.insert("dd".to_string(), "dd1".to_string());
        sk.insert("aa".to_string(), "aa2".to_string());
        sk.insert("a".to_string(), "a1".to_string());

        let a = "a".to_string();
        let aa = "aa".to_string();
        let ab = "ab".to_string();
        let dd = "dd".to_string();
        let cc = "cc".to_string();
        println!("a: {:?}", sk.get(&a));
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

        println!("dd: {:?}", sk.get_mut(&dd));
    }

    #[test]
    fn iter() {
        let mut sk = SkipList::new();
        sk.insert("aa".to_string(), "aa1".to_string());
        sk.insert("ab".to_string(), "ab1".to_string());
        sk.insert("dd".to_string(), "dd1".to_string());
        sk.insert("aa".to_string(), "aa2".to_string());
        sk.insert("a".to_string(), "a1".to_string());

        for (k, v) in sk.iter() {
            println!("--- {}: {}", k, v);
        }

        for (k, v) in sk.iter_mut() {
            println!("---2 {}: {}", k, v);
            *v = "hhh".to_string();
        }

        for (k, v) in sk.iter() {
            println!("---3 {}: {}", k, v);
        }
    }



    struct Node {
        a: i32,
        b: i32,
    }

    // #[test]
    // fn multi_mut() {
    //     let mut n = Node{a: 0, b: 0};
    //     let pn1: *mut _ = &mut n;
    //     let pn2: *mut _ = &mut n;

    //     let upn1 = unsafe {
    //         &mut *pn1
    //     };
    //     let upn2 = unsafe {
    //         &mut *pn2
    //     };

    //     upn1.a = 1;
    //     upn2.b = 2;
    //     upn1.a = 11;

    //     println!("{}", upn1.a);
    //     println!("{}", upn2.a);
    //     println!("{}", upn2.b);
    // }

    // #[test]
    // fn multi_mut2() {
    //     let mut n = Node{a: 0, b: 0};
    //     let rn2 = &mut n;
    //     let rn1 = &mut n;

    //     println!("{}", rn1.a)

    // }

}
