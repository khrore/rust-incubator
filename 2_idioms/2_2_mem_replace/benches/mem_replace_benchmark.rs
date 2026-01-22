use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use step_2_2::{original, optimized};

// ===== Benchmark rotate() with different types =====

fn bench_rotate_i32(c: &mut Criterion) {
    let mut group = c.benchmark_group("rotate_i32");

    group.bench_function("original_clone", |b| {
        b.iter(|| {
            let mut trinity = original::Trinity { a: 1, b: 2, c: 3 };
            for _ in 0..100 {
                trinity.rotate();
                black_box(&trinity);
            }
        });
    });

    group.bench_function("optimized_swap", |b| {
        b.iter(|| {
            let mut trinity = optimized::Trinity { a: 1, b: 2, c: 3 };
            for _ in 0..100 {
                trinity.rotate();
                black_box(&trinity);
            }
        });
    });

    group.finish();
}

fn bench_rotate_string(c: &mut Criterion) {
    let mut group = c.benchmark_group("rotate_string");

    group.bench_function("original_clone", |b| {
        b.iter(|| {
            let mut trinity = original::Trinity {
                a: "Hello World".to_string(),
                b: "Rust Programming".to_string(),
                c: "Performance Test".to_string(),
            };
            for _ in 0..100 {
                trinity.rotate();
                black_box(&trinity);
            }
        });
    });

    group.bench_function("optimized_swap", |b| {
        b.iter(|| {
            let mut trinity = optimized::Trinity {
                a: "Hello World".to_string(),
                b: "Rust Programming".to_string(),
                c: "Performance Test".to_string(),
            };
            for _ in 0..100 {
                trinity.rotate();
                black_box(&trinity);
            }
        });
    });

    group.finish();
}

fn bench_rotate_vec(c: &mut Criterion) {
    let mut group = c.benchmark_group("rotate_vec");

    group.bench_function("original_clone", |b| {
        b.iter(|| {
            let mut trinity = original::Trinity {
                a: vec![1, 2, 3, 4, 5],
                b: vec![6, 7, 8, 9, 10],
                c: vec![11, 12, 13, 14, 15],
            };
            for _ in 0..100 {
                trinity.rotate();
                black_box(&trinity);
            }
        });
    });

    group.bench_function("optimized_swap", |b| {
        b.iter(|| {
            let mut trinity = optimized::Trinity {
                a: vec![1, 2, 3, 4, 5],
                b: vec![6, 7, 8, 9, 10],
                c: vec![11, 12, 13, 14, 15],
            };
            for _ in 0..100 {
                trinity.rotate();
                black_box(&trinity);
            }
        });
    });

    group.finish();
}

// ===== Benchmark resolve() with different scenarios =====

fn bench_resolve_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("resolve_small");

    let expected_orig = original::Trinity { a: 1, b: 2, c: 3 };
    let unsolved_orig = vec![
        original::Trinity { a: 1, b: 2, c: 3 },
        original::Trinity { a: 2, b: 1, c: 3 },
        original::Trinity { a: 2, b: 3, c: 1 },
        original::Trinity { a: 3, b: 1, c: 2 },
    ];

    let expected_opt = optimized::Trinity { a: 1, b: 2, c: 3 };
    let unsolved_opt = vec![
        optimized::Trinity { a: 1, b: 2, c: 3 },
        optimized::Trinity { a: 2, b: 1, c: 3 },
        optimized::Trinity { a: 2, b: 3, c: 1 },
        optimized::Trinity { a: 3, b: 1, c: 2 },
    ];

    group.bench_function("original_clone", |b| {
        b.iter(|| {
            let mut solver = original::Solver::new(
                expected_orig.clone(),
                black_box(unsolved_orig.clone()),
            );
            solver.resolve();
            black_box(&solver);
        });
    });

    group.bench_function("optimized_take", |b| {
        b.iter(|| {
            let mut solver = optimized::Solver::new(
                expected_opt.clone(),
                black_box(unsolved_opt.clone()),
            );
            solver.resolve();
            black_box(&solver);
        });
    });

    group.finish();
}

