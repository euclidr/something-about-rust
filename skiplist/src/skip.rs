use rand::random;
use std::borrow::Borrow;
use std::cmp::Ordering;
use core::ops::{Bound, Index, RangeBounds};
use core::marker::PhantomData;
use std::fmt::Debug;

#[derive(Debug)]
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

struct Range<'a, K, V> {
    front: Option<&'a Node<K, V>>,
    back: *const Node<K, V>,
}

impl<'a, K: Ord, V> Iterator for Range<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        // println!("next trace 0 {:?}", self.back);

        if self.back.is_null() {
            return None;
        }

        // println!("next trace 1");

        match self.front.take() {
            Some(node) => {
                let node_ptr :*const _ = node;
                if node_ptr != self.back {
                    self.front = node.next.as_ref().map(|node| { & **node});
                }
                Some((&node.key, &node.value))
            }
            None => None,
        }
    }
}

struct RangeMut<'a, K :'a, V: 'a> {
    front: Option<&'a mut Node<K, V>>,
    back: *const Node<K, V>,
}

impl<'a, K: Ord, V> Iterator for RangeMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.back.is_null() {
            return None;
        }

        match self.front.take() {
            Some(node) => {
                let node_ptr :*const _ = node;
                if node_ptr != self.back {
                    self.front = node.next.as_mut().map(|node| { &mut **node});
                }
                Some((&node.key, &mut node.value))
            }
            None => None,
        }
    }
}

impl<K: Ord + Debug, V: Debug> SkipList<K, V> {
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

    fn pop(&mut self) -> Option<(K, V)> {
        match self.next.take() {
            Some(mut node) => {
                self.next = node.next.take();
                for (i, n) in node.nexts.iter().enumerate() {
                    self.nexts[i] = *n;
                }
                if *self.nexts.last().unwrap() == std::ptr::null_mut() {
                    self.nexts.pop();
                }
                Some((node.key, node.value))
            },
            None => None,
        }
    }

    #[allow(dead_code)]
    fn range<T: ?Sized+Debug, R>(&self, range: R) -> Range<'_, K, V> 
        where T: Ord, K: Borrow<T>, R: RangeBounds<T> {
        if self.next.is_none() {
            return Range { front: None, back: std::ptr::null() };
        }

        let front_bound = match range.start_bound() {
            Bound::Unbounded => Some(&**self.next.as_ref().unwrap()),
            Bound::Included(key) => self._front_include(key),
            Bound::Excluded(key) => self._front_exclude(key),
        };

        // println!("first nexts trace {:?}", self.nexts);
        // println!("first trace {:?}", self.next);
        // println!("range trace 1 {:?}", front_bound);

        if front_bound.is_none() {
            return Range { front: None, back: std::ptr::null() };
        }

        // println!("range trace 2");

        let front_key = &front_bound.as_ref().unwrap().key;

        let back_bound = match range.end_bound() {
            Bound::Unbounded => self._get_last_node(),
            Bound::Included(key) => {
                // println!("back_bound included");
                match key.cmp(front_key.borrow()) {
                    Ordering::Greater | Ordering::Equal => self._back_include_ptr(key),
                    Ordering::Less => std::ptr::null(),
                }
            },
            Bound::Excluded(key) => {
                // println!("back_bound excluded");
                match key.cmp(front_key.borrow()) {
                    Ordering::Greater => {
                        // println!("back_bound greater");
                        self._back_exclude_ptr(key)
                    }
                    Ordering::Less | Ordering::Equal => std::ptr::null(),
                }
            }
        };

        // println!("range trace 3");

        if back_bound.is_null() {
            return Range { front: None, back: std::ptr::null() };
        }

        // println!("range trace 4 {:?}", unsafe { &(&*back_bound).key });

        Range { front: front_bound, back: back_bound }
    }

