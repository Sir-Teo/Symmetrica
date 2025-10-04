//! Benchmarks for calculus operations (Phase L)

use calculus::{diff, integrate};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use expr_core::Store;

pub fn bench_diff_polynomial(c: &mut Criterion) {
    c.bench_function("diff_x5_polynomial", |b| {
        let mut st = Store::new();
        let x = st.sym("x");
        // x^5 + x^4 + x^3 + x^2 + x + 1
        let mut terms = Vec::new();
        for i in 0..=5 {
            let power = st.int(i);
            terms.push(st.pow(x, power));
        }
        let poly = st.add(terms);

        b.iter(|| {
            let _deriv = diff(&mut st, poly, "x");
        });
    });
}

pub fn bench_diff_product_rule(c: &mut Criterion) {
    c.bench_function("diff_product_x_times_x2", |b| {
        b.iter(|| {
            let mut st = Store::new();
            let x = st.sym("x");
            let two = st.int(2);
            let x2 = st.pow(x, two);
            let product = st.mul(vec![x, x2]);
            let _deriv = diff(&mut st, product, "x");
        });
    });
}

pub fn bench_diff_chain_rule(c: &mut Criterion) {
    c.bench_function("diff_sin_x2", |b| {
        b.iter(|| {
            let mut st = Store::new();
            let x = st.sym("x");
            let two = st.int(2);
            let x2 = st.pow(x, two);
            let sin_x2 = st.func("sin", vec![x2]);
            let _deriv = diff(&mut st, sin_x2, "x");
        });
    });
}

pub fn bench_diff_trig_functions(c: &mut Criterion) {
    c.bench_function("diff_trig_combo", |b| {
        b.iter(|| {
            let mut st = Store::new();
            let x = st.sym("x");
            let sin_x = st.func("sin", vec![x]);
            let cos_x = st.func("cos", vec![x]);
            // sin(x) + cos(x)
            let expr = st.add(vec![sin_x, cos_x]);
            let _deriv = diff(&mut st, expr, "x");
        });
    });
}

pub fn bench_diff_nested(c: &mut Criterion) {
    c.bench_function("diff_nested_10_times", |b| {
        let mut st = Store::new();
        let x = st.sym("x");
        let four = st.int(4);
        let x4 = st.pow(x, four);

        b.iter(|| {
            let mut current = x4;
            for _ in 0..10 {
                current = diff(&mut st, current, "x");
            }
            black_box(current);
        });
    });
}

pub fn bench_integrate_polynomial(c: &mut Criterion) {
    c.bench_function("integrate_x3", |b| {
        b.iter(|| {
            let mut st = Store::new();
            let x = st.sym("x");
            let three = st.int(3);
            let x3 = st.pow(x, three);
            let _integral = integrate(&mut st, x3, "x");
        });
    });
}

pub fn bench_integrate_sum(c: &mut Criterion) {
    c.bench_function("integrate_polynomial_sum", |b| {
        b.iter(|| {
            let mut st = Store::new();
            let x = st.sym("x");
            let mut terms = Vec::new();
            for i in 1..=5 {
                let power = st.int(i);
                terms.push(st.pow(x, power));
            }
            let poly = st.add(terms);
            let _integral = integrate(&mut st, poly, "x");
        });
    });
}

criterion_group!(
    benches,
    bench_diff_polynomial,
    bench_diff_product_rule,
    bench_diff_chain_rule,
    bench_diff_trig_functions,
    bench_diff_nested,
    bench_integrate_polynomial,
    bench_integrate_sum
);
criterion_main!(benches);
