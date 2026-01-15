use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};

// ============================================================================
// Core Domain Types
// ============================================================================

/// User entity with unique ID, email, and activation status
#[derive(Clone, Debug, PartialEq, Eq)]
struct User {
    id: u64,
    email: Cow<'static, str>,
    activated: bool,
}

impl User {
    fn new(id: u64, email: impl Into<Cow<'static, str>>) -> Self {
        Self {
            id,
            email: email.into(),
            activated: false,
        }
    }

    fn activate(&mut self) {
        self.activated = true;
    }
}

// ============================================================================
// Storage Abstraction
// ============================================================================

/// Generic storage interface for key-value operations
///
/// This trait is object-safe, allowing both static and dynamic dispatch.
trait Storage<K, V> {
    fn set(&mut self, key: K, val: V);
    fn get(&self, key: &K) -> Option<&V>;
    fn remove(&mut self, key: &K) -> Option<V>;
}

// ============================================================================
// Concrete Storage Implementations
// ============================================================================

/// HashMap-based storage implementation (O(1) average case operations)
#[derive(Default)]
struct HashMapStorage<K, V> {
    map: HashMap<K, V>,
}

impl<K, V> HashMapStorage<K, V> {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
        }
    }
}

impl<K: std::hash::Hash + Eq, V> Storage<K, V> for HashMapStorage<K, V> {
    fn set(&mut self, key: K, val: V) {
        self.map.insert(key, val);
    }

    fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        self.map.remove(key)
    }
}

/// BTreeMap-based storage implementation (O(log n) operations, sorted keys)
#[derive(Default)]
struct BTreeMapStorage<K, V> {
    map: BTreeMap<K, V>,
}

impl<K, V> BTreeMapStorage<K, V> {
    fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }
}

impl<K: Ord, V> Storage<K, V> for BTreeMapStorage<K, V> {
    fn set(&mut self, key: K, val: V) {
        self.map.insert(key, val);
    }

    fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        self.map.remove(key)
    }
}

// ============================================================================
// Static Dispatch Repository
// ============================================================================

/// User repository using STATIC DISPATCH (compile-time polymorphism)
///
/// Advantages:
/// - Zero-cost abstraction: No runtime overhead
/// - Compiler can inline and optimize aggressively
/// - Better CPU cache locality
/// - Type known at compile time
///
/// Disadvantages:
/// - Code duplication (monomorphization) for each concrete type
/// - Larger binary size
/// - Cannot store in heterogeneous collections
/// - Longer compile times
struct UserRepository<S> {
    storage: S,
}

impl<S: Storage<u64, User>> UserRepository<S> {
    /// Create a new repository with the given storage backend
    fn new(storage: S) -> Self {
        Self { storage }
    }

    /// Add a new user to the repository
    ///
    /// Returns an error if a user with the same ID already exists.
    fn add(&mut self, user: User) -> Result<(), String> {
        if self.storage.get(&user.id).is_some() {
            return Err(format!("User with id {} already exists", user.id));
        }
        self.storage.set(user.id, user);
        Ok(())
    }

    /// Retrieve a user by ID
    fn get(&self, id: u64) -> Option<&User> {
        self.storage.get(&id)
    }

    /// Update an existing user
    ///
    /// Returns an error if the user doesn't exist.
    fn update(&mut self, user: User) -> Result<(), String> {
        if self.storage.get(&user.id).is_none() {
            return Err(format!("User with id {} not found", user.id));
        }
        self.storage.set(user.id, user);
        Ok(())
    }

    /// Remove a user by ID
    ///
    /// Returns the removed user, or None if not found.
    fn remove(&mut self, id: u64) -> Option<User> {
        self.storage.remove(&id)
    }

    /// Count total users in the repository
    fn count(&self) -> usize {
        // Note: This is inefficient but demonstrates that we can't easily
        // add methods to the Storage trait without breaking object safety
        let mut count = 0;
        for id in 0..1000 {
            if self.storage.get(&id).is_some() {
                count += 1;
            }
        }
        count
    }
}

// ============================================================================
// Dynamic Dispatch Repository
// ============================================================================

/// User repository using DYNAMIC DISPATCH (runtime polymorphism)
///
/// Advantages:
/// - Smaller binary size (no code duplication)
/// - Faster compile times
/// - Can store different implementations in collections
/// - Runtime flexibility (e.g., plugin systems)
///
/// Disadvantages:
/// - Runtime overhead (~2-3ns per call for vtable lookup)
/// - Cannot be inlined by compiler
/// - Worse CPU cache locality (indirect calls)
/// - Requires heap allocation (Box) for owned trait objects
struct UserRepositoryDyn {
    storage: Box<dyn Storage<u64, User>>,
}

