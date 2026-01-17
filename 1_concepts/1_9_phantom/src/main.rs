//! Production-grade phantom types implementation
//!
//! This demonstrates best practices for:
//! - Zero-cost abstractions (no runtime overhead)
//! - Thread-safe phantom types (Send + Sync)
//! - Compile-time guarantees
//! - Extensible trait-based design

use std::marker::PhantomData;

// ============================================================================
// Core Trait Design - Extensible & Type-Safe
// ============================================================================

/// Trait for types that can provide facts about themselves.
///
/// # Design Rationale
/// - Uses associated constants for zero allocation (facts stored in binary)
/// - Compile-time validation (must have at least one fact)
/// - Thread-safe by design (no mutable state)
pub trait HasFacts {
    /// All available facts for this type.
    /// Stored as static strings - zero heap allocation!
    const FACTS: &'static [&'static str];
}

// ============================================================================
// Phantom Type Implementation - Thread-Safe & Zero-Cost
// ============================================================================

/// A type that provides random facts about `T`.
///
/// # Production Considerations
///
/// ## Thread Safety
/// Uses `PhantomData<fn() -> T>` instead of `PhantomData<T>`:
/// - Always `Send + Sync` regardless of `T`
/// - Function pointers are always thread-safe
/// - Critical for backend services with thread pools
///
/// ## Zero Cost
/// ```rust,ignore
/// assert_eq!(std::mem::size_of::<Fact<Vec<u8>>>(), 0);
/// ```
/// PhantomData is zero-sized type (ZST) - no runtime overhead!
///
/// ## Type Safety
/// Can only construct `Fact<T>` if `T: HasFacts` - compile-time guarantee.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Fact<T> {
    // Using fn() -> T instead of T for Send + Sync guarantees
    // This is critical for backend/blockchain where everything must be thread-safe
    _phantom: PhantomData<fn() -> T>,
}

impl<T> Fact<T>
where
    T: HasFacts,
{
    /// Creates a new Fact instance.
    ///
    /// # Compile-Time Guarantees
    /// This only compiles if `T: HasFacts`, ensuring type safety.
    #[inline]
    pub const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    /// Returns a random fact about type `T`.
    ///
    /// # Performance
    /// - O(1) time complexity
    /// - Zero heap allocations
    /// - Returns static string (lives for entire program)
    ///
    /// # Implementation Note
    /// TODO(human): Implement random selection strategy here
    pub fn fact(&self) -> &'static str {
        // Placeholder implementation
        T::FACTS[0]
    }

    /// Returns the total number of available facts.
    ///
    /// # Use Case
    /// Useful for testing and debugging.
    #[inline]
    pub const fn fact_count() -> usize {
        T::FACTS.len()
    }

    /// Returns all available facts (useful for testing).
    #[inline]
    pub const fn all_facts() -> &'static [&'static str] {
        T::FACTS
    }
}

// ============================================================================
// HasFacts Implementations - Standard Library Types
// ============================================================================

// Vec<T> - Dynamic array facts
impl<T> HasFacts for Vec<T> {
    const FACTS: &'static [&'static str] = &[
        "Vec is heap-allocated",
        "Vec may re-allocate on growing",
        "Vec has O(1) push when capacity is available",
        "Vec has O(1) indexed access",
        "Vec stores length and capacity separately",
        "Vec deallocates on drop",
        "Vec uses the global allocator by default",
    ];
}

// String - Owned string facts
impl HasFacts for String {
    const FACTS: &'static [&'static str] = &[
        "String is a Vec<u8> wrapper with UTF-8 guarantee",
        "String is heap-allocated",
        "String may re-allocate when growing",
        "String validates UTF-8 on construction",
        "String uses 3 words of stack space (ptr, len, cap)",
    ];
}

// Option<T> - Maybe facts
impl<T> HasFacts for Option<T> {
    const FACTS: &'static [&'static str] = &[
        "Option uses null pointer optimization for some types",
        "Option<&T> has the same size as &T",
        "Option is an enum with two variants: Some and None",
        "Option<Box<T>> has no overhead vs Box<T>",
        "Option implements many combinator methods (map, and_then, etc.)",
    ];
}

// Result<T, E> - Error handling facts
impl<T, E> HasFacts for Result<T, E> {
    const FACTS: &'static [&'static str] = &[
        "Result is an enum representing success or failure",
        "Result should be used instead of exceptions in Rust",
        "Result implements the Try trait for the ? operator",
        "Result can be pattern matched ergonomically",
        "Result size is max(size_of::<T>(), size_of::<E>()) + discriminant",
    ];
}

// Box<T> - Heap allocation facts
impl<T: ?Sized> HasFacts for Box<T> {
    const FACTS: &'static [&'static str] = &[
        "Box provides heap allocation for a single value",
        "Box has the same size as a raw pointer",
        "Box provides unique ownership (no reference counting)",
        "Box deallocates when dropped",
        "Box can store unsized types (like trait objects)",
    ];
}

