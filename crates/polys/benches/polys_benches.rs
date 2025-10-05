//! Benchmarks for polynomial operations (Phase L)
//!
//! Tests performance of:
//! - Univariate polynomial arithmetic (add, mul, div, gcd)
//! - Advanced operations (factor, resultant, discriminant)
//! - Expr ⟷ Poly conversions
//! - Multivariate polynomial operations

use arith::Q;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use expr_core::Store;
use polys::{expr_to_unipoly, unipoly_to_expr, MultiPoly, UniPoly};

// ========== Univariate Polynomial Arithmetic ==========

pub fn bench_unipoly_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("unipoly_add");
    for &degree in &[10usize, 50, 100] {
        group.throughput(Throughput::Elements(degree as u64));
        group.bench_with_input(BenchmarkId::from_parameter(degree), &degree, |b, &deg| {
            // Pre-build polynomials
            let coeffs1: Vec<Q> = (0..=deg).map(|i| Q(i as i64, 1)).collect();
            let coeffs2: Vec<Q> = (0..=deg).map(|i| Q((i + 1) as i64, 1)).collect();
            let p1 = UniPoly::new("x", coeffs1);
            let p2 = UniPoly::new("x", coeffs2);

            b.iter(|| {
                let _sum = black_box(&p1).add(black_box(&p2));
            });
        });
    }
    group.finish();
}

pub fn bench_unipoly_mul(c: &mut Criterion) {
    let mut group = c.benchmark_group("unipoly_mul");
    for &degree in &[5usize, 10, 20, 50] {
        group.throughput(Throughput::Elements(degree as u64));
        group.bench_with_input(BenchmarkId::from_parameter(degree), &degree, |b, &deg| {
            // Pre-build polynomials
            let coeffs1: Vec<Q> = (0..=deg).map(|i| Q(i as i64 + 1, 1)).collect();
            let coeffs2: Vec<Q> = (0..=deg).map(|i| Q((i + 2) as i64, 1)).collect();
            let p1 = UniPoly::new("x", coeffs1);
            let p2 = UniPoly::new("x", coeffs2);

            b.iter(|| {
                let _product = black_box(&p1).mul(black_box(&p2));
            });
        });
    }
    group.finish();
}

