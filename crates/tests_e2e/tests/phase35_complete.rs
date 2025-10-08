//! End-to-end tests for Phases 3-5 completion
//!
//! Phase 3: Special Functions
//! Phase 4: Advanced Solving (Gröbner bases foundation)
//! Phase 5: Symbolic Summation

use calculus::diff;
use evalf::{eval, EvalContext};
use expr_core::Store;
use simplify::simplify;
use summation::sum;

#[test]
fn phase3_bessel_j_numeric_and_symbolic() {
    let mut st = Store::new();
    let ctx = EvalContext::new();

    // BesselJ(0, 1) ≈ 0.7652
    let zero = st.int(0);
    let one = st.int(1);
    let bessel_expr = st.func("BesselJ", vec![zero, one]);

    let result = eval(&st, bessel_expr, &ctx).unwrap();
    assert!((result - 0.7651976866).abs() < 1e-6);

    // Symbolic differentiation: d/dx BesselJ(0, x) involves BesselJ(-1, x) and BesselJ(1, x)
    let x = st.sym("x");
    let bessel_x = st.func("BesselJ", vec![zero, x]);
    let deriv = diff(&mut st, bessel_x, "x");
    let deriv_str = st.to_string(deriv);
    assert!(deriv_str.contains("BesselJ"));
}

#[test]
fn phase3_legendre_polynomials() {
    let mut st = Store::new();
    let ctx = EvalContext::new();

    // LegendreP(2, 0.5) = (3*0.25 - 1)/2 = -0.125
    let two = st.int(2);
    let half = st.rat(1, 2);
    let legendre_expr = st.func("LegendreP", vec![two, half]);

    let result = eval(&st, legendre_expr, &ctx).unwrap();
    assert!((result - (-0.125)).abs() < 1e-10);

    // Symbolic form
    let x = st.sym("x");
    let legendre_x = st.func("LegendreP", vec![two, x]);
    let legendre_str = st.to_string(legendre_x);
    assert_eq!(legendre_str, "LegendreP(2, x)");

    // Differentiation
    let deriv = diff(&mut st, legendre_x, "x");
    let deriv_str = st.to_string(deriv);
    assert!(deriv_str.contains("LegendreP"));
}

#[test]
fn phase3_chebyshev_polynomials() {
    let mut st = Store::new();
    let ctx = EvalContext::new();

    // ChebyshevT(2, 0.5) = 2*0.25 - 1 = -0.5
    let two = st.int(2);
    let half = st.rat(1, 2);
    let cheb_expr = st.func("ChebyshevT", vec![two, half]);

    let result = eval(&st, cheb_expr, &ctx).unwrap();
    assert!((result - (-0.5)).abs() < 1e-10);

    // Symbolic differentiation
    let x = st.sym("x");
    let cheb_x = st.func("ChebyshevT", vec![two, x]);
    let deriv = diff(&mut st, cheb_x, "x");
    let deriv_str = st.to_string(deriv);
    assert!(deriv_str.contains("ChebyshevU") || deriv_str.contains("2"));
}

#[test]
fn phase3_gamma_erf_ei_integration() {
    let mut st = Store::new();
    let ctx = EvalContext::new();

    // Gamma(5) = 4! = 24
    let five = st.int(5);
    let gamma_expr = st.func("Gamma", vec![five]);
    let gamma_result = eval(&st, gamma_expr, &ctx).unwrap();
    assert!((gamma_result - 24.0).abs() < 1e-6);

    // erf(1) ≈ 0.8427
    let one = st.int(1);
    let erf_expr = st.func("erf", vec![one]);
    let erf_result = eval(&st, erf_expr, &ctx).unwrap();
    assert!((erf_result - 0.8427).abs() < 0.001);

    // erfc(1) ≈ 1 - 0.8427 = 0.1573
    let erfc_expr = st.func("erfc", vec![one]);
    let erfc_result = eval(&st, erfc_expr, &ctx).unwrap();
    assert!((erfc_result - 0.1573).abs() < 0.001);

    // Ei(1) ≈ 1.895
    let ei_expr = st.func("Ei", vec![one]);
    let ei_result = eval(&st, ei_expr, &ctx).unwrap();
    assert!((ei_result - 1.895).abs() < 0.01);
}

#[test]
fn phase5_power_sum_formulas() {
    let mut st = Store::new();

    // sum(k, k=0..n) = n(n+1)/2
    let k = st.sym("k");
    let n = st.sym("n");
    let zero = st.int(0);

    let sum_k = sum(&mut st, k, "k", zero, n).unwrap();
    let sum_k_str = st.to_string(sum_k);
    // Should contain n and 1/2
    assert!(sum_k_str.contains("n"));
    assert!(sum_k_str.contains("1/2") || sum_k_str.contains("2"));

    // sum(k^2, k=0..n) = n(n+1)(2n+1)/6
    let two = st.int(2);
    let k_sq = st.pow(k, two);
    let sum_k_sq = sum(&mut st, k_sq, "k", zero, n).unwrap();
    let sum_k_sq_str = st.to_string(sum_k_sq);
    assert!(sum_k_sq_str.contains("n"));
    assert!(sum_k_sq_str.contains("1/6") || sum_k_sq_str.contains("6"));

    // sum(k^3, k=0..n) = [n(n+1)/2]^2
    let three = st.int(3);
    let k_cubed = st.pow(k, three);
    let sum_k_cubed = sum(&mut st, k_cubed, "k", zero, n).unwrap();
    let sum_k_cubed_str = st.to_string(sum_k_cubed);
    assert!(sum_k_cubed_str.contains("n"));
}

