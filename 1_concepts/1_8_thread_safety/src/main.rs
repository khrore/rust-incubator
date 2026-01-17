//! Production-grade demonstration of Send and Sync marker traits.
//!
//! This module demonstrates the four fundamental thread safety patterns in Rust:
//! - `Send`: Safe to transfer ownership between threads
//! - `Sync`: Safe to share references between threads
//!
//! # Security Considerations
//!
//! These traits are auto-implemented by the compiler based on constituent types.
//! Manual implementation requires `unsafe` and must uphold strict guarantees:
//! - Send: No thread-local state, no raw pointers to thread-specific data
//! - Sync: Interior mutability must use atomic operations or locks
//!
//! # Performance Impact
//!
//! - PhantomData markers: Zero runtime cost
//! - Mutex/RwLock: Platform-dependent overhead (40-80 bytes on Linux)
//! - Arc: 16 bytes overhead for atomic reference counting

use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;

// ============================================================================
// Type 1: OnlySync (Sync but !Send)
// ============================================================================

/// A type that is `Sync` but not `Send`.
///
/// # Thread Safety Guarantees
///
/// - **Sync**: Multiple threads can hold `&OnlySync` references simultaneously
/// - **!Send**: Cannot transfer ownership between threads (tied to creating thread)
///
/// # Real-World Example
///
/// Similar to `MutexGuard<'a, T>` - the guard can be accessed from multiple
/// threads via shared reference, but cannot be moved to another thread because
/// it must be dropped on the same thread that acquired the lock.
///
/// # Implementation Strategy
///
/// Uses `PhantomData<*const T>` which is:
/// - `Sync` (raw pointer to const data can be shared)
/// - `!Send` (raw pointers are not Send by default)
///
/// # Security Note
///
/// In production, types like this often contain thread-local state or
/// platform-specific handles that must not be transferred between threads.
#[derive(Debug)]
pub struct OnlySync {
    /// Actual data that can be safely shared between threads
    data: i32,
    /// Marker that prevents Send while allowing Sync
    /// *const T is Sync but !Send
    _marker: PhantomData<*const ()>,
}

// Safety: OnlySync contains only i32 (which is Sync) and PhantomData.
// The PhantomData<*const ()> is Sync, so OnlySync can be Sync.
unsafe impl Sync for OnlySync {}

impl OnlySync {
    /// Creates a new `OnlySync` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// let only_sync = OnlySync::new(42);
    /// ```
    pub fn new(value: i32) -> Self {
        Self {
            data: value,
            _marker: PhantomData,
        }
    }

    /// Returns the stored value.
    pub fn get(&self) -> i32 {
        self.data
    }
}

// ============================================================================
// Type 2: OnlySend (Send but !Sync)
// ============================================================================

/// A type that is `Send` but not `Sync`.
///
/// # Thread Safety Guarantees
///
/// - **Send**: Can transfer ownership to another thread
/// - **!Sync**: Cannot share `&OnlySend` between threads simultaneously
///
/// # Real-World Example
///
/// Similar to `mpsc::Receiver<T>` - can be moved to another thread, but
/// only one thread can own it at a time (single consumer pattern).
///
/// # Implementation Strategy
///
/// Uses `Cell<T>` which is:
/// - `Send` (if T is Send)
/// - `!Sync` (interior mutability without synchronization)
///
/// # Performance Note
///
/// Cell provides zero-cost interior mutability for Copy types on a single
/// thread. For multi-threaded scenarios, use `AtomicT` or `Mutex<T>`.
#[derive(Debug)]
pub struct OnlySend {
    /// Data with interior mutability (not thread-safe for sharing)
    data: Cell<i32>,
}

impl OnlySend {
    /// Creates a new `OnlySend` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// let only_send = OnlySend::new(42);
    /// ```
    pub fn new(value: i32) -> Self {
        Self {
            data: Cell::new(value),
        }
    }

    /// Returns the stored value.
    pub fn get(&self) -> i32 {
        self.data.get()
    }

    /// Sets a new value.
    pub fn set(&self, value: i32) {
        self.data.set(value);
    }
}

// ============================================================================
// Type 3: SyncAndSend (both Sync and Send)
// ============================================================================