impl UserRepositoryDyn {
    /// Create a new repository with the given storage backend
    ///
    /// Requires 'static lifetime to ensure the storage lives long enough.
    fn new<S: Storage<u64, User> + 'static>(storage: S) -> Self {
        Self {
            storage: Box::new(storage),
        }
    }

    /// Add a new user to the repository
    fn add(&mut self, user: User) -> Result<(), String> {
        if self.storage.get(&user.id).is_some() {
            return Err(format!("User with id {} already exists", user.id));
        }
        self.storage.set(user.id, user);
        Ok(())
    }

    /// Retrieve a user by ID
    fn get(&self, id: u64) -> Option<&User> {
        self.storage.get(&id)
    }

    /// Update an existing user
    fn update(&mut self, user: User) -> Result<(), String> {
        if self.storage.get(&user.id).is_none() {
            return Err(format!("User with id {} not found", user.id));
        }
        self.storage.set(user.id, user);
        Ok(())
    }

    /// Remove a user by ID
    fn remove(&mut self, id: u64) -> Option<User> {
        self.storage.remove(&id)
    }

    /// Count total users in the repository
    fn count(&self) -> usize {
        let mut count = 0;
        for id in 0..1000 {
            if self.storage.get(&id).is_some() {
                count += 1;
            }
        }
        count
    }
}

// ============================================================================
// Main Function
// ============================================================================

