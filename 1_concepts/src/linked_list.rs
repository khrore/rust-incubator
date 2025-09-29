use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

#[derive(Debug)]
struct Node<T> {
    next: Option<Arc<Mutex<Node<T>>>>,
    prev: Option<Arc<Mutex<Node<T>>>>,
    val: T,
}

#[derive(Debug)]
pub struct LinkedList<T> {
    head: Option<Arc<Mutex<Node<T>>>>,
    tail: Option<Arc<Mutex<Node<T>>>>,
    len: usize,
}

impl<T> Node<T> {
    pub fn new(val: T) -> Self {
        Self {
            next: None,
            prev: None,
            val,
        }
    }
}

impl<T> LinkedList<T>
where
    T: Clone + Copy,
{
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            len: 0,
        }
    }

    pub fn push_back(&mut self, val: T) {
        let node = Arc::new(Mutex::new(Node::new(val)));
        match self.head.take() {
            Some(head) => {
                head.lock().unwrap().next = Some(node.clone());
                node.lock().unwrap().prev = Some(head);
                self.head = Some(node);
            }
            None => {
                self.head = Some(node.clone());
                self.tail = Some(node);
            }
        }
        self.len += 1;
    }

    pub fn push_front(&mut self, val: T) {
        let node = Arc::new(Mutex::new(Node::new(val)));
        match self.tail.take() {
            Some(tail) => {
                tail.lock().unwrap().prev = Some(node.clone());
                node.lock().unwrap().next = Some(tail);
                self.tail = Some(node);
            }
            None => {
                self.tail = Some(node.clone());
                self.head = Some(node);
            }
        }
        self.len += 1;
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.head.take().and_then(|head| {
            let popped = head.lock().unwrap().val;
            match head.lock().unwrap().prev.take() {
                Some(node) => {
                    node.lock().unwrap().next.take();
                    self.head = Some(node);
                }
                None => {
                    self.head.take();
                }
            }
            self.len -= 1;
            Some(popped)
        })
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.tail.take().and_then(|tail| {
            let popped = tail.lock().unwrap().val;
            match tail.lock().unwrap().next.take() {
                Some(node) => {
                    node.lock().unwrap().prev.take();
                    self.tail = Some(node);
                }
                None => {
                    self.tail.take();
                }
            }
            self.len -= 1;
            Some(popped)
        })
    }

    pub fn back(&self) -> Option<T> {
        self.head.as_ref().map(|head| head.lock().unwrap().val)
        // match &self.head {
        //     Some(head) => Some(head.lock().unwrap().val),
        //     None => None,
        // }
    }

    pub fn front(&self) -> Option<T> {
        self.tail.as_ref().map(|head| head.lock().unwrap().val)
    }
}

impl<T> Default for LinkedList<T>
where
    T: Clone + Copy,
{
    fn default() -> Self {
        Self::new()
    }
}

pub struct IntoIter<T>(LinkedList<T>);

impl<T> Iterator for IntoIter<T>
where
    T: Copy + Clone,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front()
    }
}

impl<T> IntoIterator for LinkedList<T>
where
    T: Copy + Clone,
{
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self)
    }
}
