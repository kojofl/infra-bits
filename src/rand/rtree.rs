use rand::{rngs::ThreadRng, thread_rng, Rng};
use std::{cmp::Ordering, fmt::Debug, ptr::NonNull};

#[derive(Debug)]
pub struct RTree<T: PartialOrd + Debug> {
    rng: ThreadRng,
    pub root: Link<T>,
}

impl<T: PartialOrd + Debug> RTree<T> {
    pub fn new() -> Self {
        Self {
            rng: thread_rng(),
            root: None,
        }
    }

    pub fn insert(&mut self, value: T) {
        let priority = self.rng.gen();
        if let Some(root) = self.root.as_ref() {
            unsafe { Node::insert(root.clone(), value, priority) };
            // It might happen that we are no longer referencing the root node.
            if unsafe { self.root.unwrap().as_ref().parent.is_some() } {
                self.root = unsafe { self.root.unwrap().as_ref().parent };
            }
        } else {
            self.root = NonNull::new(Box::into_raw(Node::new(value, priority)));
        }
    }

    pub fn contains(&self, needle: T) -> bool {
        self.root
            .map(|r| unsafe { r.as_ref().contains(needle) })
            .unwrap_or(false)
    }

    pub fn insert_high_prio(&mut self, value: T) {
        let priority = usize::MAX;
        if let Some(root) = self.root.as_ref() {
            unsafe { Node::insert(root.clone(), value, priority) };
            // It might happen that we are no longer referencing the root node.
            if unsafe { self.root.unwrap().as_ref().parent.is_some() } {
                self.root = unsafe { self.root.unwrap().as_ref().parent };
            }
        } else {
            self.root = NonNull::new(Box::into_raw(Node::new(value, priority)));
        }
    }
}

type Link<T> = Option<NonNull<Node<T>>>;
type Parant<T> = Option<NonNull<Node<T>>>;

pub struct Node<T: PartialOrd + Debug> {
    parent: Parant<T>,
    value: T,
    priority: usize,
    left: Link<T>,
    right: Link<T>,
}

impl<T: PartialOrd + Debug> Debug for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("value", &self.value)
            .field("priority", &self.priority)
            .field("left", &self.left.map(|l| unsafe { l.as_ref() }))
            .field("right", &self.right.map(|r| unsafe { r.as_ref() }))
            .field("has_parent", &self.parent.is_some())
            .finish()
    }
}

impl<T: PartialOrd + Debug> Node<T> {
    fn new(value: T, priority: usize) -> Box<Self> {
        Box::new(Self {
            parent: None,
            value,
            priority,
            left: None,
            right: None,
        })
    }

    fn contains(&self, needle: T) -> bool {
        match self.value.partial_cmp(&needle) {
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

    fn new_with_parent(value: T, priority: usize, parent: Parant<T>) -> Box<Self> {
        Box::new(Self {
            parent,
            value,
            priority,
            left: None,
            right: None,
        })
    }

    /// After a new insertion it is likely for the max heap structure of the tree to be gone
    /// so this function fixes it from the bottom up by rotating accordingly so we do not
    /// destroy the in order attribute of our search tree.
    unsafe fn fix(parent: Parant<T>) {
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
            if p.left
                .map(|l| l.as_ref().value == self.value)
                .unwrap_or(false)
            {
                p.left = Some(new_parent);
            } else {
                p.right = Some(new_parent);
            }
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
            if p.left
                .map(|l| l.as_ref().value == self.value)
                .unwrap_or(false)
            {
                p.left = Some(new_parent);
            } else {
                p.right = Some(new_parent);
            }
        }
        new_parent.as_mut().parent = self.parent.take();
        self.parent = Some(new_parent);
        new_parent.as_mut().left = NonNull::new(self as *mut Self);
    }

    unsafe fn insert(mut dst: NonNull<Node<T>>, mut value: T, priority: usize) {
        let target = dst.as_mut();
        match target.value.partial_cmp(&value) {
            Some(Ordering::Equal) => std::mem::swap(&mut target.value, &mut value),
            Some(Ordering::Greater) => match target.left.as_ref() {
                Some(l) => {
                    Self::insert(*l, value, priority);
                }
                None => {
                    let new_element = Node::new_with_parent(value, priority, Some(dst));
                    target.left = NonNull::new(Box::into_raw(new_element));
                    Self::fix(Some(dst));
                }
            },
            Some(Ordering::Less) => match target.right.as_ref() {
                Some(r) => {
                    Self::insert(*r, value, priority);
                }
                None => {
                    let new_element = Node::new_with_parent(value, priority, Some(dst));
                    target.right = NonNull::new(Box::into_raw(new_element));
                    Self::fix(Some(dst));
                }
            },
            None => panic!("Failed to compare"),
        }
    }
}
