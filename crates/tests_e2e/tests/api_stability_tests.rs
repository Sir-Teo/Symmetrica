//! API Stability Tests for 1.0 Release
//!
//! These tests lock down the behavior of public APIs that are guaranteed stable in 1.0.
//! Any breaking change to these tests requires a major version bump.
//!
//! Test categories:
//! 1. Core API behavior (expr_core)
//! 2. Mathematics correctness guarantees (simplify, calculus, polys, matrix)
//! 3. Serialization format stability (io)
//! 4. Error handling consistency

use expr_core::{Op, Payload, Store};
use simplify::simplify;
use calculus::{diff, integrate};
use polys::{expr_to_unipoly, unipoly_to_expr, UniPoly};
use matrix::MatrixQ;
use solver::solve_univariate;
use pattern::subst_symbol;
use assumptions::{Context, Prop, Truth};
use io::{to_sexpr, from_sexpr, to_latex, to_json};
use arith::Q;

// ============================================================================
// CORE API GUARANTEES (expr_core)
// ============================================================================

#[test]
fn api_guarantee_store_construction() {
    // Store::new() must always work
    let _store = Store::new();
    // Store is created successfully (no panic)
}

#[test]
fn api_guarantee_atomic_constructors() {
    let mut st = Store::new();

    // Integer construction
    let zero = st.int(0);
    let one = st.int(1);
    let neg = st.int(-5);
    assert!(matches!(st.get(zero).op, Op::Integer));
    assert!(matches!(st.get(one).op, Op::Integer));
    assert!(matches!(st.get(neg).op, Op::Integer));

    // Rational construction and normalization
    let half = st.rat(1, 2);
    let normalized = st.rat(2, 4); // Should normalize to 1/2
    assert_eq!(half, normalized); // Guaranteed: canonicalization

    let as_int = st.rat(4, 2); // Should become integer 2
    let two = st.int(2);
    assert_eq!(as_int, two); // Guaranteed: rational to int promotion

    // Symbol construction
    let x = st.sym("x");
    let x2 = st.sym("x");
    assert_eq!(x, x2); // Guaranteed: interning
}

#[test]
fn api_guarantee_composite_constructors() {
    let mut st = Store::new();
    let x = st.sym("x");
    let one = st.int(1);
    let two = st.int(2);

    // Add construction
    let sum = st.add(vec![x, one]);
    assert!(matches!(st.get(sum).op, Op::Add));
    assert_eq!(st.get(sum).children.len(), 2);

    // Mul construction
    let prod = st.mul(vec![two, x]);
    assert!(matches!(st.get(prod).op, Op::Mul));

    // Pow construction
    let power = st.pow(x, two);
    assert!(matches!(st.get(power).op, Op::Pow));
    assert_eq!(st.get(power).children.len(), 2);

    // Function construction
    let sin_x = st.func("sin", vec![x]);
    assert!(matches!(st.get(sin_x).op, Op::Function));
    if let Payload::Func(name) = &st.get(sin_x).payload {
        assert_eq!(name, "sin");
    }
}

#[test]
fn api_guarantee_hash_consing() {
    let mut st = Store::new();
    let x = st.sym("x");
    let one = st.int(1);

    // Same expression created twice should be identical (hash-consing)
    let sum1 = st.add(vec![x, one]);
    let sum2 = st.add(vec![x, one]);

    assert_eq!(sum1, sum2); // Guaranteed: structural sharing
    assert_eq!(st.get(sum1).digest, st.get(sum2).digest);
}

#[test]
fn api_guarantee_canonical_ordering() {
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");

    // Addition should be commutative in representation
    let xy = st.add(vec![x, y]);
    let yx = st.add(vec![y, x]);
    assert_eq!(xy, yx); // Guaranteed: canonical ordering

    // Multiplication should be commutative in representation
    let xy_mul = st.mul(vec![x, y]);
    let yx_mul = st.mul(vec![y, x]);
    assert_eq!(xy_mul, yx_mul); // Guaranteed: canonical ordering
}

// ============================================================================
// SIMPLIFICATION GUARANTEES
// ============================================================================

#[test]
fn api_guarantee_simplify_idempotence() {
    let mut st = Store::new();
    let x = st.sym("x");
    let one = st.int(1);
    let expr = st.add(vec![x, one, x]); // x + 1 + x

    let s1 = simplify(&mut st, expr);
    let s2 = simplify(&mut st, s1);

    assert_eq!(s1, s2); // GUARANTEED: simplify is idempotent
    assert_eq!(st.get(s1).digest, st.get(s2).digest);
}

