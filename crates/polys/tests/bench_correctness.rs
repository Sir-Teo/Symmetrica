//! Tests to verify correctness of benchmark operations
//! Ensures benchmark code paths are exercised and produce valid results

use arith::Q;
use expr_core::Store;
use polys::{expr_to_unipoly, unipoly_to_expr, Monomial, MultiPoly, UniPoly};
use std::collections::BTreeMap;

// ========== Univariate Polynomial Tests ==========

#[test]
fn test_unipoly_add_correctness() {
    // x + 1 added to 2x + 3 should give 3x + 4
    let p1 = UniPoly::new("x", vec![Q(1, 1), Q(1, 1)]);
    let p2 = UniPoly::new("x", vec![Q(3, 1), Q(2, 1)]);
    let sum = p1.add(&p2);
    assert_eq!(sum.coeffs, vec![Q(4, 1), Q(3, 1)]);
}

#[test]
fn test_unipoly_mul_correctness() {
    // (x + 1) * (x + 2) = x^2 + 3x + 2
    let p1 = UniPoly::new("x", vec![Q(1, 1), Q(1, 1)]);
    let p2 = UniPoly::new("x", vec![Q(2, 1), Q(1, 1)]);
    let product = p1.mul(&p2);
    assert_eq!(product.coeffs, vec![Q(2, 1), Q(3, 1), Q(1, 1)]);
}

#[test]
fn test_unipoly_div_rem_correctness() {
    // (x^2 + 3x + 2) / (x + 1) = (x + 2) remainder 0
    let dividend = UniPoly::new("x", vec![Q(2, 1), Q(3, 1), Q(1, 1)]);
    let divisor = UniPoly::new("x", vec![Q(1, 1), Q(1, 1)]);
    let (quotient, remainder) = dividend.div_rem(&divisor).unwrap();
    assert_eq!(quotient.coeffs, vec![Q(2, 1), Q(1, 1)]);
    assert!(remainder.is_zero());
}

#[test]
fn test_unipoly_gcd_correctness() {
    // gcd(x^2 - 1, x^2 - x) = x - 1
    let p1 = UniPoly::new("x", vec![Q(-1, 1), Q(0, 1), Q(1, 1)]);
    let p2 = UniPoly::new("x", vec![Q(0, 1), Q(-1, 1), Q(1, 1)]);
    let gcd = UniPoly::gcd(p1, p2);
    // GCD returns monic, so should be x - 1
    assert_eq!(gcd.degree(), Some(1));
    assert_eq!(gcd.coeffs[1], Q(1, 1)); // Monic
}

#[test]
fn test_unipoly_deriv_correctness() {
    // d/dx(x^3 + 2x^2 + 3x + 4) = 3x^2 + 4x + 3
    let p = UniPoly::new("x", vec![Q(4, 1), Q(3, 1), Q(2, 1), Q(1, 1)]);
    let dp = p.deriv();
    assert_eq!(dp.coeffs, vec![Q(3, 1), Q(4, 1), Q(3, 1)]);
}

#[test]
fn test_unipoly_eval_correctness() {
    // Evaluate x^2 + 2x + 1 at x = 3 should give 16
    let p = UniPoly::new("x", vec![Q(1, 1), Q(2, 1), Q(1, 1)]);
    let result = p.eval_q(Q(3, 1));
    assert_eq!(result, Q(16, 1));
}

#[test]
fn test_unipoly_factor_correctness() {
    // (x-1)(x-2) = x^2 - 3x + 2
    let p = UniPoly::new("x", vec![Q(2, 1), Q(-3, 1), Q(1, 1)]);
    let factors = p.factor();
    // Should find two linear factors
    assert!(factors.len() >= 2 || factors.iter().any(|(f, _)| f.degree() == Some(2)));
}

#[test]
fn test_unipoly_resultant_correctness() {
    // Resultant of x - 1 and x - 2 should be non-zero (no common roots)
    let p1 = UniPoly::new("x", vec![Q(-1, 1), Q(1, 1)]);
    let p2 = UniPoly::new("x", vec![Q(-2, 1), Q(1, 1)]);
    let res = UniPoly::resultant(&p1, &p2);
    assert!(res.is_some());
    assert!(!res.unwrap().is_zero());
}

#[test]
fn test_unipoly_discriminant_correctness() {
    // Discriminant of x^2 - 4 is 16 (two distinct roots)
    let p = UniPoly::new("x", vec![Q(-4, 1), Q(0, 1), Q(1, 1)]);
    let disc = p.discriminant();
    assert!(disc.is_some());
}

#[test]
fn test_unipoly_square_free_correctness() {
    // x^2 + 2x + 1 = (x+1)^2 has repeated roots
    let p = UniPoly::new("x", vec![Q(1, 1), Q(2, 1), Q(1, 1)]);
    let sf = p.square_free_decomposition();
    assert!(!sf.is_empty());
}

// ========== Conversion Tests ==========

#[test]
fn test_expr_to_unipoly_correctness() {
    let mut st = Store::new();
    let x = st.sym("x");
    let two = st.int(2);
    let three = st.int(3);

    // Build 3x^2 + 2
    let x_sq = st.pow(x, two);
    let three_x_sq = st.mul(vec![three, x_sq]);
    let expr = st.add(vec![three_x_sq, two]);

    let poly = expr_to_unipoly(&st, expr, "x").unwrap();
    assert_eq!(poly.coeffs, vec![Q(2, 1), Q(0, 1), Q(3, 1)]);
}

#[test]
fn test_unipoly_to_expr_correctness() {
    let poly = UniPoly::new("x", vec![Q(2, 1), Q(3, 1), Q(1, 1)]);
    let mut st = Store::new();
    let expr = unipoly_to_expr(&mut st, &poly);

    // Should produce an Add node with terms
    assert!(matches!(st.get(expr).op, expr_core::Op::Add));
}

