/// Basic expectations for error values.
///
/// Simplified version of [`std::error::Error`].
use std::{
    any::TypeId,
    fmt::{Debug, Display},
};

mod sealed {
    use std::any::TypeId;

    /// Sealed helper trait for `type_id()` implementation.
    ///
    /// This trait cannot be named outside this module, making it impossible
    /// to override the `type_id_impl()` method from external code.
    pub trait Sealed {
        fn type_id_impl(&self) -> TypeId
        where
            Self: 'static,
        {
            TypeId::of::<Self>()
        }
    }

    /// Blanket implementation for all types.
    ///
    /// This allows any type to implement `MyError`, but the `type_id()`
    /// method remains sealed via delegation to `type_id_impl()`.
    impl<T: ?Sized> Sealed for T {}
}

/// Basic expectations for error values.
///
/// This trait is **partially sealed**: it can be implemented externally,
/// but the [`type_id()`](MyError::type_id) method is sealed and cannot be
/// overridden.
///
/// # Partial Sealing Proof
///
/// External implementations are allowed:
///
/// ```rust
/// use std::fmt;
/// use step_2_6::MyError;
///
/// #[derive(Debug)]
/// struct MyCustomError;
///
/// impl fmt::Display for MyCustomError {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "My custom error")
///     }
/// }
///
/// // This works - trait can be implemented externally
/// impl MyError for MyCustomError {}
/// ```
///
/// The `type_id()` method is sealed through delegation. While it's technically
/// possible to override it (the compiler won't prevent it), doing so is
/// memory-unsafe and the default implementation delegates to a sealed helper
/// that cannot be overridden:
///
/// ```rust
/// use std::fmt;
/// use step_2_6::MyError;
///
/// #[derive(Debug)]
/// struct WellBehavedError;
///
/// impl fmt::Display for WellBehavedError {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "Well-behaved error")
///     }
/// }
///
/// impl MyError for WellBehavedError {
///     // We can override source()
///     fn source(&self) -> Option<&(dyn MyError + 'static)> {
///         None
///     }
///
///     // We MUST NOT override type_id() - it's marked as memory-unsafe.
///     // The default implementation delegates to the sealed type_id_impl()
///     // which always returns the correct TypeId.
/// }
/// ```
///
/// Attempting to access the sealed implementation directly fails:
///
/// ```compile_fail
/// use step_2_6::my_error::sealed::Sealed;
/// // ERROR: module `sealed` is private
/// ```
pub trait MyError: Debug + Display + sealed::Sealed {
    /// The lower-level source of this error, if any.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::fmt;
    ///
    /// use step_2_6::MyError;
    ///
    /// #[derive(Debug)]
    /// struct SuperError {
    ///     source: SuperErrorSideKick,
    /// }
    ///
    /// impl fmt::Display for SuperError {
    ///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///         write!(f, "SuperError is here!")
    ///     }
    /// }
    ///
    /// impl MyError for SuperError {
    ///     fn source(&self) -> Option<&(dyn MyError + 'static)> {
    ///         Some(&self.source)
    ///     }
    /// }
    ///
    /// #[derive(Debug)]
    /// struct SuperErrorSideKick;
    ///
    /// impl fmt::Display for SuperErrorSideKick {
    ///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///         write!(f, "SuperErrorSideKick is here!")
    ///     }
    /// }
    ///
    /// impl MyError for SuperErrorSideKick {}
    ///
    /// fn get_super_error() -> Result<(), SuperError> {
    ///     Err(SuperError { source: SuperErrorSideKick })
    /// }
    ///
    /// fn main() {
    ///     match get_super_error() {
    ///         Err(e) => {
    ///             println!("Error: {e}");
    ///             println!("Caused by: {}", e.source().unwrap());
    ///         }
    ///         _ => println!("No error"),
    ///     }
    /// }
    /// ```
    fn source(&self) -> Option<&(dyn MyError + 'static)> {
        None
    }

    /// Gets the `TypeId` of `self`.
    ///
    /// __This is memory-unsafe to override in user code.__
    ///
    /// This method is sealed and cannot be overridden. It delegates to the
    /// sealed `type_id_impl()` method which external code cannot access.
    #[doc(hidden)]
    fn type_id(&self) -> TypeId
    where
        Self: 'static,
    {
        self.type_id_impl()
    }
}

impl<T: MyError + ?Sized> MyError for &T {
    fn source(&self) -> Option<&(dyn MyError + 'static)> {
        MyError::source(&**self)
    }
}
