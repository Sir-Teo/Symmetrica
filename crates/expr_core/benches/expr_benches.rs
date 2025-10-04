//! Benchmarks for expr_core operations (Phase L)

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use expr_core::Store;
use simplify::simplify;

pub fn bench_build_atoms(c: &mut Criterion) {
    let mut group = c.benchmark_group("build_atoms");
    for &n in &[10_000usize, 50_000, 100_000] {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                let mut st = Store::new();
                for i in 0..n {
                    let _ = st.sym(format!("x{i}"));
                }
            });
        });
    }
    group.finish();
}

pub fn bench_hash_consing(c: &mut Criterion) {
    c.bench_function("hash_consing_dedup", |b| {
        b.iter(|| {
            let mut st = Store::new();
            // Same expressions should produce same IDs
            for _ in 0..1000 {
                let x = st.sym("x");
                let five = st.int(5);
                let _ = st.add(vec![x, five]);
            }
        });
    });
}

pub fn bench_add_chain(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_chain");
    for &n in &[1_000usize, 5_000, 10_000] {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                let mut st = Store::new();
                let mut terms = Vec::with_capacity(n);
                for i in 0..n as i64 {
                    terms.push(st.int(i));
                }
                let _sum = st.add(terms);
            });
        });
    }
    group.finish();
}

pub fn bench_mul_chain(c: &mut Criterion) {
    c.bench_function("mul_chain_100", |b| {
        b.iter(|| {
            let mut st = Store::new();
            let mut factors = Vec::new();
            for i in 1..=100 {
                factors.push(st.int(i));
            }
            let _product = st.mul(factors);
        });
    });
}

pub fn bench_pow_operations(c: &mut Criterion) {
    c.bench_function("pow_symbolic", |b| {
        b.iter(|| {
            let mut st = Store::new();
            let x = st.sym("x");
            for i in 1..=20 {
                let exp = st.int(i);
                let _ = black_box(st.pow(x, exp));
            }
        });
    });
}

pub fn bench_rational_ops(c: &mut Criterion) {
    c.bench_function("rational_normalization", |b| {
        b.iter(|| {
            let mut st = Store::new();
            for i in 1..=100 {
                for j in 1..=100 {
                    let _ = st.rat(i, j);
                }
            }
        });
    });
}

pub fn bench_simplify_collect(c: &mut Criterion) {
    c.bench_function("simplify_collect_small", |b| {
        b.iter(|| {
            let mut st = Store::new();
            let x = st.sym("x");
            let two = st.int(2);
            let two_x = st.mul(vec![two, x]);
            let three = st.int(3);
            let three_x = st.mul(vec![three, x]);
            let half = st.rat(1, 2);
            let half_x = st.mul(vec![half, x]);
            let expr = st.add(vec![two_x, three_x, half_x, half]);
            let _s = simplify(&mut st, expr);
        })
    });
}

pub fn bench_deep_expr_tree(c: &mut Criterion) {
    c.bench_function("deep_expr_tree_depth_10", |b| {
        b.iter(|| {
            let mut st = Store::new();
            let x = st.sym("x");
            let mut current = x;
            for i in 1..=10 {
                let n = st.int(i);
                let n_x = st.mul(vec![n, x]);
                current = st.add(vec![current, n_x]);
            }
            black_box(current);
        });
    });
}

criterion_group!(
    benches,
    bench_build_atoms,
    bench_hash_consing,
    bench_add_chain,
    bench_mul_chain,
    bench_pow_operations,
    bench_rational_ops,
    bench_simplify_collect,
    bench_deep_expr_tree
);
criterion_main!(benches);
