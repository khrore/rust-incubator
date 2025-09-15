use std::{borrow::Cow, hash::Hash};

pub trait UserRepo {}

pub trait Storage<K, V> {
    fn set(&mut self, key: K, val: V);
    fn get(&self, key: &K) -> Option<&V>;
    fn remove(&mut self, key: &K) -> Option<V>;
}

#[derive(Hash, PartialEq, Eq, Debug)]
pub struct User {
    pub id: u64,
    pub email: Cow<'static, str>,
    pub activated: bool,
}
