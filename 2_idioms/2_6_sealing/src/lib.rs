pub mod my_error;
pub mod my_iterator_ext;

pub use self::{my_error::MyError, my_iterator_ext::MyIteratorExt};

// ============================================================================
// Module-level sealing proof tests
// ============================================================================
//
// These commented implementations prove that the traits are sealed at module
// level. Uncommenting them will cause compilation errors.

// Proof 1: MyIteratorExt is fully sealed
// ---------------------------------------
// The following code fails to compile because external modules cannot
// name `my_iterator_ext::format::Sealed`, which is a required supertrait.
//
// Error: the trait bound `test_iterator_seal::ExternalIterator:
//        my_iterator_ext::format::Sealed` is not satisfied
/*
mod test_iterator_seal {
    use super::MyIteratorExt;

    struct ExternalIterator {
        count: usize,
    }

    impl Iterator for ExternalIterator {
        type Item = usize;

        fn next(&mut self) -> Option<Self::Item> {
            if self.count < 5 {
                self.count += 1;
                Some(self.count)
            } else {
                None
            }
        }
    }

    // ERROR: cannot implement MyIteratorExt because format::Sealed is private
    impl MyIteratorExt for ExternalIterator {}
}
*/

// Proof 2: MyError::type_id() is sealed (partial sealing)
// --------------------------------------------------------
// The MyError trait CAN be implemented externally (proving it's not fully
// sealed), but the type_id() method CANNOT be overridden because it delegates
// to the sealed `my_error::sealed::Sealed::type_id_impl()` method.
//
// This code compiles successfully, showing external implementation is allowed:
/*
mod test_error_seal {
    use std::fmt;
    use super::MyError;

    #[derive(Debug)]
    struct ExternalError {
        message: String,
    }

    impl fmt::Display for ExternalError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "External error: {}", self.message)
        }
    }

    // This works - trait can be implemented externally
    impl MyError for ExternalError {
        fn source(&self) -> Option<&(dyn MyError + 'static)> {
            None
        }

        // Attempting to override type_id() fails because we cannot
        // access my_error::sealed::Sealed to override type_id_impl()
        //
        // Even if we try to override type_id() directly, it just calls
        // self.type_id_impl() which we cannot override, so our override
        // would still call the sealed implementation.
    }
}
*/