pub fn bench_unipoly_div_rem(c: &mut Criterion) {
    let mut group = c.benchmark_group("unipoly_div_rem");
    for &dividend_degree in &[10usize, 20, 50] {
        let divisor_degree = 5;
        group.throughput(Throughput::Elements(dividend_degree as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(dividend_degree),
            &dividend_degree,
            |b, &deg| {
                // dividend: x^deg + ... + x + 1
                let dividend_coeffs: Vec<Q> = (0..=deg).map(|_| Q(1, 1)).collect();
                let dividend = UniPoly::new("x", dividend_coeffs);

                // divisor: x^5 + 2x^4 + ... + 2
                let divisor_coeffs: Vec<Q> = (0..=divisor_degree).map(|_| Q(2, 1)).collect();
                let divisor = UniPoly::new("x", divisor_coeffs);

                b.iter(|| {
                    let _result = black_box(&dividend).div_rem(black_box(&divisor)).unwrap();
                });
            },
        );
    }
    group.finish();
}

pub fn bench_unipoly_gcd(c: &mut Criterion) {
    let mut group = c.benchmark_group("unipoly_gcd");
    for &degree in &[5usize, 10, 20] {
        group.throughput(Throughput::Elements(degree as u64));
        group.bench_with_input(BenchmarkId::from_parameter(degree), &degree, |b, &deg| {
            // p1 = (x - 1) * (x^deg + x^(deg-1) + ... + 1)
            // p2 = (x - 1) * (x^(deg-1) + ... + 1)
            // GCD should be (x - 1)
            let common_factor = UniPoly::new("x", vec![Q(-1, 1), Q(1, 1)]);

            let q1_coeffs = vec![Q(1, 1); deg + 1];
            let q1 = UniPoly::new("x", q1_coeffs);
            let p1 = common_factor.mul(&q1);

            let q2_coeffs = vec![Q(1, 1); deg.saturating_sub(1).max(1)];
            let q2 = UniPoly::new("x", q2_coeffs);
            let p2 = common_factor.mul(&q2);

            b.iter(|| {
                let _gcd = UniPoly::gcd(black_box(p1.clone()), black_box(p2.clone()));
            });
        });
    }
    group.finish();
}

pub fn bench_unipoly_deriv(c: &mut Criterion) {
    c.bench_function("unipoly_deriv_degree_20", |b| {
        // x^20 + x^19 + ... + x + 1
        let coeffs: Vec<Q> = (0..=20).map(|_| Q(1, 1)).collect();
        let p = UniPoly::new("x", coeffs);

        b.iter(|| {
            let _deriv = black_box(&p).deriv();
        });
    });
}

pub fn bench_unipoly_eval(c: &mut Criterion) {
    let mut group = c.benchmark_group("unipoly_eval");
    for &degree in &[10usize, 50, 100] {
        group.throughput(Throughput::Elements(degree as u64));
        group.bench_with_input(BenchmarkId::from_parameter(degree), &degree, |b, &deg| {
            let coeffs: Vec<Q> = (0..=deg).map(|i| Q(i as i64 + 1, 1)).collect();
            let p = UniPoly::new("x", coeffs);
            let x_val = Q(2, 1);

            b.iter(|| {
                let _result = black_box(&p).eval_q(black_box(x_val));
            });
        });
    }
    group.finish();
}

// ========== Advanced Univariate Operations ==========

pub fn bench_unipoly_factor(c: &mut Criterion) {
    let mut group = c.benchmark_group("unipoly_factor");

    // Test factoring polynomials with known rational roots
    // (x-1)(x-2)(x-3) = x^3 - 6x^2 + 11x - 6
    group.bench_function(BenchmarkId::new("cubic_three_roots", 3), |b| {
        let p = UniPoly::new("x", vec![Q(-6, 1), Q(11, 1), Q(-6, 1), Q(1, 1)]);
        b.iter(|| {
            let _factors = black_box(&p).factor();
        });
    });

    // (x-1)(x-2)(x-3)(x-4) = x^4 - 10x^3 + 35x^2 - 50x + 24
    group.bench_function(BenchmarkId::new("quartic_four_roots", 4), |b| {
        let p = UniPoly::new("x", vec![Q(24, 1), Q(-50, 1), Q(35, 1), Q(-10, 1), Q(1, 1)]);
        b.iter(|| {
            let _factors = black_box(&p).factor();
        });
    });

    group.finish();
}

pub fn bench_unipoly_resultant(c: &mut Criterion) {
    let mut group = c.benchmark_group("unipoly_resultant");

    for &degree in &[3usize, 5, 7] {
        group.bench_with_input(BenchmarkId::from_parameter(degree), &degree, |b, &deg| {
            // Create two polynomials of given degree
            let coeffs1: Vec<Q> = (0..=deg).map(|i| Q(i as i64 + 1, 1)).collect();
            let coeffs2: Vec<Q> = (0..=deg).map(|i| Q((deg - i) as i64 + 1, 1)).collect();
            let p1 = UniPoly::new("x", coeffs1);
            let p2 = UniPoly::new("x", coeffs2);

            b.iter(|| {
                let _res = UniPoly::resultant(black_box(&p1), black_box(&p2));
            });
        });
    }
    group.finish();
}

pub fn bench_unipoly_discriminant(c: &mut Criterion) {
    let mut group = c.benchmark_group("unipoly_discriminant");

    for &degree in &[3usize, 5, 7] {
        group.bench_with_input(BenchmarkId::from_parameter(degree), &degree, |b, &deg| {
            let coeffs: Vec<Q> = (0..=deg).map(|i| Q(i as i64 + 1, 1)).collect();
            let p = UniPoly::new("x", coeffs);

            b.iter(|| {
                let _disc = black_box(&p).discriminant();
            });
        });
    }
    group.finish();
}

pub fn bench_unipoly_square_free(c: &mut Criterion) {
    c.bench_function("square_free_decomposition", |b| {
        // x^4 - 2x^3 + x^2 = x^2(x-1)^2
        let p = UniPoly::new("x", vec![Q(0, 1), Q(0, 1), Q(1, 1), Q(-2, 1), Q(1, 1)]);

        b.iter(|| {
            let _factors = black_box(&p).square_free_decomposition();
        });
    });
}

// ========== Expr ⟷ Poly Conversions ==========

pub fn bench_expr_to_unipoly(c: &mut Criterion) {
    let mut group = c.benchmark_group("expr_to_unipoly");

    for &degree in &[5usize, 10, 20] {
        group.bench_with_input(BenchmarkId::from_parameter(degree), &degree, |b, &deg| {
            // Build polynomial expression: x^deg + ... + x + 1
            let mut st = Store::new();
            let x = st.sym("x");
            let mut terms = Vec::new();
            for i in 0..=deg {
                let exp = st.int(i.try_into().unwrap());
                let pow = st.pow(x, exp);
                terms.push(pow);
            }
            let expr = st.add(terms);

            b.iter(|| {
                let _poly = expr_to_unipoly(black_box(&st), black_box(expr), "x").unwrap();
            });
        });
    }
    group.finish();
}

pub fn bench_unipoly_to_expr(c: &mut Criterion) {
    let mut group = c.benchmark_group("unipoly_to_expr");

    for &degree in &[5usize, 10, 20] {
        group.bench_with_input(BenchmarkId::from_parameter(degree), &degree, |b, &deg| {
            let coeffs: Vec<Q> = (0..=deg).map(|i| Q(i as i64 + 1, 1)).collect();
            let poly = UniPoly::new("x", coeffs);

            b.iter(|| {
                let mut st = Store::new();
                let _expr = unipoly_to_expr(&mut st, black_box(&poly));
            });
        });
    }
    group.finish();
}

pub fn bench_expr_poly_roundtrip(c: &mut Criterion) {
    c.bench_function("expr_poly_roundtrip_degree_10", |b| {
        // Build polynomial: x^10 + 2x^9 + ... + 10x + 11
        let mut st = Store::new();
        let x = st.sym("x");
        let mut terms = Vec::new();
        for i in 0i64..=10 {
            let coeff = st.int(i + 1);
            let exp = st.int(i);
            let pow = st.pow(x, exp);
            let term = st.mul(vec![coeff, pow]);
            terms.push(term);
        }
        let expr = st.add(terms);

        b.iter(|| {
            let poly = expr_to_unipoly(&st, expr, "x").unwrap();
            let mut st2 = Store::new();
            let _back = unipoly_to_expr(&mut st2, &poly);
        });
    });
}

// ========== Multivariate Polynomials ==========

pub fn bench_multipoly_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("multipoly_add");

    for &num_terms in &[10usize, 50, 100] {
        group.throughput(Throughput::Elements(num_terms as u64));
        group.bench_with_input(BenchmarkId::from_parameter(num_terms), &num_terms, |b, &n| {
            // Create two multivariate polynomials with n terms
            // Build using public API by repeated multiplication
            let x = MultiPoly::var("x");
            let y = MultiPoly::var("y");

            let mut p1 = MultiPoly::zero();
            let mut p2 = MultiPoly::zero();

            for i in 0..n.min(25) {
                let x_pow = i % 5;
                let y_pow = (i / 5) % 5;

                // Build x^x_pow * y^y_pow
                let mut term = MultiPoly::constant(Q(i as i64 + 1, 1));
                for _ in 0..x_pow {
                    term = term.mul(&x);
                }
                for _ in 0..y_pow {
                    term = term.mul(&y);
                }
                p1 = p1.add(&term);

                let mut term2 = MultiPoly::constant(Q((n - i) as i64 + 1, 1));
                for _ in 0..x_pow {
                    term2 = term2.mul(&x);
                }
                for _ in 0..y_pow {
                    term2 = term2.mul(&y);
                }
                p2 = p2.add(&term2);
            }

            b.iter(|| {
                let _sum = black_box(&p1).add(black_box(&p2));
            });
        });
    }
    group.finish();
}

