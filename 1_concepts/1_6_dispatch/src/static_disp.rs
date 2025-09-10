use crate::base::{Storage, User};
use std::marker::PhantomData;

struct UserRepo<K, S> {
    storage: S,
    _data: PhantomData<K>,
}

impl<K, S: Storage<K, User>> UserRepo<K, S> {
    pub fn new(storage: S) -> Self {
        Self {
            storage,
            _data: PhantomData,
        }
    }

    pub fn set(&mut self, key: K, val: User) {
        self.storage.set(key, val);
    }

    pub fn get(&self, key: &K) -> Option<&User> {
        self.storage.get(key)
    }

    pub fn remove(&mut self, key: &K) -> Option<User> {
        self.storage.remove(key)
    }
}
