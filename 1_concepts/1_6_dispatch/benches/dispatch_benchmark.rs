use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};

// ============================================================================
// Copy the core types from main.rs (necessary for benchmarks)
// ============================================================================

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
}

trait Storage<K, V> {
    fn set(&mut self, key: K, val: V);
    fn get(&self, key: &K) -> Option<&V>;
    fn remove(&mut self, key: &K) -> Option<V>;
}

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

struct UserRepository<S> {
    storage: S,
}

impl<S: Storage<u64, User>> UserRepository<S> {
    fn new(storage: S) -> Self {
        Self { storage }
    }

    fn add(&mut self, user: User) -> Result<(), String> {
        if self.storage.get(&user.id).is_some() {
            return Err(format!("User with id {} already exists", user.id));
        }
        self.storage.set(user.id, user);
        Ok(())
    }

    fn get(&self, id: u64) -> Option<&User> {
        self.storage.get(&id)
    }

    fn update(&mut self, user: User) -> Result<(), String> {
        if self.storage.get(&user.id).is_none() {
            return Err(format!("User with id {} not found", user.id));
        }
        self.storage.set(user.id, user);
        Ok(())
    }

    fn remove(&mut self, id: u64) -> Option<User> {
        self.storage.remove(&id)
    }
}

struct UserRepositoryDyn {
    storage: Box<dyn Storage<u64, User>>,
}

impl UserRepositoryDyn {
    fn new<S: Storage<u64, User> + 'static>(storage: S) -> Self {
        Self {
            storage: Box::new(storage),
        }
    }

    fn add(&mut self, user: User) -> Result<(), String> {
        if self.storage.get(&user.id).is_some() {
            return Err(format!("User with id {} already exists", user.id));
        }
        self.storage.set(user.id, user);
        Ok(())
    }

    fn get(&self, id: u64) -> Option<&User> {
        self.storage.get(&id)
    }

    fn update(&mut self, user: User) -> Result<(), String> {
        if self.storage.get(&user.id).is_none() {
            return Err(format!("User with id {} not found", user.id));
        }
        self.storage.set(user.id, user);
        Ok(())
    }

    fn remove(&mut self, id: u64) -> Option<User> {
        self.storage.remove(&id)
    }
}

// ============================================================================
// Benchmarks
// ============================================================================

fn bench_get_operation(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_operation");

    // Prepare repositories with 100 users
    let mut static_repo = UserRepository::new(HashMapStorage::new());
    let mut dyn_repo = UserRepositoryDyn::new(HashMapStorage::new());

    for i in 0..100 {
        let user = User::new(i, format!("user{}@example.com", i));
        static_repo.add(user.clone()).unwrap();
        dyn_repo.add(user).unwrap();
    }

    group.bench_function("static_dispatch", |b| {
        b.iter(|| {
            for i in 0..100 {
                black_box(static_repo.get(black_box(i)));
            }
        })
    });

    group.bench_function("dynamic_dispatch", |b| {
        b.iter(|| {
            for i in 0..100 {
                black_box(dyn_repo.get(black_box(i)));
            }
        })
    });

    group.finish();
}

fn bench_add_operation(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_operation");

    group.bench_function("static_dispatch", |b| {
        b.iter(|| {
            let mut repo = UserRepository::new(HashMapStorage::new());
            for i in 0..100 {
                let user = User::new(i, format!("user{}@example.com", i));
                black_box(repo.add(user).unwrap());
            }
        })
    });

    group.bench_function("dynamic_dispatch", |b| {
        b.iter(|| {
            let mut repo = UserRepositoryDyn::new(HashMapStorage::new());
            for i in 0..100 {
                let user = User::new(i, format!("user{}@example.com", i));
                black_box(repo.add(user).unwrap());
            }
        })
    });

    group.finish();
}

fn bench_update_operation(c: &mut Criterion) {
    let mut group = c.benchmark_group("update_operation");

    // Prepare repositories with 100 users
    let mut static_repo = UserRepository::new(HashMapStorage::new());
    let mut dyn_repo = UserRepositoryDyn::new(HashMapStorage::new());

    for i in 0..100 {
        let user = User::new(i, format!("user{}@example.com", i));
        static_repo.add(user.clone()).unwrap();
        dyn_repo.add(user).unwrap();
    }

    group.bench_function("static_dispatch", |b| {
        b.iter(|| {
            for i in 0..100 {
                let mut user = User::new(i, format!("updated{}@example.com", i));
                user.activated = true;
                black_box(static_repo.update(user).unwrap());
            }
        })
    });

    group.bench_function("dynamic_dispatch", |b| {
        b.iter(|| {
            for i in 0..100 {
                let mut user = User::new(i, format!("updated{}@example.com", i));
                user.activated = true;
                black_box(dyn_repo.update(user).unwrap());
            }
        })
    });

    group.finish();
}