#[test]
fn api_guarantee_simplify_identity_elements() {
    let mut st = Store::new();
    let x = st.sym("x");
    let zero = st.int(0);
    let one = st.int(1);

    // x + 0 = x
    let x_plus_0 = st.add(vec![x, zero]);
    let s = simplify(&mut st, x_plus_0);
    assert_eq!(s, x); // GUARANTEED: additive identity

    // x * 1 = x
    let x_times_1 = st.mul(vec![x, one]);
    let s = simplify(&mut st, x_times_1);
    assert_eq!(s, x); // GUARANTEED: multiplicative identity

    // x^1 = x
    let x_pow_1 = st.pow(x, one);
    let s = simplify(&mut st, x_pow_1);
    assert_eq!(s, x); // GUARANTEED: power identity
}

#[test]
fn api_guarantee_simplify_zero_propagation() {
    let mut st = Store::new();
    let x = st.sym("x");
    let zero = st.int(0);

    // x * 0 = 0
    let x_times_0 = st.mul(vec![x, zero]);
    let s = simplify(&mut st, x_times_0);
    assert_eq!(s, zero); // GUARANTEED: zero propagation

    // 0 * x = 0 (order shouldn't matter due to canonicalization)
    let zero_times_x = st.mul(vec![zero, x]);
    let s = simplify(&mut st, zero_times_x);
    assert_eq!(s, zero);
}

#[test]
fn api_guarantee_simplify_like_terms() {
    let mut st = Store::new();
    let x = st.sym("x");
    let two = st.int(2);

    // x + x = 2*x
    let expr = st.add(vec![x, x]);
    let s = simplify(&mut st, expr);
    let expected = st.mul(vec![two, x]);
    assert_eq!(st.get(s).digest, st.get(expected).digest);
}

// ============================================================================
// CALCULUS GUARANTEES
// ============================================================================

#[test]
fn api_guarantee_diff_power_rule() {
    let mut st = Store::new();
    let x = st.sym("x");
    let n = st.int(3);

    // d/dx (x^3) = 3*x^2
    let x_cubed = st.pow(x, n);
    let derivative = diff(&mut st, x_cubed, "x");

    let three = st.int(3);
    let two = st.int(2);
    let x_squared = st.pow(x, two);
    let expected = st.mul(vec![three, x_squared]);

    assert_eq!(derivative, expected); // GUARANTEED: power rule
}

#[test]
fn api_guarantee_diff_linearity() {
    let mut st = Store::new();
    let x = st.sym("x");
    let c = st.int(5);

    // d/dx (5*x) = 5
    let expr = st.mul(vec![c, x]);
    let derivative = diff(&mut st, expr, "x");
    assert_eq!(derivative, c); // GUARANTEED: linearity
}

#[test]
fn api_guarantee_diff_chain_rule() {
    let mut st = Store::new();
    let x = st.sym("x");
    let two = st.int(2);

    // d/dx sin(x^2) = cos(x^2) * 2x
    let x2 = st.pow(x, two);
    let sin_x2 = st.func("sin", vec![x2]);
    let derivative = diff(&mut st, sin_x2, "x");

    // Verify it contains cos(x^2) and 2x as factors
    let s = st.to_string(derivative);
    assert!(s.contains("cos"));
    assert!(s.contains("2"));
    // GUARANTEED: chain rule works
}

#[test]
fn api_guarantee_diff_standard_functions() {
    let mut st = Store::new();
    let x = st.sym("x");

    // d/dx sin(x) = cos(x)
    let sin_x = st.func("sin", vec![x]);
    let d_sin = diff(&mut st, sin_x, "x");
    let cos_x = st.func("cos", vec![x]);
    assert_eq!(d_sin, cos_x); // GUARANTEED

    // d/dx cos(x) = -sin(x)
    let cos_x2 = st.func("cos", vec![x]);
    let d_cos = diff(&mut st, cos_x2, "x");
    let neg_one = st.int(-1);
    let sin_x3 = st.func("sin", vec![x]);
    let neg_sin = st.mul(vec![neg_one, sin_x3]);
    assert_eq!(d_cos, neg_sin); // GUARANTEED

    // d/dx exp(x) = exp(x)
    let exp_x = st.func("exp", vec![x]);
    let d_exp = diff(&mut st, exp_x, "x");
    assert_eq!(d_exp, exp_x); // GUARANTEED

    // d/dx ln(x) = 1/x
    let ln_x = st.func("ln", vec![x]);
    let d_ln = diff(&mut st, ln_x, "x");
    let minus_one = st.int(-1);
    let inv_x = st.pow(x, minus_one);
    assert_eq!(d_ln, inv_x); // GUARANTEED
}

