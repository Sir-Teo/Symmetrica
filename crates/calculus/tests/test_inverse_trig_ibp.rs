//! Tests for integration by parts of inverse trigonometric and logarithmic functions

use calculus::{diff, integrate};
use expr_core::Store;
use simplify::simplify;

#[test]
fn integrate_ln_x() {
    // ∫ ln(x) dx = x·ln(x) - x
    let mut st = Store::new();
    let x = st.sym("x");
    let ln_x = st.func("ln", vec![x]);

    let result = integrate(&mut st, ln_x, "x");

    assert!(result.is_some(), "Should integrate ln(x)");
    if let Some(integral) = result {
        let integral_str = st.to_string(integral);
        // Should contain x·ln(x) and -x terms
        assert!(integral_str.contains("ln"));
        assert!(integral_str.contains("x"));

        // Verify by differentiation
        let derivative = diff(&mut st, integral, "x");
        let deriv_simplified = simplify(&mut st, derivative);
        let orig_simplified = simplify(&mut st, ln_x);

        assert_eq!(
            st.get(deriv_simplified).digest,
            st.get(orig_simplified).digest,
            "d/dx[∫ ln(x) dx] should equal ln(x)"
        );
    }
}

#[test]
fn integrate_atan_x() {
    // ∫ atan(x) dx = x·atan(x) - (1/2)ln(1+x²)
    let mut st = Store::new();
    let x = st.sym("x");
    let atan_x = st.func("atan", vec![x]);

    let result = integrate(&mut st, atan_x, "x");

    assert!(result.is_some(), "Should integrate atan(x)");
    if let Some(integral) = result {
        let integral_str = st.to_string(integral);
        // Should contain atan and ln terms
        assert!(integral_str.contains("atan") || integral_str.contains("arctan"));
        assert!(integral_str.contains("ln"));

        // Verify by differentiation
        let derivative = diff(&mut st, integral, "x");
        let deriv_simplified = simplify(&mut st, derivative);
        let orig_simplified = simplify(&mut st, atan_x);

        assert_eq!(
            st.get(deriv_simplified).digest,
            st.get(orig_simplified).digest,
            "d/dx[∫ atan(x) dx] should equal atan(x)"
        );
    }
}

#[test]
fn integrate_atan_fundamental_theorem() {
    // Comprehensive test: ∫ atan(x) dx and verify differentiation
    let mut st = Store::new();
    let x = st.sym("x");
    let atan_x = st.func("atan", vec![x]);

    // Integrate
    let integral = integrate(&mut st, atan_x, "x").expect("atan(x) should be integrable");

    // Differentiate back
    let derivative = diff(&mut st, integral, "x");

    // Simplify both
    let deriv_simplified = simplify(&mut st, derivative);
    let orig_simplified = simplify(&mut st, atan_x);

    // Should match
    assert_eq!(st.get(deriv_simplified).digest, st.get(orig_simplified).digest);
}

#[test]
fn integrate_ln_fundamental_theorem() {
    // Comprehensive test: ∫ ln(x) dx and verify differentiation
    let mut st = Store::new();
    let x = st.sym("x");
    let ln_x = st.func("ln", vec![x]);

    // Integrate
    let integral = integrate(&mut st, ln_x, "x").expect("ln(x) should be integrable");

    // Differentiate back
    let derivative = diff(&mut st, integral, "x");

    // Simplify both
    let deriv_simplified = simplify(&mut st, derivative);
    let orig_simplified = simplify(&mut st, ln_x);

    // Should match
    assert_eq!(st.get(deriv_simplified).digest, st.get(orig_simplified).digest);
}

#[test]
fn integrate_atan_with_constant() {
    // ∫ 2·atan(x) dx = 2[x·atan(x) - (1/2)ln(1+x²)]
    let mut st = Store::new();
    let x = st.sym("x");
    let two = st.int(2);
    let atan_x = st.func("atan", vec![x]);
    let integrand = st.mul(vec![two, atan_x]);

    let result = integrate(&mut st, integrand, "x");

    assert!(result.is_some(), "Should integrate 2·atan(x)");
    if let Some(integral) = result {
        // Verify by differentiation
        let derivative = diff(&mut st, integral, "x");
        let deriv_simplified = simplify(&mut st, derivative);
        let orig_simplified = simplify(&mut st, integrand);

        assert_eq!(st.get(deriv_simplified).digest, st.get(orig_simplified).digest);
    }
}

#[test]
fn integrate_ln_with_constant() {
    // ∫ 3·ln(x) dx = 3[x·ln(x) - x]
    let mut st = Store::new();
    let x = st.sym("x");
    let three = st.int(3);
    let ln_x = st.func("ln", vec![x]);
    let integrand = st.mul(vec![three, ln_x]);

    let result = integrate(&mut st, integrand, "x");

    assert!(result.is_some(), "Should integrate 3·ln(x)");
    if let Some(integral) = result {
        // Verify by differentiation
        let derivative = diff(&mut st, integral, "x");
        let deriv_simplified = simplify(&mut st, derivative);
        let orig_simplified = simplify(&mut st, integrand);

        assert_eq!(st.get(deriv_simplified).digest, st.get(orig_simplified).digest);
    }
}

#[test]
fn atan_not_integrated_with_complex_arg() {
    // ∫ atan(x²) dx should not match standalone pattern
    // (requires more sophisticated techniques)
    let mut st = Store::new();
    let x = st.sym("x");
    let two = st.int(2);
    let x_sq = st.pow(x, two);
    let atan_x_sq = st.func("atan", vec![x_sq]);

    // This specific pattern shouldn't match the standalone atan integration
    // It would need a different approach (u-substitution or more complex IBP)
    let result = integrate(&mut st, atan_x_sq, "x");

    // It's OK if this returns None or Some, just verify no panic
    let _ = result;
}

#[test]
fn ln_not_integrated_with_complex_arg() {
    // ∫ ln(x²) dx should not match standalone pattern
    let mut st = Store::new();
    let x = st.sym("x");
    let two = st.int(2);
    let x_sq = st.pow(x, two);
    let ln_x_sq = st.func("ln", vec![x_sq]);

    // This could potentially be handled via simplification: ln(x²) = 2ln(x)
    // but our standalone function only handles ln(x) directly
    let result = integrate(&mut st, ln_x_sq, "x");

    // It's OK if this returns None or Some, just verify no panic
    let _ = result;
}