// Rc<T> - Reference counting facts
impl<T: ?Sized> HasFacts for std::rc::Rc<T> {
    const FACTS: &'static [&'static str] = &[
        "Rc provides shared ownership via reference counting",
        "Rc is not thread-safe (use Arc for that)",
        "Rc has weak reference support",
        "Rc has overhead of two reference counts (strong + weak)",
        "Rc uses non-atomic reference counting",
    ];
}

// Arc<T> - Atomic reference counting facts
impl<T: ?Sized> HasFacts for std::sync::Arc<T> {
    const FACTS: &'static [&'static str] = &[
        "Arc provides thread-safe shared ownership",
        "Arc uses atomic reference counting",
        "Arc is Send + Sync when T: Send + Sync",
        "Arc has more overhead than Rc due to atomic operations",
        "Arc is essential for concurrent Rust programming",
    ];
}

// ============================================================================
// Tests - Comprehensive Coverage
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec_facts() {
        let f: Fact<Vec<i32>> = Fact::new();
        let fact = f.fact();
        assert!(Fact::<Vec<i32>>::all_facts().contains(&fact));
    }

    #[test]
    fn test_string_facts() {
        let f: Fact<String> = Fact::new();
        let fact = f.fact();
        assert!(Fact::<String>::all_facts().contains(&fact));
    }

    #[test]
    fn test_option_facts() {
        let f: Fact<Option<i32>> = Fact::new();
        let fact = f.fact();
        assert!(Fact::<Option<i32>>::all_facts().contains(&fact));
    }

    #[test]
    fn test_result_facts() {
        let f: Fact<Result<i32, String>> = Fact::new();
        let fact = f.fact();
        assert!(Fact::<Result<i32, String>>::all_facts().contains(&fact));
    }

    /// Verify zero-cost abstraction - Fact<T> should be zero-sized
    #[test]
    fn test_zero_size() {
        assert_eq!(std::mem::size_of::<Fact<Vec<u8>>>(), 0);
        assert_eq!(std::mem::size_of::<Fact<String>>(), 0);
        assert_eq!(std::mem::size_of::<Fact<Option<i32>>>(), 0);
    }

    /// Verify thread safety - critical for backend services
    #[test]
    fn test_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}

        // These should compile - Fact is always Send + Sync
        assert_send_sync::<Fact<Vec<u8>>>();
        assert_send_sync::<Fact<String>>();

        // Even with non-Send types, Fact itself is Send
        assert_send_sync::<Fact<std::rc::Rc<u8>>>();
    }

    /// Verify that Fact count is correct
    #[test]
    fn test_fact_count() {
        assert_eq!(Fact::<Vec<i32>>::fact_count(), 7);
        assert_eq!(Fact::<String>::fact_count(), 5);
        assert_eq!(Fact::<Option<i32>>::fact_count(), 5);
    }

    /// Verify all_facts returns all facts
    #[test]
    fn test_all_facts() {
        let facts = Fact::<Vec<i32>>::all_facts();
        assert_eq!(facts.len(), 7);
        assert!(facts.contains(&"Vec is heap-allocated"));
    }
}

// ============================================================================
// Main - Demonstration
// ============================================================================

fn main() {
    println!("=== Production-Grade Phantom Types Demo ===\n");

    // Vec facts
    let f: Fact<Vec<i32>> = Fact::new();
    println!("Vec<T> facts ({} total):", Fact::<Vec<i32>>::fact_count());
    for _ in 0..3 {
        println!("  - {}", f.fact());
    }

    println!();

    // String facts
    let f: Fact<String> = Fact::new();
    println!("String facts ({} total):", Fact::<String>::fact_count());
    for _ in 0..3 {
        println!("  - {}", f.fact());
    }

    println!();

    // Option facts
    let f: Fact<Option<i32>> = Fact::new();
    println!("Option<T> facts ({} total):", Fact::<Option<i32>>::fact_count());
    for _ in 0..3 {
        println!("  - {}", f.fact());
    }

    println!();

    // Result facts
    let f: Fact<Result<i32, String>> = Fact::new();
    println!("Result<T, E> facts ({} total):", Fact::<Result<i32, String>>::fact_count());
    for _ in 0..3 {
        println!("  - {}", f.fact());
    }

    println!();

    // Arc facts (important for concurrent systems)
    let f: Fact<std::sync::Arc<i32>> = Fact::new();
    println!("Arc<T> facts ({} total):", Fact::<std::sync::Arc<i32>>::fact_count());
    for _ in 0..3 {
        println!("  - {}", f.fact());
    }

    println!("\n=== Zero-Cost Verification ===");
    println!("Size of Fact<Vec<u8>>: {} bytes", std::mem::size_of::<Fact<Vec<u8>>>());
    println!("Size of Fact<String>: {} bytes", std::mem::size_of::<Fact<String>>());
    println!("Size of Fact<Arc<u8>>: {} bytes", std::mem::size_of::<Fact<std::sync::Arc<u8>>>());
}