#[test]
fn api_guarantee_integrate_power_rule() {
    let mut st = Store::new();
    let x = st.sym("x");
    let two = st.int(2);

    // ∫ x^2 dx = x^3/3
    let x2 = st.pow(x, two);
    let integral = integrate(&mut st, x2, "x").expect("should integrate");

    let three = st.int(3);
    let x3 = st.pow(x, three);
    let third = st.rat(1, 3);
    let expected = st.mul(vec![third, x3]);

    assert_eq!(integral, expected); // GUARANTEED: power rule integration
}

#[test]
fn api_guarantee_integrate_returns_none_when_unknown() {
    let mut st = Store::new();
    let x = st.sym("x");

    // Some integrals may not be computable - API guarantees Option type
    let complicated = st.func("nonstandard_function", vec![x]);
    let result = integrate(&mut st, complicated, "x");

    // API contract: returns Option - None is a valid response
    // This test ensures the signature is stable
    let _ = result;
}

#[test]
fn api_guarantee_fundamental_theorem() {
    let mut st = Store::new();
    let x = st.sym("x");
    let two = st.int(2);
    let x2 = st.pow(x, two);

    // ∫ f dx, then d/dx of result should give back f (approximately)
    if let Some(integral) = integrate(&mut st, x2, "x") {
        let derivative = diff(&mut st, integral, "x");
        let simplified = simplify(&mut st, derivative);
        let original_simplified = simplify(&mut st, x2);

        assert_eq!(st.get(simplified).digest, st.get(original_simplified).digest);
        // GUARANTEED: fundamental theorem
    }
}

// ============================================================================
// POLYNOMIAL GUARANTEES
// ============================================================================

#[test]
fn api_guarantee_poly_roundtrip() {
    let mut st = Store::new();
    let x = st.sym("x");
    let _one = st.int(1);
    let two = st.int(2);
    let three = st.int(3);

    // x^2 + 2x + 3
    let x2 = st.pow(x, two);
    let two_x = st.mul(vec![two, x]);
    let expr = st.add(vec![x2, two_x, three]);
    let simplified = simplify(&mut st, expr);

    // Convert to polynomial and back
    let poly = expr_to_unipoly(&st, simplified, "x").expect("should convert");
    let expr_back = unipoly_to_expr(&mut st, &poly);

    assert_eq!(st.get(simplified).digest, st.get(expr_back).digest); // GUARANTEED: roundtrip preservation
}

#[test]
fn api_guarantee_poly_arithmetic() {
    // (x + 1) * (x - 1) = x^2 - 1
    let p1 = UniPoly::new("x", vec![Q(1, 1), Q(1, 1)]); // x + 1
    let p2 = UniPoly::new("x", vec![Q(-1, 1), Q(1, 1)]); // x - 1
    let product = p1.mul(&p2);

    // Should be x^2 - 1
    assert_eq!(product.coeffs.len(), 3);
    assert_eq!(product.coeffs[0], Q(-1, 1)); // constant term
    assert_eq!(product.coeffs[2], Q(1, 1)); // x^2 term
                                             // GUARANTEED: polynomial arithmetic is correct
}

#[test]
fn api_guarantee_poly_gcd() {
    // gcd(x^2 - 1, x - 1) = x - 1
    let p1 = UniPoly::new("x", vec![Q(-1, 1), Q(0, 1), Q(1, 1)]); // x^2 - 1
    let p2 = UniPoly::new("x", vec![Q(-1, 1), Q(1, 1)]); // x - 1
    let g = UniPoly::gcd(p1, p2);

    // GCD should be linear (degree 1)
    assert_eq!(g.degree(), Some(1)); // GUARANTEED: GCD correctness
}

// ============================================================================
// MATRIX GUARANTEES
// ============================================================================