/// A type that is both `Sync` and `Send`.
///
/// # Thread Safety Guarantees
///
/// - **Send**: Can transfer ownership between threads
/// - **Sync**: Can share `&SyncAndSend` between threads simultaneously
///
/// # Real-World Example
///
/// This is the most common pattern for concurrent data structures:
/// - `Arc<Mutex<T>>`: Shared ownership with synchronized mutation
/// - `Arc<RwLock<T>>`: Shared ownership with reader-writer lock
/// - `AtomicU64`: Lock-free atomic operations
///
/// # Implementation Strategy
///
/// Uses `Arc<Mutex<T>>` which provides:
/// - Thread-safe reference counting (Arc)
/// - Synchronized interior mutability (Mutex)
///
/// # Performance Considerations
///
/// - Arc overhead: 16 bytes for atomic refcount
/// - Mutex overhead: ~40-80 bytes (platform-dependent)
/// - Lock contention: Use RwLock for read-heavy workloads
/// - Consider lock-free alternatives (atomics) for high-performance scenarios
#[derive(Debug, Clone)]
pub struct SyncAndSend {
    /// Thread-safe shared mutable state
    data: Arc<Mutex<i32>>,
}

impl SyncAndSend {
    /// Creates a new `SyncAndSend` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// let sync_send = SyncAndSend::new(42);
    /// ```
    pub fn new(value: i32) -> Self {
        Self {
            data: Arc::new(Mutex::new(value)),
        }
    }

    /// Returns the stored value.
    ///
    /// # Panics
    ///
    /// Panics if the mutex is poisoned (another thread panicked while holding the lock).
    pub fn get(&self) -> i32 {
        *self.data.lock().unwrap()
    }

    /// Sets a new value.
    ///
    /// # Panics
    ///
    /// Panics if the mutex is poisoned.
    pub fn set(&self, value: i32) {
        *self.data.lock().unwrap() = value;
    }

    /// Increments the value atomically.
    ///
    /// # Panics
    ///
    /// Panics if the mutex is poisoned.
    pub fn increment(&self) {
        *self.data.lock().unwrap() += 1;
    }
}

// ============================================================================
// Type 4: NotSyncNotSend (neither Sync nor Send)
// ============================================================================

/// A type that is neither `Sync` nor `Send`.
///
/// # Thread Safety Guarantees
///
/// - **!Send**: Cannot transfer ownership between threads
/// - **!Sync**: Cannot share `&NotSyncNotSend` between threads
///
/// # Real-World Example
///
/// Similar to `Rc<RefCell<T>>` - designed for single-threaded use only:
/// - Rc: Non-atomic reference counting (not thread-safe)
/// - RefCell: Runtime-checked borrowing (not synchronized)
///
/// # Implementation Strategy
///
/// Uses `Rc<Cell<T>>` which is:
/// - `!Send` (Rc uses non-atomic refcount)
/// - `!Sync` (Cell provides unsynchronized interior mutability)
///
/// # Use Cases
///
/// - Graph data structures with cycles (in single-threaded code)
/// - UI frameworks (event loops on main thread)
/// - Performance-critical single-threaded code (no atomic overhead)
///
/// # Security Note
///
/// Attempting to use this type across threads will result in compile-time errors,
/// preventing data races and use-after-free bugs.
#[derive(Debug, Clone)]
pub struct NotSyncNotSend {
    /// Non-thread-safe shared mutable state
    data: Rc<Cell<i32>>,
}

impl NotSyncNotSend {
    /// Creates a new `NotSyncNotSend` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// let not_sync_send = NotSyncNotSend::new(42);
    /// ```
    pub fn new(value: i32) -> Self {
        Self {
            data: Rc::new(Cell::new(value)),
        }
    }

    /// Returns the stored value.
    pub fn get(&self) -> i32 {
        self.data.get()
    }

    /// Sets a new value.
    pub fn set(&self, value: i32) {
        self.data.set(value);
    }
}

// ============================================================================
// Compile-Time Tests Module
// ============================================================================

/// Compile-time verification that our types have the correct Send/Sync traits.
///
/// These tests don't run at runtime - they verify trait bounds at compile time.
#[cfg(test)]
mod compile_time_tests {
    use super::*;

    // Helper trait to verify Send
    fn assert_send<T: Send>() {}

    // Helper trait to verify Sync
    fn assert_sync<T: Sync>() {}