    #[allow(dead_code)]
    fn range_mut<Q: ?Sized, R>(&mut self, range: R) -> RangeMut<'_, K, V>
    where Q: Ord + Debug, K: Borrow<Q>, R: RangeBounds<Q> {
        if self.next.is_none() {
            return RangeMut { front: None, back: std::ptr::null() };
        }

        let front_bound_ptr = match range.start_bound() {
            Bound::Unbounded => self.nexts[0],
            Bound::Included(key) => self._front_include_ptr(key),
            Bound::Excluded(key) => self._front_exclude_ptr(key),
        };

        if front_bound_ptr.is_null() {
            return RangeMut { front: None, back: std::ptr::null() };
        }

        let front_bound = unsafe { Some(&mut *front_bound_ptr)};
        let front_key = &front_bound.as_ref().unwrap().key;

        let back_bound = match range.end_bound() {
            Bound::Unbounded => self._get_last_node(),
            Bound::Included(key) => {
                match key.cmp(front_key.borrow()) {
                    Ordering::Greater | Ordering::Equal => self._back_include_ptr(key),
                    Ordering::Less => std::ptr::null(),
                }
            },
            Bound::Excluded(key) => {
                match key.cmp(front_key.borrow()) {
                    Ordering::Greater => self._back_exclude_ptr(key),
                    Ordering::Less | Ordering::Equal => std::ptr::null(),
                }
            }
        };

        if back_bound.is_null() {
            return RangeMut { front: None, back: std::ptr::null() };
        }

        RangeMut { front: front_bound, back: back_bound }
    }

    fn _front_include<Q: ?Sized>(&self, key :&Q) -> Option<& Node<K, V>>
    where K: Borrow<Q>,
          Q: Ord + Debug,
    {

        // println!("front_include: {:?}", key);
        if self.next.is_none() {
            return None
        }

        let pre_ptr = self._get_pre_node(key);
        // println!("front_include pre_ptr: {:?}", pre_ptr);
        if !pre_ptr.is_null() {
            let pre = unsafe {
                &*pre_ptr
            };
            match pre.next.as_ref() {
                None => None,
                Some(node) => {
                    Some(&**node)
                }
            }
        } else {
            Some(&**self.next.as_ref().unwrap())
        }
    }

    fn _front_include_ptr<Q: ?Sized>(&self, key :&Q) -> *mut Node<K, V>
    where K: Borrow<Q>,
          Q: Ord + Debug,
    {
        if self.next.is_none() {
            return std::ptr::null_mut();
        }

        let pre_ptr = self._get_pre_node(key);
        if !pre_ptr.is_null() {
            let pre = unsafe { &*pre_ptr };
            match pre.next.as_ref() {
                None => std::ptr::null_mut(),
                Some(_) => pre.nexts[0],
            }
        } else {
            self.nexts[0]
        }
    }

    fn _front_exclude<Q: ?Sized>(&self, key :&Q) -> Option<& Node<K, V>>
    where K: Borrow<Q>,
          Q: Ord + Debug,
    {
        if self.next.is_none() {
            return None
        }

        let pre_ptr = self._get_pre_node(key);
        let current;
        if !pre_ptr.is_null() {
            let pre = unsafe { &*pre_ptr };
            match pre.next.as_ref() {
                None => current = None,
                Some(node) => current = Some(&**node),
            };
        } else {
            current = Some(&**self.next.as_ref().unwrap());
        }

        current.and_then(|node| {
            match key.cmp(node.key.borrow()) {
                Ordering::Equal => node.next.as_ref().map(|next| {&**next}),
                Ordering::Greater => Some(node),
                Ordering::Less => unreachable!(),
            }
        })
    }

    fn _front_exclude_ptr<Q: ?Sized>(&self, key :&Q) -> *mut Node<K, V>
    where K: Borrow<Q>,
          Q: Ord + Debug,
    {
        if self.next.is_none() {
            return std::ptr::null_mut()
        }

        let pre_ptr = self._get_pre_node(key);
        let current;
        let mut current_ptr = std::ptr::null_mut();
        if !pre_ptr.is_null() {
            let pre = unsafe { &*pre_ptr };
            match pre.next.as_ref() {
                None => current = None,
                Some(node) => {
                    current = Some(&**node);
                    current_ptr = pre.nexts[0];
                }
            };
        } else {
            current = Some(&**self.next.as_ref().unwrap());
            current_ptr = self.nexts[0];
        }

        if current.is_none() {
            return std::ptr::null_mut();
        }

        let node = current.unwrap();
        match key.cmp(node.key.borrow()) {
            Ordering::Equal => node.nexts[0],
            Ordering::Greater => current_ptr,
            Ordering::Less => unreachable!(),
        }
    }

    fn _back_include_ptr<Q: ?Sized>(&self, key :&Q) -> *const Node<K,V>
    where K: Borrow<Q>,
          Q: Ord + Debug,
    {
        let pre_ptr = self._get_pre_node(key);
        let current;
        if !pre_ptr.is_null() {
            let pre_node = unsafe { &*pre_ptr };
            current = pre_node.next.as_ref().map(|node| {&**node})
        } else {
            current = Some(&**self.next.as_ref().unwrap())
        }

        match current {
            None => pre_ptr as *const _,
            Some(node) => {
                match key.cmp(node.key.borrow()) {
                    Ordering::Equal => node as *const _,
                    Ordering::Greater => pre_ptr as *const _,
                    Ordering::Less => unreachable!(),
                }
            }
        }
    }