fn bench_remove_operation(c: &mut Criterion) {
    let mut group = c.benchmark_group("remove_operation");

    group.bench_function("static_dispatch", |b| {
        b.iter(|| {
            let mut repo = UserRepository::new(HashMapStorage::new());
            for i in 0..100 {
                let user = User::new(i, format!("user{}@example.com", i));
                repo.add(user).unwrap();
            }
            for i in 0..100 {
                black_box(repo.remove(i));
            }
        })
    });

    group.bench_function("dynamic_dispatch", |b| {
        b.iter(|| {
            let mut repo = UserRepositoryDyn::new(HashMapStorage::new());
            for i in 0..100 {
                let user = User::new(i, format!("user{}@example.com", i));
                repo.add(user).unwrap();
            }
            for i in 0..100 {
                black_box(repo.remove(i));
            }
        })
    });

    group.finish();
}

fn bench_mixed_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_workload");

    group.bench_function("static_dispatch", |b| {
        b.iter(|| {
            let mut repo = UserRepository::new(HashMapStorage::new());

            // Add 50 users
            for i in 0..50 {
                let user = User::new(i, format!("user{}@example.com", i));
                repo.add(user).unwrap();
            }

            // Read 25 users
            for i in 0..25 {
                black_box(repo.get(i));
            }

            // Update 25 users
            for i in 0..25 {
                let mut user = User::new(i, format!("updated{}@example.com", i));
                user.activated = true;
                repo.update(user).unwrap();
            }

            // Remove 10 users
            for i in 0..10 {
                repo.remove(i);
            }
        })
    });

    group.bench_function("dynamic_dispatch", |b| {
        b.iter(|| {
            let mut repo = UserRepositoryDyn::new(HashMapStorage::new());

            // Add 50 users
            for i in 0..50 {
                let user = User::new(i, format!("user{}@example.com", i));
                repo.add(user).unwrap();
            }

            // Read 25 users
            for i in 0..25 {
                black_box(repo.get(i));
            }

            // Update 25 users
            for i in 0..25 {
                let mut user = User::new(i, format!("updated{}@example.com", i));
                user.activated = true;
                repo.update(user).unwrap();
            }

            // Remove 10 users
            for i in 0..10 {
                repo.remove(i);
            }
        })
    });

    group.finish();
}

fn bench_storage_backends(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_backends_comparison");

    // Static dispatch with HashMap
    group.bench_function("static_hashmap", |b| {
        b.iter(|| {
            let mut repo = UserRepository::new(HashMapStorage::new());
            for i in 0..100 {
                let user = User::new(i, format!("user{}@example.com", i));
                repo.add(user).unwrap();
            }
            for i in 0..100 {
                black_box(repo.get(i));
            }
        })
    });

    // Static dispatch with BTreeMap
    group.bench_function("static_btreemap", |b| {
        b.iter(|| {
            let mut repo = UserRepository::new(BTreeMapStorage::new());
            for i in 0..100 {
                let user = User::new(i, format!("user{}@example.com", i));
                repo.add(user).unwrap();
            }
            for i in 0..100 {
                black_box(repo.get(i));
            }
        })
    });

    // Dynamic dispatch with HashMap
    group.bench_function("dynamic_hashmap", |b| {
        b.iter(|| {
            let mut repo = UserRepositoryDyn::new(HashMapStorage::new());
            for i in 0..100 {
                let user = User::new(i, format!("user{}@example.com", i));
                repo.add(user).unwrap();
            }
            for i in 0..100 {
                black_box(repo.get(i));
            }
        })
    });

    // Dynamic dispatch with BTreeMap
    group.bench_function("dynamic_btreemap", |b| {
        b.iter(|| {
            let mut repo = UserRepositoryDyn::new(BTreeMapStorage::new());
            for i in 0..100 {
                let user = User::new(i, format!("user{}@example.com", i));
                repo.add(user).unwrap();
            }
            for i in 0..100 {
                black_box(repo.get(i));
            }
        })
    });

    group.finish();
}

fn bench_single_operation_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_operation_overhead");

    // Prepare repositories with data
    let mut static_repo = UserRepository::new(HashMapStorage::new());
    let mut dyn_repo = UserRepositoryDyn::new(HashMapStorage::new());

    for i in 0..1000 {
        let user = User::new(i, format!("user{}@example.com", i));
        static_repo.add(user.clone()).unwrap();
        dyn_repo.add(user).unwrap();
    }

    // Measure single get operation to see pure dispatch overhead
    group.bench_function("static_single_get", |b| {
        b.iter(|| black_box(static_repo.get(black_box(42))))
    });

    group.bench_function("dynamic_single_get", |b| {
        b.iter(|| black_box(dyn_repo.get(black_box(42))))
    });

    group.finish();
}

fn bench_scale_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("scale_operations");

    for size in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("static_dispatch", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut repo = UserRepository::new(HashMapStorage::new());
                    for i in 0..size {
                        let user = User::new(i, format!("user{}@example.com", i));
                        repo.add(user).unwrap();
                    }
                    for i in 0..size {
                        black_box(repo.get(i));
                    }
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("dynamic_dispatch", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut repo = UserRepositoryDyn::new(HashMapStorage::new());
                    for i in 0..size {
                        let user = User::new(i, format!("user{}@example.com", i));
                        repo.add(user).unwrap();
                    }
                    for i in 0..size {
                        black_box(repo.get(i));
                    }
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_get_operation,
    bench_add_operation,
    bench_update_operation,
    bench_remove_operation,
    bench_mixed_workload,
    bench_storage_backends,
    bench_single_operation_overhead,
    bench_scale_operations,
);

criterion_main!(benches);
