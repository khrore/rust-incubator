use crate::base::{Storage, User};

pub struct UserRepo<K>(Box<dyn Storage<K, User>>);

impl<K> UserRepo<K> {
    pub fn new<S>(storage: S) -> Self
    where
        S: Storage<K, User> + 'static,
    {
        UserRepo(Box::new(storage))
    }

    pub fn set(&mut self, key: K, val: User) {
        self.0.set(key, val);
    }

    pub fn get(&self, key: &K) -> Option<&User> {
        self.0.get(key)
    }

    pub fn remove(&mut self, key: &K) -> Option<User> {
        self.0.remove(key)
    }
}
