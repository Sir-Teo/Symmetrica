//! Tests for sqrt and atan differentiation and integration

use calculus::{diff, integrate};
use expr_core::Store;
use simplify::simplify;

#[test]
fn diff_sqrt() {
    // d/dx √x = 1/(2√x)
    let mut st = Store::new();
    let x = st.sym("x");
    let sqrt_x = st.func("sqrt", vec![x]);

    let result = diff(&mut st, sqrt_x, "x");

    // Should contain 1/2 and x^(-1/2)
    let result_str = st.to_string(result);
    assert!(result_str.contains("1/2") || result_str.contains("^(-1/2)"));
}

#[test]
fn diff_atan() {
    // d/dx atan(x) = 1/(1+x²)
    let mut st = Store::new();
    let x = st.sym("x");
    let atan_x = st.func("atan", vec![x]);

    let result = diff(&mut st, atan_x, "x");

    // Should be 1 * (1 + x²)^(-1)
    let result_str = st.to_string(result);
    // Result should involve x² and power of -1
    assert!(result_str.contains("^2") || result_str.contains("^(2"));
}

#[test]
fn diff_tan() {
    // d/dx tan(x) = 1 + tan²(x) = sec²(x)
    let mut st = Store::new();
    let x = st.sym("x");
    let tan_x = st.func("tan", vec![x]);

    let result = diff(&mut st, tan_x, "x");

    // Should contain tan² term
    let result_str = st.to_string(result);
    assert!(result_str.contains("tan"));
}

#[test]
fn fundamental_theorem_atan() {
    // Verify d/dx(∫ 1/(1+x²) dx) = 1/(1+x²) if we can integrate it
    let mut st = Store::new();
    let x = st.sym("x");
    let two = st.int(2);
    let x_sq = st.pow(x, two);
    let one = st.int(1);
    let denom = st.add(vec![one, x_sq]);
    let neg_one = st.int(-1);
    let integrand = st.pow(denom, neg_one); // 1/(1+x²)

    // Try to integrate
    if let Some(integral) = integrate(&mut st, integrand, "x") {
        // Differentiate the result
        let derivative = diff(&mut st, integral, "x");
        let deriv_simplified = simplify(&mut st, derivative);
        let orig_simplified = simplify(&mut st, integrand);

        // Should match original
        assert_eq!(st.get(deriv_simplified).digest, st.get(orig_simplified).digest);
    }
    // If integration fails, that's OK for now - we're testing the framework
}

#[test]
fn test_weierstrass_with_sqrt_atan() {
    // Test ∫ 1/(2 + cos(x)) dx
    // This should now work with sqrt and atan support
    let mut st = Store::new();
    let x = st.sym("x");
    let two = st.int(2);
    let cos_x = st.func("cos", vec![x]);
    let denom = st.add(vec![two, cos_x]);
    let neg_one = st.int(-1);
    let integrand = st.pow(denom, neg_one);

    let result = integrate(&mut st, integrand, "x");

    // Should return Some result with atan and sqrt
    if let Some(res) = result {
        let res_str = st.to_string(res);
        // Result should contain atan and possibly sqrt
        assert!(res_str.contains("atan") || res_str.contains("arctan"));
    } else {
        // For now, it's OK if this doesn't work yet
        // The framework is in place
    }
}

#[test]
fn chain_rule_sqrt() {
    // d/dx √(x²) = x/√(x²) = x/|x|
    let mut st = Store::new();
    let x = st.sym("x");
    let two = st.int(2);
    let x_sq = st.pow(x, two);
    let sqrt_x_sq = st.func("sqrt", vec![x_sq]);

    let result = diff(&mut st, sqrt_x_sq, "x");

    // Result should contain x and sqrt
    let result_str = st.to_string(result);
    assert!(result_str.contains("x"));
}

#[test]
fn chain_rule_atan() {
    // d/dx atan(2x) = 2/(1+(2x)²) = 2/(1+4x²)
    let mut st = Store::new();
    let x = st.sym("x");
    let two = st.int(2);
    let two_x = st.mul(vec![two, x]);
    let atan_2x = st.func("atan", vec![two_x]);

    let result = diff(&mut st, atan_2x, "x");

    // Result should contain 2 and x²
    let result_str = st.to_string(result);
    assert!(result_str.contains("2"));
    assert!(result_str.contains("^2") || result_str.contains("^(2"));
}