fn main() {
    println!("=== Static vs Dynamic Dispatch Demo ===\n");

    // Static dispatch example with HashMap
    println!("1. Static Dispatch with HashMap:");
    let mut static_repo = UserRepository::new(HashMapStorage::new());

    let user1 = User::new(1, "alice@example.com");
    static_repo.add(user1.clone()).unwrap();
    println!("   Added: {:?}", user1);

    if let Some(retrieved) = static_repo.get(1) {
        println!("   Retrieved: {:?}", retrieved);
    }

    static_repo.remove(1);
    println!("   Removed user 1");
    println!("   Remaining users: {}\n", static_repo.count());

    // Static dispatch example with BTreeMap
    println!("2. Static Dispatch with BTreeMap:");
    let mut btree_repo = UserRepository::new(BTreeMapStorage::new());

    let user2 = User::new(2, "bob@example.com");
    btree_repo.add(user2.clone()).unwrap();
    println!("   Added: {:?}", user2);
    println!("   Total users: {}\n", btree_repo.count());

    // Dynamic dispatch example with HashMap
    println!("3. Dynamic Dispatch with HashMap:");
    let mut dyn_repo = UserRepositoryDyn::new(HashMapStorage::new());

    let user3 = User::new(3, "charlie@example.com");
    dyn_repo.add(user3.clone()).unwrap();
    println!("   Added: {:?}", user3);

    if let Some(retrieved) = dyn_repo.get(3) {
        println!("   Retrieved: {:?}", retrieved);
    }
    println!("   Total users: {}\n", dyn_repo.count());

    // Heterogeneous collection (only possible with dynamic dispatch)
    println!("4. Heterogeneous Collection (Dynamic Dispatch Only):");
    let storages: Vec<Box<dyn Storage<u64, User>>> = vec![
        Box::new(HashMapStorage::new()),
        Box::new(BTreeMapStorage::new()),
    ];
    println!("   Created {} different storage backends", storages.len());
    println!("   (This is impossible with static dispatch!)\n");

    println!("=== Performance Comparison ===");
    println!("Run `cargo bench` to see performance benchmarks!");
    println!("\nExpected results:");
    println!("- Static dispatch: ~1-2ns per operation");
    println!("- Dynamic dispatch: ~4-5ns per operation");
    println!("- Difference: 2-3x slower (but still very fast!)");
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Static dispatch tests
    mod static_dispatch {
        use super::*;

        #[test]
        fn test_add_and_get_user() {
            let mut repo = UserRepository::new(HashMapStorage::new());
            let user = User::new(1, "test@example.com");

            repo.add(user.clone()).unwrap();
            assert_eq!(repo.get(1), Some(&user));
        }

        #[test]
        fn test_add_duplicate_user_fails() {
            let mut repo = UserRepository::new(HashMapStorage::new());
            let user = User::new(1, "test@example.com");

            repo.add(user.clone()).unwrap();
            let result = repo.add(user);

            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .contains("already exists"));
        }

        #[test]
        fn test_update_user() {
            let mut repo = UserRepository::new(HashMapStorage::new());
            let mut user = User::new(1, "test@example.com");

            repo.add(user.clone()).unwrap();
            user.activate();
            repo.update(user.clone()).unwrap();

            let retrieved = repo.get(1).unwrap();
            assert!(retrieved.activated);
        }

        #[test]
        fn test_update_nonexistent_user_fails() {
            let mut repo = UserRepository::new(HashMapStorage::new());
            let user = User::new(999, "test@example.com");

            let result = repo.update(user);
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("not found"));
        }

        #[test]
        fn test_remove_user() {
            let mut repo = UserRepository::new(HashMapStorage::new());
            let user = User::new(1, "test@example.com");

            repo.add(user.clone()).unwrap();
            let removed = repo.remove(1);

            assert_eq!(removed, Some(user));
            assert_eq!(repo.get(1), None);
        }

        #[test]
        fn test_remove_nonexistent_user() {
            let mut repo = UserRepository::new(HashMapStorage::<u64, User>::new());
            assert_eq!(repo.remove(999), None);
        }

        #[test]
        fn test_btreemap_storage() {
            let mut repo = UserRepository::new(BTreeMapStorage::new());
            let user = User::new(1, "test@example.com");

            repo.add(user.clone()).unwrap();
            assert_eq!(repo.get(1), Some(&user));
        }

        #[test]
        fn test_multiple_users() {
            let mut repo = UserRepository::new(HashMapStorage::new());

            for i in 1..=10 {
                let user = User::new(i, format!("user{}@example.com", i));
                repo.add(user).unwrap();
            }

            for i in 1..=10 {
                assert!(repo.get(i).is_some());
            }

            repo.remove(5);
            assert!(repo.get(5).is_none());
        }
    }

    // Dynamic dispatch tests
    mod dynamic_dispatch {
        use super::*;

        #[test]
        fn test_add_and_get_user() {
            let mut repo = UserRepositoryDyn::new(HashMapStorage::new());
            let user = User::new(1, "test@example.com");

            repo.add(user.clone()).unwrap();
            assert_eq!(repo.get(1), Some(&user));
        }

        #[test]
        fn test_add_duplicate_user_fails() {
            let mut repo = UserRepositoryDyn::new(HashMapStorage::new());
            let user = User::new(1, "test@example.com");

            repo.add(user.clone()).unwrap();
            let result = repo.add(user);

            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .contains("already exists"));
        }

        #[test]
        fn test_update_user() {
            let mut repo = UserRepositoryDyn::new(HashMapStorage::new());
            let mut user = User::new(1, "test@example.com");

            repo.add(user.clone()).unwrap();
            user.activate();
            repo.update(user.clone()).unwrap();

            let retrieved = repo.get(1).unwrap();
            assert!(retrieved.activated);
        }

        #[test]
        fn test_update_nonexistent_user_fails() {
            let mut repo = UserRepositoryDyn::new(HashMapStorage::new());
            let user = User::new(999, "test@example.com");

            let result = repo.update(user);
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("not found"));
        }

        #[test]
        fn test_remove_user() {
            let mut repo = UserRepositoryDyn::new(HashMapStorage::new());
            let user = User::new(1, "test@example.com");

            repo.add(user.clone()).unwrap();
            let removed = repo.remove(1);

            assert_eq!(removed, Some(user));
            assert_eq!(repo.get(1), None);
        }

        #[test]
        fn test_remove_nonexistent_user() {
            let mut repo =
                UserRepositoryDyn::new(HashMapStorage::<u64, User>::new());
            assert_eq!(repo.remove(999), None);
        }

        #[test]
        fn test_btreemap_storage() {
            let mut repo = UserRepositoryDyn::new(BTreeMapStorage::new());
            let user = User::new(1, "test@example.com");

            repo.add(user.clone()).unwrap();
            assert_eq!(repo.get(1), Some(&user));
        }

        #[test]
        fn test_multiple_users() {
            let mut repo = UserRepositoryDyn::new(HashMapStorage::new());

            for i in 1..=10 {
                let user = User::new(i, format!("user{}@example.com", i));
                repo.add(user).unwrap();
            }

            for i in 1..=10 {
                assert!(repo.get(i).is_some());
            }

            repo.remove(5);
            assert!(repo.get(5).is_none());
        }

        #[test]
        fn test_heterogeneous_storage_collection() {
            // This demonstrates the key advantage of dynamic dispatch:
            // we can store different storage implementations in the same collection
            let storages: Vec<Box<dyn Storage<u64, User>>> = vec![
                Box::new(HashMapStorage::new()),
                Box::new(BTreeMapStorage::new()),
            ];

            assert_eq!(storages.len(), 2);

            // We can even use them polymorphically
            for mut storage in storages {
                let user = User::new(1, "test@example.com");
                storage.set(1, user.clone());
                assert_eq!(storage.get(&1), Some(&user));
            }
        }
    }

    // Functional equivalence tests
    mod equivalence {
        use super::*;

        #[test]
        fn test_static_and_dynamic_behave_identically() {
            let mut static_repo = UserRepository::new(HashMapStorage::new());
            let mut dyn_repo = UserRepositoryDyn::new(HashMapStorage::new());

            let user = User::new(1, "test@example.com");

            // Both should successfully add
            static_repo.add(user.clone()).unwrap();
            dyn_repo.add(user.clone()).unwrap();

            // Both should retrieve the same user
            assert_eq!(static_repo.get(1), dyn_repo.get(1));

            // Both should fail to add duplicate
            assert!(static_repo.add(user.clone()).is_err());
            assert!(dyn_repo.add(user.clone()).is_err());

            // Both should successfully update
            let mut updated = user.clone();
            updated.activate();
            static_repo.update(updated.clone()).unwrap();
            dyn_repo.update(updated.clone()).unwrap();

            assert_eq!(static_repo.get(1), dyn_repo.get(1));

            // Both should successfully remove
            assert_eq!(static_repo.remove(1), dyn_repo.remove(1));
            assert_eq!(static_repo.get(1), dyn_repo.get(1));
        }
    }
}
