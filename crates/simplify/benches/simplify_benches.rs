//! Benchmarks for simplification operations (Phase L)

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use expr_core::Store;
use simplify::simplify;

pub fn bench_simplify_idempotence(c: &mut Criterion) {
    c.bench_function("simplify_idempotent", |b| {
        let mut st = Store::new();
        let x = st.sym("x");
        let expr = st.add(vec![x, x, x, x]);
        b.iter(|| {
            let s1 = simplify(&mut st, expr);
            let s2 = simplify(&mut st, s1);
            black_box(s2);
        });
    });
}

pub fn bench_collect_like_terms(c: &mut Criterion) {
    c.bench_function("collect_like_terms_10", |b| {
        b.iter(|| {
            let mut st = Store::new();
            let x = st.sym("x");
            let mut terms = Vec::new();
            for i in 1..=10 {
                let coef = st.int(i);
                terms.push(st.mul(vec![coef, x]));
            }
            let expr = st.add(terms);
            let _simplified = simplify(&mut st, expr);
        });
    });
}

pub fn bench_distributive_law(c: &mut Criterion) {
    c.bench_function("distributive_simplify", |b| {
        b.iter(|| {
            let mut st = Store::new();
            let x = st.sym("x");
            let y = st.sym("y");
            let z = st.sym("z");
            // (x + y) * (x + z) should expand
            let sum1 = st.add(vec![x, y]);
            let sum2 = st.add(vec![x, z]);
            let expr = st.mul(vec![sum1, sum2]);
            let _simplified = simplify(&mut st, expr);
        });
    });
}

pub fn bench_rational_arithmetic(c: &mut Criterion) {
    c.bench_function("rational_add_simplify", |b| {
        b.iter(|| {
            let mut st = Store::new();
            let mut terms = Vec::new();
            for i in 1..=20 {
                terms.push(st.rat(1, i));
            }
            let expr = st.add(terms);
            let _simplified = simplify(&mut st, expr);
        });
    });
}

pub fn bench_polynomial_simplify(c: &mut Criterion) {
    c.bench_function("polynomial_x5_simplify", |b| {
        b.iter(|| {
            let mut st = Store::new();
            let x = st.sym("x");
            let mut terms = Vec::new();
            // 5x^4 + 4x^3 + 3x^2 + 2x + 1
            for i in (1..=5).rev() {
                let coef = st.int(i);
                let power = st.int(i - 1);
                let x_pow = st.pow(x, power);
                terms.push(st.mul(vec![coef, x_pow]));
            }
            let expr = st.add(terms);
            let _simplified = simplify(&mut st, expr);
        });
    });
}

pub fn bench_cancel_terms(c: &mut Criterion) {
    c.bench_function("cancel_x_minus_x", |b| {
        b.iter(|| {
            let mut st = Store::new();
            let x = st.sym("x");
            let neg_one = st.int(-1);
            let neg_x = st.mul(vec![neg_one, x]);
            let expr = st.add(vec![x, neg_x]);
            let _simplified = simplify(&mut st, expr);
        });
    });
}

pub fn bench_nested_simplify(c: &mut Criterion) {
    c.bench_function("nested_expr_simplify", |b| {
        b.iter(|| {
            let mut st = Store::new();
            let x = st.sym("x");
            let y = st.sym("y");
            // ((x + 0) * 1) + ((y * 1) + 0)
            let zero = st.int(0);
            let one = st.int(1);
            let x_plus_zero = st.add(vec![x, zero]);
            let left = st.mul(vec![x_plus_zero, one]);
            let y_times_one = st.mul(vec![y, one]);
            let right = st.add(vec![y_times_one, zero]);
            let expr = st.add(vec![left, right]);
            let _simplified = simplify(&mut st, expr);
        });
    });
}

criterion_group!(
    benches,
    bench_simplify_idempotence,
    bench_collect_like_terms,
    bench_distributive_law,
    bench_rational_arithmetic,
    bench_polynomial_simplify,
    bench_cancel_terms,
    bench_nested_simplify
);
criterion_main!(benches);