fn bench_resolve_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("resolve_medium");

    let expected_orig = original::Trinity {
        a: "target".to_string(),
        b: "value".to_string(),
        c: "here".to_string(),
    };

    let unsolved_orig: Vec<_> = (0..100)
        .map(|i| original::Trinity {
            a: format!("string_{}", i),
            b: format!("value_{}", i),
            c: format!("data_{}", i),
        })
        .collect();

    let expected_opt = optimized::Trinity {
        a: "target".to_string(),
        b: "value".to_string(),
        c: "here".to_string(),
    };

    let unsolved_opt: Vec<_> = (0..100)
        .map(|i| optimized::Trinity {
            a: format!("string_{}", i),
            b: format!("value_{}", i),
            c: format!("data_{}", i),
        })
        .collect();

    group.bench_function("original_clone", |b| {
        b.iter(|| {
            let mut solver = original::Solver::new(
                expected_orig.clone(),
                black_box(unsolved_orig.clone()),
            );
            solver.resolve();
            black_box(&solver);
        });
    });

    group.bench_function("optimized_take", |b| {
        b.iter(|| {
            let mut solver = optimized::Solver::new(
                expected_opt.clone(),
                black_box(unsolved_opt.clone()),
            );
            solver.resolve();
            black_box(&solver);
        });
    });

    group.finish();
}

fn bench_resolve_large_vecs(c: &mut Criterion) {
    let mut group = c.benchmark_group("resolve_large_vecs");

    let expected_orig = original::Trinity {
        a: vec![0; 1000],
        b: vec![1; 1000],
        c: vec![2; 1000],
    };

    let unsolved_orig: Vec<_> = (0..50)
        .map(|i| original::Trinity {
            a: vec![i; 1000],
            b: vec![i + 1; 1000],
            c: vec![i + 2; 1000],
        })
        .collect();

    let expected_opt = optimized::Trinity {
        a: vec![0; 1000],
        b: vec![1; 1000],
        c: vec![2; 1000],
    };

    let unsolved_opt: Vec<_> = (0..50)
        .map(|i| optimized::Trinity {
            a: vec![i; 1000],
            b: vec![i + 1; 1000],
            c: vec![i + 2; 1000],
        })
        .collect();

    group.bench_function("original_clone", |b| {
        b.iter(|| {
            let mut solver = original::Solver::new(
                expected_orig.clone(),
                black_box(unsolved_orig.clone()),
            );
            solver.resolve();
            black_box(&solver);
        });
    });

    group.bench_function("optimized_take", |b| {
        b.iter(|| {
            let mut solver = optimized::Solver::new(
                expected_opt.clone(),
                black_box(unsolved_opt.clone()),
            );
            solver.resolve();
            black_box(&solver);
        });
    });

    group.finish();
}

// ===== Single rotation comparison with varying string sizes =====

fn bench_single_rotation_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_rotation_by_size");

    for size in [10, 100, 1000].iter() {
        let string_data = "x".repeat(*size);

        group.bench_with_input(BenchmarkId::new("original", size), size, |b, _| {
            b.iter(|| {
                let mut trinity = original::Trinity {
                    a: string_data.clone(),
                    b: string_data.clone(),
                    c: string_data.clone(),
                };
                trinity.rotate();
                black_box(&trinity);
            });
        });

        group.bench_with_input(BenchmarkId::new("optimized", size), size, |b, _| {
            b.iter(|| {
                let mut trinity = optimized::Trinity {
                    a: string_data.clone(),
                    b: string_data.clone(),
                    c: string_data.clone(),
                };
                trinity.rotate();
                black_box(&trinity);
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_rotate_i32,
    bench_rotate_string,
    bench_rotate_vec,
    bench_resolve_small,
    bench_resolve_medium,
    bench_resolve_large_vecs,
    bench_single_rotation_comparison,
);

criterion_main!(benches);