    fn _back_exclude_ptr<Q: ?Sized>(&self, key :&Q) -> *const Node<K, V>
    where K: Borrow<Q>,
          Q: Ord + Debug,
    {
        self._get_pre_node(key) as *const _
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

    fn _choose_level(&self, max: usize) -> usize {
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
        let mut level = self._choose_level(self.nexts.len());
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

    fn _get_node<Q: ?Sized>(&self, q: &Q) -> *mut Node<K, V>
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

    fn _get_pre_node<Q: ?Sized>(&self, q: &Q) -> *mut Node<K, V>
    where
        K: Borrow<Q>,
        Q: Ord + Debug,
    {
        if self.next.is_none() {
            return std::ptr::null_mut();
        }

        let mut nexts = &self.nexts;
        let mut level = self.nexts.len() - 1;
        let mut pre = std::ptr::null_mut();
        loop {
            if !nexts[level].is_null() {
                let tmp_key = unsafe { &(*nexts[level]).key };
                // println!("get_pre_node: q: {:?}, tmp_key: {:?}", q, tmp_key);
                match q.cmp(tmp_key.borrow()) {
                    Ordering::Greater => {
                        pre = nexts[level];
                        nexts = unsafe { &(*nexts[level]).nexts };
                        continue
                    },
                    Ordering::Equal | Ordering::Less => (),
                }
            }
            if level == 0 {
                break;
            }
            level -= 1;
        }

        // println!("get_pre_node result: q: {:?}, result: {:?}", q, unsafe { &(&*pre).key });

        pre
    }

    fn _get_last_node(&self) -> *mut Node<K, V> {
        if self.nexts.len() == 0 {
            return std::ptr::null_mut();
        }
        let mut nexts = &self.nexts;
        let mut level = self.nexts.len() - 1;
        let mut node = std::ptr::null_mut();
        loop {
            if !nexts[level].is_null() {
                node = nexts[level];
                nexts = unsafe { &(*nexts[level]).nexts };
                continue;
            }
            if level == 0 {
                break;
            }
            level -= 1;
        }

        node
    }

    #[allow(dead_code)]
    pub fn get<Q: ?Sized>(&self, q: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        let p_result = self._get_node(q);

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
        let p_result = self._get_node(q);

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

    pub fn into_iter(self) -> IntoIter<K, V> {
        IntoIter(self)
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

pub struct IntoIter<K, V>(SkipList<K, V>);

impl<K: Ord, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next.take() {
            Some(mut node) => {
                self.0.next = node.next.take();
                Some((node.key, node.value))
            },
            None => None,
        }
    }
}

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

    // #[test]
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

        for (k, v) in sk.into_iter() {
            println!("---4 {}: {}", k, v);
        }
    }

    #[test]
    fn range() {
        let mut sk = SkipList::new();
        sk.insert(10, 10);
        sk.insert(15, 15);
        sk.insert(12, 12);
        sk.insert(52, 52);
        sk.insert(22, 22);
        sk.insert(32, 32);

        println!("start iter");
        for (k, v) in sk.iter() {
            println!("iter_kv {}: {}", k, v)
        }

        println!("start range");
        for (k, v) in sk.range(15..31) {
            println!("range_kv {}: {}", k, v)
        }

        println!("start range2: 15..32");
        for (k, v) in sk.range(15..32) {
            println!("range_kv {}: {}", k, v)
        }

        println!("start range2: 15..");
        for (k, v) in sk.range(15..) {
            println!("range_kv {}: {}", k, v)
        }

        println!("start range2: ..33");
        for (k, v) in sk.range(..33) {
            println!("range_kv {}: {}", k, v)
        }

        println!("start range2: ..");
        for (k, v) in sk.range(..) {
            println!("range_kv {}: {}", k, v)
        }

        println!("start range_mut: 15..32");
        for (k, v) in sk.range_mut(15..32) {
            println!("range_mut_kv {}: {}", k, v);
            *v = 1;
        }

        println!("start range3: ..");
        for (k, v) in sk.range(..) {
            println!("range_kv {}: {}", k, v)
        }

    }

    // struct Node {
    //     a: i32,
    //     b: i32,
    // }

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