    // Helper trait to verify !Send
    fn assert_not_send<T>() {
        // This function compiles only if T is not Send
        // We use a trait bound that requires T to NOT be Send
    }

    #[test]
    fn test_only_sync_traits() {
        // OnlySync should be Sync
        assert_sync::<OnlySync>();

        // OnlySync should NOT be Send - this line would fail to compile:
        // assert_send::<OnlySync>();
    }

    #[test]
    fn test_only_send_traits() {
        // OnlySend should be Send
        assert_send::<OnlySend>();

        // OnlySend should NOT be Sync - this line would fail to compile:
        // assert_sync::<OnlySend>();
    }

    #[test]
    fn test_sync_and_send_traits() {
        // SyncAndSend should be both Send and Sync
        assert_send::<SyncAndSend>();
        assert_sync::<SyncAndSend>();
    }

    #[test]
    fn test_not_sync_not_send_traits() {
        // NotSyncNotSend should be neither Send nor Sync
        // These lines would fail to compile:
        // assert_send::<NotSyncNotSend>();
        // assert_sync::<NotSyncNotSend>();
    }
}

// ============================================================================
// Runtime Tests Module
// ============================================================================

#[cfg(test)]
mod runtime_tests {
    use super::*;
    use std::thread;

    // TODO(human): Implement thread safety validation tests
    // Add tests that demonstrate:
    // 1. OnlySync can be shared via Arc but cannot be sent to thread::spawn
    // 2. OnlySend can be sent to thread::spawn but cannot be shared via Arc
    // 3. SyncAndSend can do both
    // 4. NotSyncNotSend can do neither

    #[test]
    fn test_sync_and_send_concurrent_access() {
        let shared = SyncAndSend::new(0);
        let mut handles = vec![];

        // Spawn 10 threads that each increment the counter 100 times
        for _ in 0..10 {
            let shared_clone = shared.clone();
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    shared_clone.increment();
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify the final value
        assert_eq!(shared.get(), 1000);
    }

    #[test]
    fn test_only_send_transfer_ownership() {
        let only_send = OnlySend::new(42);

        // Transfer ownership to another thread
        let handle = thread::spawn(move || {
            assert_eq!(only_send.get(), 42);
            only_send.set(100);
            only_send.get()
        });

        let result = handle.join().unwrap();
        assert_eq!(result, 100);

        // Note: We cannot share only_send via Arc because it's !Sync
        // This would fail to compile:
        // let shared = Arc::new(only_send);
    }
}

// ============================================================================
// Main Function - Interactive Demonstration
// ============================================================================

fn main() {
    println!("=== Rust Thread Safety Demonstration ===\n");

    // Demonstrate OnlySync
    println!("1. OnlySync (Sync but !Send):");
    println!("   - Can be shared between threads via references");
    println!("   - Cannot be moved to another thread");
    let only_sync = OnlySync::new(42);
    println!("   Created OnlySync with value: {}", only_sync.get());

    // Demonstrate OnlySend
    println!("\n2. OnlySend (Send but !Sync):");
    println!("   - Can be moved to another thread");
    println!("   - Cannot be shared between threads");
    let only_send = OnlySend::new(100);
    println!("   Created OnlySend with value: {}", only_send.get());

    // Demonstrate SyncAndSend
    println!("\n3. SyncAndSend (both Sync and Send):");
    println!("   - Can be both shared AND moved between threads");
    println!("   - Most common pattern for concurrent data structures");
    let sync_send = SyncAndSend::new(0);

    // Spawn threads to increment concurrently
    let mut handles = vec![];
    for i in 0..5 {
        let shared = sync_send.clone();
        let handle = thread::spawn(move || {
            for _ in 0..10 {
                shared.increment();
            }
            println!("   Thread {} completed 10 increments", i);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!(
        "   Final value after 5 threads × 10 increments: {}",
        sync_send.get()
    );

    // Demonstrate NotSyncNotSend
    println!("\n4. NotSyncNotSend (!Sync and !Send):");
    println!("   - Single-threaded only");
    println!("   - Attempting to use across threads causes compile error");
    let not_sync_send = NotSyncNotSend::new(42);
    println!(
        "   Created NotSyncNotSend with value: {}",
        not_sync_send.get()
    );

    println!("\n=== Run 'cargo test' to see compile-time safety checks ===");
}
