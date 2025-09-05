use std::sync::{Arc, Mutex, MutexGuard, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

struct GlobalStack<T> {
    data: Arc<RwLock<Vec<T>>>,
}

impl<T> GlobalStack<T> {
    pub fn new(vec: Vec<T>) -> Self {
        Self {
            data: Arc::new(RwLock::new(vec)),
        }
    }

    pub fn push(&self, value: T) -> Result<(), PoisonError<RwLockWriteGuard<'_, Vec<T>>>> {
        match self.data.write() {
            Ok(mut vec) => {
                vec.push(value);
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    pub fn get(
        &self,
    ) -> Result<RwLockReadGuard<'_, Vec<T>>, PoisonError<RwLockReadGuard<'_, Vec<T>>>> {
        match self.data.read() {
            Ok(vec) => Ok(vec),
            Err(err) => Err(err),
        }
    }
}

impl<T> Clone for GlobalStack<T> {
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
        }
    }
}

fn main() {
    let stack = GlobalStack::new(vec![1, 2, 3]);
    let another = stack.clone();
    stack.push(4).unwrap();

    println!("stack: {:?}", another.get());
}