pub fn bench_multipoly_mul(c: &mut Criterion) {
    let mut group = c.benchmark_group("multipoly_mul");

    for &num_terms in &[5usize, 10, 20] {
        group.throughput(Throughput::Elements(num_terms as u64));
        group.bench_with_input(BenchmarkId::from_parameter(num_terms), &num_terms, |b, &n| {
            // (x + y + 1)^2 style polynomials
            let x = MultiPoly::var("x");
            let y = MultiPoly::var("y");
            let one = MultiPoly::constant(Q(1, 1));

            let mut p1 = x.add(&y).add(&one);
            let mut p2 = x.add(&y).add(&MultiPoly::constant(Q(2, 1)));

            // Build up slightly larger polynomials
            for i in 0..n / 3 {
                let coeff = MultiPoly::constant(Q(i as i64 + 1, 1));
                p1 = p1.add(&coeff);
                p2 = p2.add(&coeff);
            }

            b.iter(|| {
                let _product = black_box(&p1).mul(black_box(&p2));
            });
        });
    }
    group.finish();
}

pub fn bench_multipoly_eval(c: &mut Criterion) {
    c.bench_function("multipoly_eval_25_terms", |b| {
        // Create a multivariate polynomial with 25 terms
        // Build using public API
        let x = MultiPoly::var("x");
        let y = MultiPoly::var("y");
        let z = MultiPoly::var("z");

        let mut poly = MultiPoly::zero();
        for i in 0..25 {
            let x_pow = i % 5;
            let y_pow = (i / 5) % 5;
            let z_pow = (i / 10) % 3;

            // Build x^x_pow * y^y_pow * z^z_pow
            let mut term = MultiPoly::constant(Q(i as i64 + 1, 1));
            for _ in 0..x_pow {
                term = term.mul(&x);
            }
            for _ in 0..y_pow {
                term = term.mul(&y);
            }
            for _ in 0..z_pow {
                term = term.mul(&z);
            }
            poly = poly.add(&term);
        }

        let mut vals = std::collections::BTreeMap::new();
        vals.insert("x".to_string(), Q(2, 1));
        vals.insert("y".to_string(), Q(3, 1));
        vals.insert("z".to_string(), Q(5, 1));

        b.iter(|| {
            let _result = black_box(&poly).eval(black_box(&vals));
        });
    });
}

// ========== Criterion Configuration ==========

criterion_group!(
    arithmetic,
    bench_unipoly_add,
    bench_unipoly_mul,
    bench_unipoly_div_rem,
    bench_unipoly_gcd,
    bench_unipoly_deriv,
    bench_unipoly_eval
);

criterion_group!(
    advanced,
    bench_unipoly_factor,
    bench_unipoly_resultant,
    bench_unipoly_discriminant,
    bench_unipoly_square_free
);

criterion_group!(
    conversions,
    bench_expr_to_unipoly,
    bench_unipoly_to_expr,
    bench_expr_poly_roundtrip
);

criterion_group!(multivariate, bench_multipoly_add, bench_multipoly_mul, bench_multipoly_eval);

criterion_main!(arithmetic, advanced, conversions, multivariate);
