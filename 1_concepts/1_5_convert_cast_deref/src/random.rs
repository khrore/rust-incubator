use rand::{Rng, seq::SliceRandom};
use std::ops::Deref;

const SIZE: usize = 3;

pub struct Random<T>([T; SIZE]);

impl<T> Random<T> {
    pub fn new(v1: T, v2: T, v3: T) -> Self {
        Self([v1, v2, v3])
    }
}

impl<T> Deref for Random<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        let index = rand::rng().random_range(..=SIZE);
        &self.0[index]
    }
}
