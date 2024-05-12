use rand::{rngs::ThreadRng, thread_rng, Rng};
use std::{cmp::Ordering, fmt::Debug, ptr::NonNull};

pub struct RTreeMap<K: PartialOrd, V> {
    rng: ThreadRng,
    root: Link<K, V>,
}

impl<K: PartialOrd + Debug, V: Debug> Debug for RTreeMap<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{:?}",
            &self.root.map(|p| unsafe { p.as_ref() })
        ))
    }
}

impl<K: PartialOrd, V> RTreeMap<K, V> {
    pub fn new() -> Self {
        Self {
            rng: thread_rng(),
            root: None,
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let priority = self.rng.gen();
        if let Some(root) = self.root.as_ref() {
            unsafe { Node::insert(root.clone(), key, value, priority) };
            // It might happen that we are no longer referencing the root node.
            if unsafe { self.root.unwrap().as_ref().parent.is_some() } {
                self.root = unsafe { self.root.unwrap().as_ref().parent };
            }
        } else {
            self.root = NonNull::new(Box::into_raw(Node::new(key, value, priority)));
        }
    }

    pub fn remove(&mut self, key: K) -> Option<V> {
        if let Some(mut root) = self.root.take() {
            // we need to delete the root node so we have to replace it now to keep a reference to
            // the tree.
            unsafe {
                if root.as_mut().key == key {
                    match root.as_ref().get_highest_prio_child() {
                        Some(d) => match d {
                            Direction::Left => root.as_mut().rotate_right(),
                            Direction::Right => root.as_mut().rotate_left(),
                        },
                        None => {
                            let h = Box::from_raw(root.as_ptr());
                            return Some(h.value);
                        }
                    }
                    self.root = root.as_ref().parent;
                } else {
                    // Since we took the root node we will have to put it back in case it is not
                    // the match.
                    self.root = Some(root);
                }
                return Node::remove(root, key);
            }
        } else {
            None
        }
    }

    pub fn contains(&self, needle: K) -> bool {
        self.root
            .map(|r| unsafe { r.as_ref().contains(needle) })
            .unwrap_or(false)
    }
}

impl<K: PartialOrd, V> Drop for RTreeMap<K, V> {
    fn drop(&mut self) {
        if let Some(root) = self.root {
            let root = unsafe { Box::from_raw(root.as_ptr()) };
            root.drop_children();
        }
    }
}

type Link<K, V> = Option<NonNull<Node<K, V>>>;
type Parant<K, V> = Option<NonNull<Node<K, V>>>;

pub struct Node<K: PartialOrd, V> {
    parent: Parant<K, V>,
    key: K,
    value: V,
    priority: usize,
    left: Link<K, V>,
    right: Link<K, V>,
}

enum Direction {
    Left,
    Right,
}

impl<K: PartialOrd + Debug, V: Debug> Debug for Node<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("key", &self.key)
            .field("value", &self.value)
            .field("priority", &self.priority)
            .field("left", &self.left.map(|l| unsafe { l.as_ref() }))
            .field("right", &self.right.map(|r| unsafe { r.as_ref() }))
            .field("has_parent", &self.parent.is_some())
            .finish()
    }
}

impl<K: PartialOrd, V> Node<K, V> {
    fn new(key: K, value: V, priority: usize) -> Box<Self> {
        Box::new(Self {
            parent: None,
            key,
            value,
            priority,
            left: None,
            right: None,
        })
    }

    fn get_highest_prio_child(&self) -> Option<Direction> {
        match (self.left, self.right) {
            (None, None) => None,
            (None, Some(_)) => Some(Direction::Right),
            (Some(_), None) => Some(Direction::Left),
            (Some(l), Some(r)) => unsafe {
                if l.as_ref().priority >= r.as_ref().priority {
                    Some(Direction::Left)
                } else {
                    Some(Direction::Right)
                }
            },
        }
    }

    fn drop_children(mut self) {
        if let Some(left) = self.left.take() {
            let left = unsafe { Box::from_raw(left.as_ptr()) };
            left.drop_children()
        }
        if let Some(right) = self.right {
            let right = unsafe { Box::from_raw(right.as_ptr()) };
            right.drop_children()
        }
    }

    fn contains(&self, needle: K) -> bool {
        match self.key.partial_cmp(&needle) {
            Some(r) => match r {
                Ordering::Greater => self
                    .left
                    .map(|l| unsafe { l.as_ref().contains(needle) })
                    .unwrap_or(false),
                Ordering::Equal => true,
                Ordering::Less => self
                    .right
                    .map(|r| unsafe { r.as_ref().contains(needle) })
                    .unwrap_or(false),
            },
            None => panic!("Failed to compare values"),
        }
    }

