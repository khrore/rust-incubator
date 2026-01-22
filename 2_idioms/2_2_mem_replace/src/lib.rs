// ===== Optimized Implementation (mem::swap/take) =====

pub mod optimized {
    use std::mem;

    #[derive(Clone, Debug, PartialEq)]
    pub struct Trinity<T> {
        pub a: T,
        pub b: T,
        pub c: T,
    }

    impl<T> Trinity<T> {
        /// Rotates elements: (a, b, c) → (b, c, a)
        ///
        /// Uses `mem::swap` for zero-cost rotation without cloning.
        pub fn rotate(&mut self) {
            mem::swap(&mut self.a, &mut self.b);
            mem::swap(&mut self.b, &mut self.c);
        }
    }

    #[derive(Debug)]
    pub struct Solver<T> {
        pub expected: Trinity<T>,
        pub unsolved: Vec<Trinity<T>>,
    }

    impl<T: PartialEq> Solver<T> {
        pub fn new(expected: Trinity<T>, unsolved: Vec<Trinity<T>>) -> Self {
            Self { expected, unsolved }
        }

        /// Resolves trinities using mem::take to avoid cloning
        pub fn resolve(&mut self) {
            let mut all_items = mem::take(&mut self.unsolved);

            all_items.retain_mut(|t| {
                for _ in 0..3 {
                    if *t == self.expected {
                        return false;
                    }
                    t.rotate();
                }
                true
            });

            self.unsolved = all_items;
        }
    }
}

// ===== Original Implementation (clone-based) =====

pub mod original {
    #[derive(Clone, Debug, PartialEq)]
    pub struct Trinity<T> {
        pub a: T,
        pub b: T,
        pub c: T,
    }

    impl<T: Clone> Trinity<T> {
        /// Original rotate implementation using clones
        pub fn rotate(&mut self) {
            let a = self.a.clone();
            let b = self.b.clone();
            let c = self.c.clone();
            self.a = b;
            self.b = c;
            self.c = a;
        }
    }

    #[derive(Debug)]
    pub struct Solver<T> {
        pub expected: Trinity<T>,
        pub unsolved: Vec<Trinity<T>>,
    }

    impl<T: Clone + PartialEq> Solver<T> {
        pub fn new(expected: Trinity<T>, unsolved: Vec<Trinity<T>>) -> Self {
            Self { expected, unsolved }
        }

        /// Original resolve implementation using clones
        pub fn resolve(&mut self) {
            let mut unsolved = Vec::with_capacity(self.unsolved.len());
            'l: for t in self.unsolved.iter_mut() {
                for _ in 0..3 {
                    if *t == self.expected {
                        continue 'l;
                    }
                    t.rotate();
                }
                unsolved.push(t.clone())
            }
            self.unsolved = unsolved;
        }
    }
}

// Re-export optimized version for convenience
pub use optimized::{Solver, Trinity};