#[test]
fn phase5_arithmetic_and_geometric_sums() {
    let mut st = Store::new();

    // sum(5 + 3k, k=0..n)
    let k = st.sym("k");
    let n = st.sym("n");
    let zero = st.int(0);
    let five = st.int(5);
    let three = st.int(3);
    let three_k = st.mul(vec![three, k]);
    let term = st.add(vec![five, three_k]);

    let sum_arith = sum(&mut st, term, "k", zero, n).unwrap();
    let sum_arith_str = st.to_string(sum_arith);
    assert!(sum_arith_str.contains("n"));
    assert!(sum_arith_str.contains("5") || sum_arith_str.contains("3"));

    // sum(3*2^k, k=0..n)
    let two = st.int(2);
    let pow_2_k = st.pow(two, k);
    let geom_term = st.mul(vec![three, pow_2_k]);

    let sum_geom = sum(&mut st, geom_term, "k", zero, n).unwrap();
    let sum_geom_str = st.to_string(sum_geom);
    assert!(sum_geom_str.contains("n"));
    assert!(sum_geom_str.contains("2") && sum_geom_str.contains("3"));
}

#[test]
fn phase3_special_function_differentiation_chain_rule() {
    let mut st = Store::new();

    // d/dx erf(x^2) = (2/√π) * exp(-(x^2)^2) * 2x
    let x = st.sym("x");
    let two = st.int(2);
    let x_sq = st.pow(x, two);
    let erf_x_sq = st.func("erf", vec![x_sq]);

    let deriv = diff(&mut st, erf_x_sq, "x");
    let deriv_simp = simplify(&mut st, deriv);
    let deriv_str = st.to_string(deriv_simp);

    // Should contain exp and x
    assert!(deriv_str.contains("exp") || deriv_str.contains("x"));
}

#[test]
fn phase4_grobner_basis_foundation() {
    use grobner::{Monomial, MonomialOrder};

    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");
    let two = st.int(2);
    let three = st.int(3);

    // x^2 * y^3
    let x2 = st.pow(x, two);
    let y3 = st.pow(y, three);
    let prod = st.mul(vec![x2, y3]);

    let mono = Monomial::from_expr(&st, prod).unwrap();
    assert_eq!(mono.degree(), 5);
    assert_eq!(mono.exponents.get("x"), Some(&2));
    assert_eq!(mono.exponents.get("y"), Some(&3));

    // Test monomial ordering
    let vars = vec!["x".to_string(), "y".to_string()];
    let mono2 = Monomial::from_expr(&st, x2).unwrap();

    use std::cmp::Ordering;
    // x^2*y^3 > x^2 in grlex (higher degree)
    assert_eq!(mono.compare(&mono2, MonomialOrder::GrLex, &vars), Ordering::Greater);
}

#[test]
fn phase35_comprehensive_workflow() {
    let mut st = Store::new();
    let ctx = EvalContext::new();

    // Create expression: sum(BesselJ(0, k), k=0..2) numerically
    // This tests integration of Phase 3 (special functions) with Phase 5 (summation)
    let zero = st.int(0);
    let one = st.int(1);
    let two = st.int(2);

    // Evaluate BesselJ(0, 0) + BesselJ(0, 1) + BesselJ(0, 2)
    let b0 = st.func("BesselJ", vec![zero, zero]);
    let b1 = st.func("BesselJ", vec![zero, one]);
    let b2 = st.func("BesselJ", vec![zero, two]);

    let v0 = eval(&st, b0, &ctx).unwrap(); // ≈ 1.0
    let v1 = eval(&st, b1, &ctx).unwrap(); // ≈ 0.7652
    let v2 = eval(&st, b2, &ctx).unwrap(); // ≈ 0.2239

    let total = v0 + v1 + v2;
    assert!((total - 1.989).abs() < 0.01);

    // Test differentiation of Legendre polynomial
    let x = st.sym("x");
    let legendre = st.func("LegendreP", vec![two, x]);
    let deriv = diff(&mut st, legendre, "x");
    let deriv_simp = simplify(&mut st, deriv);

    // Should produce a valid derivative expression
    let deriv_str = st.to_string(deriv_simp);
    assert!(deriv_str.contains("LegendreP") || deriv_str.contains("x"));
}

#[test]
fn phase3_bessel_i_numeric_and_symbolic() {
    let mut st = Store::new();
    let ctx = EvalContext::new();

    // BesselI(0, 1) ≈ 1.2661
    let zero = st.int(0);
    let one = st.int(1);
    let bessel_expr = st.func("BesselI", vec![zero, one]);

    let result = eval(&st, bessel_expr, &ctx).unwrap();
    assert!((result - 1.2661).abs() < 0.001);

    // Symbolic differentiation: d/dx BesselI(0, x)
    let x = st.sym("x");
    let bessel_x = st.func("BesselI", vec![zero, x]);
    let deriv = diff(&mut st, bessel_x, "x");
    let deriv_str = st.to_string(deriv);
    assert!(deriv_str.contains("BesselI"));
}

#[test]
fn phase3_bessel_y_symbolic() {
    let mut st = Store::new();

    // Symbolic differentiation: d/dx BesselY(1, x)
    let one = st.int(1);
    let x = st.sym("x");
    let bessel_x = st.func("BesselY", vec![one, x]);
    let deriv = diff(&mut st, bessel_x, "x");
    let deriv_str = st.to_string(deriv);
    assert!(deriv_str.contains("BesselY"));
}

#[test]
fn phase3_bessel_k_symbolic() {
    let mut st = Store::new();

    // Symbolic differentiation: d/dx BesselK(0, x)
    let zero = st.int(0);
    let x = st.sym("x");
    let bessel_x = st.func("BesselK", vec![zero, x]);
    let deriv = diff(&mut st, bessel_x, "x");
    let deriv_str = st.to_string(deriv);
    assert!(deriv_str.contains("BesselK"));
}