    fn new_with_parent(key: K, value: V, priority: usize, parent: Parant<K, V>) -> Box<Self> {
        Box::new(Self {
            parent,
            key,
            value,
            priority,
            left: None,
            right: None,
        })
    }

    /// After a new insertion it is likely for the max heap structure of the tree to be gone
    /// so this function fixes it from the bottom up by rotating accordingly so we do not
    /// destroy the in order attribute of our search tree.
    unsafe fn fix(parent: Parant<K, V>) {
        if let Some(parent) = parent.map(|mut p| p.as_mut()) {
            if parent
                .left
                .map(|l| l.as_ref().priority > parent.priority)
                .unwrap_or(false)
            {
                parent.rotate_right();
                Self::fix(
                    parent
                        .parent
                        .expect("parent after rotation")
                        .as_ref()
                        .parent,
                )
            } else if parent
                .right
                .map(|r| r.as_ref().priority > parent.priority)
                .unwrap_or(false)
            {
                parent.rotate_left();
                Self::fix(
                    parent
                        .parent
                        .expect("parent after rotation")
                        .as_ref()
                        .parent,
                )
            }
        }
    }

    unsafe fn rotate_right(&mut self) {
        let Some(mut new_parent) = self.left.take() else {
            return;
        };
        self.left = new_parent.as_mut().right.take();
        if let Some(p) = self.parent.map(|mut p| p.as_mut()) {
            if p.left.map(|l| l.as_ref().key == self.key).unwrap_or(false) {
                p.left = Some(new_parent);
            } else {
                p.right = Some(new_parent);
            }
        }
        if let Some(l) = self.left.map(|mut p| p.as_mut()) {
            l.parent = NonNull::new(self as *mut Self)
        }
        new_parent.as_mut().parent = self.parent.take();
        self.parent = Some(new_parent);
        new_parent.as_mut().right = NonNull::new(self as *mut Self);
    }

    unsafe fn rotate_left(&mut self) {
        let Some(mut new_parent) = self.right.take() else {
            return;
        };
        self.right = new_parent.as_mut().left.take();
        if let Some(p) = self.parent.map(|mut p| p.as_mut()) {
            if p.left.map(|l| l.as_ref().key == self.key).unwrap_or(false) {
                p.left = Some(new_parent);
            } else {
                p.right = Some(new_parent);
            }
        }
        if let Some(r) = self.right.map(|mut p| p.as_mut()) {
            r.parent = NonNull::new(self as *mut Self)
        }
        new_parent.as_mut().parent = self.parent.take();
        self.parent = Some(new_parent);
        new_parent.as_mut().left = NonNull::new(self as *mut Self);
    }

    unsafe fn insert(mut dst: NonNull<Node<K, V>>, key: K, mut value: V, priority: usize) {
        let target = dst.as_mut();
        match target.key.partial_cmp(&key) {
            Some(Ordering::Equal) => std::mem::swap(&mut target.value, &mut value),
            Some(Ordering::Greater) => match target.left.as_ref() {
                Some(l) => {
                    Self::insert(*l, key, value, priority);
                }
                None => {
                    let new_element = Node::new_with_parent(key, value, priority, Some(dst));
                    target.left = NonNull::new(Box::into_raw(new_element));
                    Self::fix(Some(dst));
                }
            },
            Some(Ordering::Less) => match target.right.as_ref() {
                Some(r) => {
                    Self::insert(*r, key, value, priority);
                }
                None => {
                    let new_element = Node::new_with_parent(key, value, priority, Some(dst));
                    target.right = NonNull::new(Box::into_raw(new_element));
                    Self::fix(Some(dst));
                }
            },
            None => panic!("Failed to compare"),
        }
    }

    unsafe fn remove(mut node: NonNull<Node<K, V>>, key: K) -> Option<V> {
        while node.as_ref().key != key {
            match node
                .as_ref()
                .key
                .partial_cmp(&key)
                .expect("to be able to compare keys")
            {
                Ordering::Greater => {
                    if let Some(l) = node.as_ref().left {
                        node = l;
                    } else {
                        return None;
                    }
                }
                Ordering::Less => {
                    if let Some(r) = node.as_ref().right {
                        node = r;
                    } else {
                        return None;
                    }
                }
                Ordering::Equal => {}
            }
        }
        while let Some(d) = node.as_ref().get_highest_prio_child() {
            match d {
                Direction::Left => node.as_mut().rotate_right(),
                Direction::Right => node.as_mut().rotate_left(),
            }
        }
        let mut parent = node.as_ref().parent.unwrap();
        if parent
            .as_ref()
            .left
            .map(|l| l.as_ref().key == key)
            .unwrap_or(false)
        {
            let d = Box::from_raw(parent.as_mut().left.take().unwrap().as_ptr());
            return Some(d.value);
        }
        let d = Box::from_raw(parent.as_mut().right.take().unwrap().as_ptr());
        return Some(d.value);
    }
}
