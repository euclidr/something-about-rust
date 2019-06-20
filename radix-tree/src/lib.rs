// /a/b/:x/d
// /a/b/c
// /a/b/*y
// /d
// /d/
use std::collections::BTreeMap;
use std::default::Default;

enum NodeKind {
    Normal,
    Param,
    CatchAll,
}

impl Default for NodeKind {
    fn default() -> NodeKind {
        NodeKind::Normal
    }
}

struct Node<T> {
    kind: NodeKind,
    name: String,
    data: Option<T>,
    normal_children: Vec<Node<T>>,
    param_child: Box<Option<Node<T>>>,
    catch_all_child: Box<Option<Node<T>>>,
}

impl<T> Default for Node<T> {
    fn default() -> Node<T> {
        Node::<T> {
            kind: NodeKind::default(),
            name: String::from(""),
            data: None,
            normal_children: vec![],
            param_child: Box::new(None),
            catch_all_child: Box::new(None),
        }
    }
}

impl<T> Node<T> {
    fn new(segment: &str) -> Node<T> {
        unimplemented!()
    }

    fn child_from_segment(&mut self, segment: &str) -> Option<&mut Node<T>> {
        unimplemented!()
    }

    fn child_index(&self, segment: &str) -> Option<usize> {
        unimplemented!()
    }

    fn add_segment(&mut self, segment: &str) -> Result<&mut Node<T>, String> {
        if segment.starts_with(':') {
            match *self.param_child {
                Some(ref mut n) => {
                    if n.name == segment {
                        return Ok(n);
                    } else {
                        return Err("conflit".to_string());
                    }
                },
                None => {
                    if self.catch_all_child.is_some() {
                        return Err("conflit".to_string());
                    } else {
                        self.param_child = Box::new(Some(Node::default()));
                        match *self.param_child {
                            Some(ref mut n) => {
                                n.kind = NodeKind::Param;
                                n.name = segment.to_string();
                                return Ok(n)
                            }
                            None => return Err("impossible".to_string()),
                        }
                    }
                }
            }
        } else if segment.starts_with('*') {
            //
        } else {
            if self.child_index(segment).is_none() {
                let n = Node::<T>::new(segment);
                self.normal_children.push(n);
                self.normal_children.sort_by(|a, b| { a.name.cmp(&b.name) })
            }
            let idx = self.child_index(segment).unwrap();
            return Ok(&mut self.normal_children[idx]);
        }

        Err("impossible".to_string())
        // let n = Node::default();
        // self.param_child = Box::new(Some(n));
        // match *self.param_child {
        //     Some(ref mut n) => Ok(n),
        //     None => Err("err".to_string()),
        // }
        // unimplemented!()
    }

    fn add_child(&mut self, child: Node<T>) {
        unimplemented!()
    }

    fn set_data(&mut self, data: T) {
        unimplemented!()
    }

    // fn will_param_conflit(&self, param: &str) -> bool {
    //     unimplemented!()
    // }
}

struct Router<T> {
    root: Node<T>,
}

struct Match<T> {
    data: T,
    params: BTreeMap<String, String>
}

impl<T> Router<T> {

    pub fn add(&mut self, path: &str, data: T) {
        if !path.starts_with('/') {
            panic!("path schema must start with /");
        }

        if path.len() > 1 && path.ends_with('/') {
            panic!("path schema must not end with /");
        }

        if path.contains("//") {
            panic!("invalid path schema");
        }

        let path = &path[1..];
        let mut last = &mut self.root;
        for segment in path.split('/') {
            if segment.len() == 0 {
                break;
            }

            let rs = last.add_segment(segment);
            last = match rs {
                Ok(r) => r,
                Err(_) => return,
            };
        }

        last.set_data(data);

        // let path = &path[1..];
        // let mut last_old = &mut self.root;
        // let mut is_new = false;
        // let mut first_new = Node::<T>::default();
        // let mut last_new = &mut first_new;
        // for segment in path.split('/') {
        //     if segment.len() == 0 {
        //         break;
        //     }

        //     if is_new {
        //         if let NodeKind::CatchAll = last_new.kind {
        //             panic!("there should not be any items after catch_all")
        //         }

        //         last_new = last_new.add_segment(segment);

        //     } else {
        //         // {
        //         //     if let NodeKind::CatchAll = last_old.kind {
        //         //         panic!("there should not be any items after catch_all")
        //         //     }
        //         // }

        //         let node = last_old.child_from_segment(segment);
        //         if node.is_none() {
        //             is_new = true;
        //             first_new = Node::new(segment);
        //             last_new = &mut first_new;
        //         } else {
        //             last_old = node.unwrap()
        //         }

        //     }

        // }

        // if is_new {
        //     last_new.set_data(data);
        //     last_old.add_child(first_new);
        // } else {
        //     last_old.set_data(data);
        // }
    }

    pub fn recognize(&self, path: &str) -> Option<Match<T>> {
        None
    }
}

fn split() {
    for (idx, part) in "/".split('/').enumerate() {
        println!("{}: {}: {}", idx, part, &"1"[1..]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        split();
        assert_eq!(2 + 2, 4);
    }
}
