//! Benchmarks for equation solving (Phase L)

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use expr_core::Store;
use solver::solve_univariate;

pub fn bench_solve_linear(c: &mut Criterion) {
    c.bench_function("solve_linear_2x_plus_3", |b| {
        b.iter(|| {
            let mut st = Store::new();
            let x = st.sym("x");
            let two = st.int(2);
            let three = st.int(3);
            // 2x + 3 = 0
            let two_x = st.mul(vec![two, x]);
            let expr = st.add(vec![two_x, three]);
            let _roots = solve_univariate(&mut st, expr, "x");
        });
    });
}

pub fn bench_solve_quadratic(c: &mut Criterion) {
    c.bench_function("solve_quadratic_x2_minus_5x_plus_6", |b| {
        b.iter(|| {
            let mut st = Store::new();
            let x = st.sym("x");
            let two = st.int(2);
            let neg_five = st.int(-5);
            let six = st.int(6);
            // x^2 - 5x + 6 = 0
            let x2 = st.pow(x, two);
            let neg_5x = st.mul(vec![neg_five, x]);
            let expr = st.add(vec![x2, neg_5x, six]);
            let _roots = solve_univariate(&mut st, expr, "x");
        });
    });
}

pub fn bench_solve_quadratic_rational(c: &mut Criterion) {
    c.bench_function("solve_quadratic_rational_roots", |b| {
        b.iter(|| {
            let mut st = Store::new();
            let x = st.sym("x");
            let two = st.int(2);
            let three = st.int(3);
            // 2x^2 + 3x - 6 = 0
            let x2 = st.pow(x, two);
            let two_x2 = st.mul(vec![two, x2]);
            let three_x = st.mul(vec![three, x]);
            let neg_six = st.int(-6);
            let expr = st.add(vec![two_x2, three_x, neg_six]);
            let _roots = solve_univariate(&mut st, expr, "x");
        });
    });
}

pub fn bench_solve_cubic(c: &mut Criterion) {
    c.bench_function("solve_cubic_x3_minus_6x2_plus_11x_minus_6", |b| {
        b.iter(|| {
            let mut st = Store::new();
            let x = st.sym("x");
            // x^3 - 6x^2 + 11x - 6 = 0 (roots: 1, 2, 3)
            let three = st.int(3);
            let x3 = st.pow(x, three);
            let two = st.int(2);
            let x2 = st.pow(x, two);
            let neg_six = st.int(-6);
            let neg_six_x2 = st.mul(vec![neg_six, x2]);
            let eleven = st.int(11);
            let eleven_x = st.mul(vec![eleven, x]);
            let neg_six_const = st.int(-6);
            let expr = st.add(vec![x3, neg_six_x2, eleven_x, neg_six_const]);
            let _roots = solve_univariate(&mut st, expr, "x");
        });
    });
}

pub fn bench_solve_perfect_square(c: &mut Criterion) {
    c.bench_function("solve_perfect_square_x2_minus_4", |b| {
        b.iter(|| {
            let mut st = Store::new();
            let x = st.sym("x");
            let two = st.int(2);
            let x2 = st.pow(x, two);
            let neg_four = st.int(-4);
            // x^2 - 4 = 0
            let expr = st.add(vec![x2, neg_four]);
            let roots = solve_univariate(&mut st, expr, "x");
            black_box(roots);
        });
    });
}

criterion_group!(
    benches,
    bench_solve_linear,
    bench_solve_quadratic,
    bench_solve_quadratic_rational,
    bench_solve_cubic,
    bench_solve_perfect_square
);
criterion_main!(benches);
