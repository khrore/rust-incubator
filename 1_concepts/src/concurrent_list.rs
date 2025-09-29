use crate::linked_list::LinkedList;
use std::sync::{Arc, Mutex};

pub type ConcurrentList<T> = Arc<Mutex<LinkedList<T>>>;