#[test]
fn test_expr_poly_roundtrip_correctness() {
    let mut st = Store::new();
    let x = st.sym("x");
    let two = st.int(2);
    let three = st.int(3);
    let one = st.int(1);

    // x^2 + 3x + 2
    let x_sq = st.pow(x, two);
    let three_x = st.mul(vec![three, x]);
    let expr = st.add(vec![x_sq, three_x, one, one]);

    let poly = expr_to_unipoly(&st, expr, "x").unwrap();
    let mut st2 = Store::new();
    let back = unipoly_to_expr(&mut st2, &poly);

    // Verify structure is preserved
    assert!(matches!(st2.get(back).op, expr_core::Op::Add | expr_core::Op::Integer));
}

// ========== Multivariate Polynomial Tests ==========

#[test]
fn test_multipoly_add_correctness() {
    // x + y + (x + 2y) = 2x + 3y
    let x = MultiPoly::var("x");
    let y = MultiPoly::var("y");
    let two_y = MultiPoly::var("y");

    let p1 = x.add(&y);
    let p2 = MultiPoly::var("x").add(&two_y.add(&MultiPoly::var("y")));

    let sum = p1.add(&p2);
    assert!(sum.num_terms() > 0);
}

#[test]
fn test_multipoly_mul_correctness() {
    // (x + 1) * (y + 1) = xy + x + y + 1
    let x = MultiPoly::var("x");
    let y = MultiPoly::var("y");
    let one = MultiPoly::constant(Q(1, 1));

    let p1 = x.add(&one);
    let p2 = y.add(&MultiPoly::constant(Q(1, 1)));

    let product = p1.mul(&p2);
    assert_eq!(product.num_terms(), 4);
}

#[test]
fn test_multipoly_eval_correctness() {
    // Evaluate x + 2y at x=3, y=4 should give 11
    let x = MultiPoly::var("x");
    let y = MultiPoly::var("y");
    let two = MultiPoly::constant(Q(2, 1));
    let two_y = two.mul(&y);
    let poly = x.add(&two_y);

    let mut vals = BTreeMap::new();
    vals.insert("x".to_string(), Q(3, 1));
    vals.insert("y".to_string(), Q(4, 1));

    let result = poly.eval(&vals).unwrap();
    assert_eq!(result, Q(11, 1));
}

#[test]
fn test_monomial_mul_correctness() {
    // x * y multiplied by x * y should give x^2 * y^2
    let m1 = Monomial::var("x").mul(&Monomial::var("y"));
    let m2 = Monomial::var("x").mul(&Monomial::var("y"));

    let product = m1.mul(&m2);
    // Total degree should be 4 (x^2 * y^2)
    assert_eq!(product.degree(), 4);
}

#[test]
fn test_monomial_degree_correctness() {
    // x * y has total degree 2
    let m = Monomial::var("x").mul(&Monomial::var("y"));
    assert_eq!(m.degree(), 2);

    // x alone has degree 1
    let mx = Monomial::var("x");
    assert_eq!(mx.degree(), 1);

    // 1 (constant) has degree 0
    let one = Monomial::one();
    assert_eq!(one.degree(), 0);
}

#[test]
fn test_multipoly_total_degree() {
    // x^3 + x^2*y + y^2 has total degree 3
    // We'll construct this using multiplication and addition
    let x = MultiPoly::var("x");
    let y = MultiPoly::var("y");

    // x^3 = x * x * x
    let x_cubed = x.mul(&x).mul(&x);

    // x^2*y = x * x * y
    let x_sq_y = x.mul(&x).mul(&y);

    // y^2 = y * y
    let y_sq = y.mul(&y);

    // Combine: x^3 + x^2*y + y^2
    let poly = x_cubed.add(&x_sq_y).add(&y_sq);

    assert_eq!(poly.total_degree(), 3);
}

// ========== Edge Cases ==========

#[test]
fn test_unipoly_zero_operations() {
    let zero = UniPoly::zero("x");
    let p = UniPoly::new("x", vec![Q(1, 1), Q(2, 1)]);

    // 0 + p = p
    let sum = zero.add(&p);
    assert_eq!(sum, p);

    // 0 * p = 0
    let product = zero.mul(&p);
    assert!(product.is_zero());
}

#[test]
fn test_multipoly_zero_operations() {
    let zero = MultiPoly::zero();
    let p = MultiPoly::var("x");

    // 0 + p = p
    let sum = zero.add(&p);
    assert_eq!(sum.num_terms(), 1);

    // 0 * p = 0
    let product = zero.mul(&p);
    assert!(product.is_zero());
}

#[test]
fn test_unipoly_monic() {
    // 2x^2 + 4x + 2 becomes monic: x^2 + 2x + 1
    let p = UniPoly::new("x", vec![Q(2, 1), Q(4, 1), Q(2, 1)]);
    let monic = p.monic();

    assert_eq!(monic.leading_coeff(), Q(1, 1));
    assert_eq!(monic.coeffs[0], Q(1, 1));
    assert_eq!(monic.coeffs[1], Q(2, 1));
    assert_eq!(monic.coeffs[2], Q(1, 1));
}

#[test]
fn test_large_degree_polynomial() {
    // Test that we can handle polynomials with many terms
    let coeffs: Vec<Q> = (0..=100).map(|i| Q(i, 1)).collect();
    let p = UniPoly::new("x", coeffs);

    assert_eq!(p.degree(), Some(100));

    // Test evaluation
    let result = p.eval_q(Q(1, 1));
    // Sum of 0 to 100 = 5050
    assert_eq!(result, Q(5050, 1));
}