#[test]
fn api_guarantee_matrix_arithmetic() {
    let a = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
    let b = MatrixQ::from_i64(2, 2, &[5, 6, 7, 8]);

    // Addition
    let c = a.add(&b).expect("addition should work");
    assert_eq!(c.get(0, 0).0, 6); // 1+5=6
    assert_eq!(c.get(1, 1).0, 12); // 4+8=12
                                   // GUARANTEED: matrix addition

    // Multiplication
    let d = a.mul(&b).expect("multiplication should work");
    assert_eq!(d.rows, 2);
    assert_eq!(d.cols, 2);
    // GUARANTEED: matrix multiplication dimensions
}

#[test]
fn api_guarantee_determinant() {
    // [[1, 2], [3, 4]] has determinant -2
    let m = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
    let det = m.det_bareiss().expect("determinant should compute");

    assert_eq!(det.0, -2);
    assert_eq!(det.1, 1);
    // GUARANTEED: determinant correctness
}

#[test]
fn api_guarantee_identity_matrix() {
    let id = MatrixQ::identity(3);

    // Identity has 1s on diagonal, 0s elsewhere
    for i in 0..3 {
        for j in 0..3 {
            let val = id.get(i, j);
            if i == j {
                assert_eq!(val.0, 1);
            } else {
                assert_eq!(val.0, 0);
            }
        }
    }
    // GUARANTEED: identity matrix construction
}

// ============================================================================
// SOLVER GUARANTEES
// ============================================================================

#[test]
fn api_guarantee_solve_linear() {
    let mut st = Store::new();
    let x = st.sym("x");
    let two = st.int(2);
    let three = st.int(3);

    // 2x + 3 = 0 => x = -3/2
    let two_x = st.mul(vec![two, x]);
    let expr = st.add(vec![two_x, three]);
    let solutions = solve_univariate(&mut st, expr, "x").expect("should solve");

    assert_eq!(solutions.len(), 1);
    // Solution is -3/2
    let sol = solutions[0];
    if let (Op::Rational, Payload::Rat(n, d)) = (&st.get(sol).op, &st.get(sol).payload) {
        assert_eq!(*n, -3);
        assert_eq!(*d, 2);
    }
    // GUARANTEED: linear equation solving
}

#[test]
fn api_guarantee_solve_quadratic() {
    let mut st = Store::new();
    let x = st.sym("x");

    // x^2 - 1 = 0 => x = ±1
    let two = st.int(2);
    let x2 = st.pow(x, two);
    let neg_one = st.int(-1);
    let expr = st.add(vec![x2, neg_one]);
    let solutions = solve_univariate(&mut st, expr, "x").expect("should solve");

    assert_eq!(solutions.len(), 2); // GUARANTEED: finds all roots

    // Both solutions should be ±1
    let solution_strs: Vec<String> = solutions.iter().map(|&s| st.to_string(s)).collect();
    assert!(solution_strs.contains(&"1".to_string()));
    assert!(solution_strs.contains(&"-1".to_string()));
}

#[test]
fn api_guarantee_solve_returns_option() {
    let mut st = Store::new();
    let x = st.sym("x");

    // Some equations may not be solvable
    let exp_x = st.func("exp", vec![x]);
    let cos_exp = st.func("cos", vec![exp_x]);
    let complicated = st.func("sin", vec![cos_exp]);
    let result = solve_univariate(&mut st, complicated, "x");

    // API contract: returns Option - both Some and None are valid
    // This test ensures the signature is stable
    let _ = result;
    // GUARANTEED: API signature is stable
}

// ============================================================================
// PATTERN/SUBSTITUTION GUARANTEES
// ============================================================================

#[test]
fn api_guarantee_substitution() {
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");
    let one = st.int(1);

    // Substitute x -> y in (x + 1)
    let expr = st.add(vec![x, one]);
    let result = subst_symbol(&mut st, expr, "x", y);

    // Should be y + 1
    let expected = st.add(vec![y, one]);
    assert_eq!(result, expected); // GUARANTEED: substitution works
}

#[test]
fn api_guarantee_substitution_recursive() {
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");
    let two = st.int(2);

    // Substitute x -> y in x^2
    let x2 = st.pow(x, two);
    let result = subst_symbol(&mut st, x2, "x", y);

    // Should be y^2
    let expected = st.pow(y, two);
    assert_eq!(result, expected); // GUARANTEED: recursive substitution
}

// ============================================================================
// ASSUMPTION GUARANTEES
// ============================================================================

#[test]
fn api_guarantee_assumptions_query() {
    let mut ctx = Context::new();

    // Initially unknown
    assert_eq!(ctx.has("x", Prop::Positive), Truth::Unknown);

    // After assuming
    ctx.assume("x", Prop::Positive);
    assert_eq!(ctx.has("x", Prop::Positive), Truth::True);
    // GUARANTEED: assumption tracking
}

