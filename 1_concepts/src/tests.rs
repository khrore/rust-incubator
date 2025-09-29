use crate::concurrent_list::ConcurrentList;
use std::{
    sync::{Arc, Mutex},
    thread,
};

#[cfg(test)]
use crate::linked_list::LinkedList;

#[test]
fn test_empty() {
    let list: LinkedList<i32> = LinkedList::new();
    assert_eq!(list.back(), None);
    assert_eq!(list.front(), None);
}

#[test]
fn test_one_added() {
    let mut list = LinkedList::new();
    list.push_back(10);
    assert_eq!(list.back(), Some(10));
    assert_eq!(list.front(), Some(10));
}

#[test]
fn test_two_added() {
    let mut list = LinkedList::new();
    list.push_back(10);
    list.push_back(20);
    assert_eq!(list.back(), Some(20));
    assert_eq!(list.front(), Some(10));
}

#[test]
fn test_three_added() {
    let mut list = LinkedList::new();
    list.push_back(20);
    list.push_back(30);
    list.push_front(10);
    assert_eq!(list.back(), Some(30));
    assert_eq!(list.front(), Some(10));
}

#[test]
fn test_one_pop() {
    let mut list = LinkedList::new();
    list.push_back(20);
    list.push_back(30);
    list.push_front(10);
    let val = list.pop_back().unwrap();
    assert_eq!(val, 30);
    assert_eq!(list.back(), Some(20));
    assert_eq!(list.front(), Some(10));
}

#[test]
fn test_two_pop() {
    let mut list = LinkedList::new();
    list.push_back(20);
    list.push_back(30);
    list.push_back(40);
    list.push_front(10);
    let val = list.pop_back().unwrap();
    let val2 = list.pop_front().unwrap();
    assert_eq!(val, 40);
    assert_eq!(val2, 10);
    assert_eq!(list.back(), Some(30));
    assert_eq!(list.front(), Some(20));
}

#[test]
fn test_multithreaded() {
    let list: ConcurrentList<i32> = Arc::new(Mutex::new(LinkedList::new()));
    thread::scope(|s| {
        for i in 0..8 {
            let list = Arc::clone(&list);
            s.spawn(move || {
                list.lock().unwrap().push_back(i);
            });
        }
    });
    let list = Arc::try_unwrap(list)
        .unwrap()
        .into_inner()
        .unwrap()
        .into_iter()
        .collect::<Vec<_>>();

    assert!(list.contains(&0));
    assert!(list.contains(&1));
    assert!(list.contains(&2));
    assert!(list.contains(&3));
    assert!(list.contains(&4));
    assert!(list.contains(&5));
    assert!(list.contains(&6));
    assert!(list.contains(&7));
    assert!(!list.contains(&8));
}