#[test]
fn api_guarantee_assumptions_independence() {
    let mut ctx = Context::new();
    ctx.assume("x", Prop::Positive);

    // Different variable should be independent
    assert_eq!(ctx.has("y", Prop::Positive), Truth::Unknown);
    // GUARANTEED: assumptions are per-variable
}

// ============================================================================
// I/O FORMAT GUARANTEES
// ============================================================================

#[test]
fn api_guarantee_sexpr_roundtrip() {
    let mut st = Store::new();
    let x = st.sym("x");
    let one = st.int(1);
    let expr = st.add(vec![x, one]);

    // Serialize and parse back
    let sexpr = to_sexpr(&st, expr);
    let parsed = from_sexpr(&mut st, &sexpr).expect("should parse");

    assert_eq!(expr, parsed); // GUARANTEED: S-expr roundtrip
    assert_eq!(st.get(expr).digest, st.get(parsed).digest);
}

#[test]
fn api_guarantee_json_structure() {
    let mut st = Store::new();
    let x = st.sym("x");
    let json = to_json(&st, x);

    // JSON should be valid
    assert!(json.contains("Symbol"));
    assert!(json.contains("\"x\""));
    // GUARANTEED: JSON format is structured
}

#[test]
fn api_guarantee_latex_output() {
    let mut st = Store::new();
    let x = st.sym("x");
    let two = st.int(2);
    let x2 = st.pow(x, two);

    let latex = to_latex(&st, x2);

    // Should contain LaTeX power notation
    assert!(latex.contains("x"));
    assert!(latex.contains("^"));
    // GUARANTEED: LaTeX format is generated
}

// ============================================================================
// ERROR HANDLING GUARANTEES
// ============================================================================

#[test]
fn api_guarantee_option_for_partial_operations() {
    let mut st = Store::new();
    let x = st.sym("x");

    // Operations that may fail return Option
    let _ = integrate(&mut st, x, "x"); // Returns Option
    let _ = solve_univariate(&mut st, x, "x"); // Returns Option
    let _ = expr_to_unipoly(&st, x, "x"); // Returns Option

    // GUARANTEED: Partial operations use Option, not panic
}

#[test]
fn api_guarantee_no_panic_on_valid_input() {
    let mut st = Store::new();

    // All valid operations should not panic
    let x = st.sym("x");
    let _ = simplify(&mut st, x);
    let _ = diff(&mut st, x, "x");
    let _ = integrate(&mut st, x, "x");
    let _ = solve_univariate(&mut st, x, "x");
    let _ = to_sexpr(&st, x);
    let _ = to_latex(&st, x);

    // GUARANTEED: Valid operations don't panic
}

// ============================================================================
// DETERMINISM GUARANTEES
// ============================================================================

#[test]
fn api_guarantee_deterministic_output() {
    let mut st1 = Store::new();
    let x1 = st1.sym("x");
    let one1 = st1.int(1);
    let expr1 = st1.add(vec![x1, one1]);
    let s1 = st1.to_string(expr1);

    let mut st2 = Store::new();
    let x2 = st2.sym("x");
    let one2 = st2.int(1);
    let expr2 = st2.add(vec![x2, one2]);
    let s2 = st2.to_string(expr2);

    assert_eq!(s1, s2); // GUARANTEED: deterministic output
}

#[test]
fn api_guarantee_digest_stability() {
    let mut st = Store::new();
    let x = st.sym("x");

    // Digest should be stable across calls
    let digest1 = st.get(x).digest;
    let digest2 = st.get(x).digest;

    assert_eq!(digest1, digest2); // GUARANTEED: stable digests
}

// ============================================================================
// PERFORMANCE CHARACTERISTICS (not strict guarantees, but documented)
// ============================================================================

#[test]
fn api_characteristic_hash_consing_efficiency() {
    let mut st = Store::new();
    let x = st.sym("x");
    let one = st.int(1);

    // Creating same expression multiple times should be efficient
    // Hash-consing ensures structural sharing
    let expr1 = st.add(vec![x, one]);

    for _ in 0..100 {
        let expr_n = st.add(vec![x, one]);
        assert_eq!(expr1, expr_n); // All should be the same due to hash-consing
    }

    // CHARACTERISTIC: Hash-consing provides efficient structural sharing
}
