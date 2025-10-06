//! Property tests for trigonometric integration patterns
//! Tests robustness: integration and differentiation complete without panics
//! Note: Full fundamental theorem verification requires deep trig identity simplification,
//! which is beyond current simplifier capabilities. These tests verify structural correctness.

use calculus::{diff, integrate};
use expr_core::Store;
use proptest::prelude::*;
/// Generate valid even powers for trig functions (2, 4, 6, ...)
fn even_power() -> impl Strategy<Value = i64> {
    (1i64..=5).prop_map(|k| 2 * k)
}

/// Generate valid odd powers for trig functions (1, 3, 5, ...)
fn odd_power() -> impl Strategy<Value = i64> {
    (0i64..=4).prop_map(|k| 2 * k + 1)
}

proptest! {
    /// Property: ∫ sin^(2k)(x) dx completes without panic and produces differentiable result
    #[test]
    fn prop_integrate_sin_even_no_panic(n in even_power()) {
        let mut st = Store::new();
        let x = st.sym("x");
        let sinx = st.func("sin", vec![x]);
        let n_e = st.int(n);
        let sin_n = st.pow(sinx, n_e);

        // Integration should succeed for even powers
        let integral = integrate(&mut st, sin_n, "x");
        prop_assert!(integral.is_some(), "Integration should succeed for sin^{}(x)", n);

        // Result should be differentiable without panic
        let integral_id = integral.unwrap();
        let _derivative = diff(&mut st, integral_id, "x");
    }

    /// Property: ∫ cos^(2k)(x) dx completes without panic and produces differentiable result
    #[test]
    fn prop_integrate_cos_even_no_panic(n in even_power()) {
        let mut st = Store::new();
        let x = st.sym("x");
        let cosx = st.func("cos", vec![x]);
        let n_e = st.int(n);
        let cos_n = st.pow(cosx, n_e);

        // Integration should succeed for even powers
        let integral = integrate(&mut st, cos_n, "x");
        prop_assert!(integral.is_some(), "Integration should succeed for cos^{}(x)", n);

        // Result should be differentiable without panic
        let integral_id = integral.unwrap();
        let _derivative = diff(&mut st, integral_id, "x");
    }

    /// Property: ∫ sin^m(x) * cos^n(x) dx with odd m completes without panic
    #[test]
    fn prop_integrate_sin_odd_cos_even_no_panic(
        m in odd_power(),
        n in even_power().prop_filter("n should be small for test speed", |&n| n <= 4)
    ) {
        let mut st = Store::new();
        let x = st.sym("x");
        let sinx = st.func("sin", vec![x]);
        let cosx = st.func("cos", vec![x]);
        let m_e = st.int(m);
        let n_e = st.int(n);
        let sin_m = st.pow(sinx, m_e);
        let cos_n = st.pow(cosx, n_e);
        let product = st.mul(vec![sin_m, cos_n]);

        let integral = integrate(&mut st, product, "x");
        prop_assert!(integral.is_some(), "Integration should succeed for sin^{}(x) * cos^{}(x)", m, n);

        let integral_id = integral.unwrap();
        let _derivative = diff(&mut st, integral_id, "x");
    }

    /// Property: ∫ sin^m(x) * cos^n(x) dx with odd n completes without panic
    #[test]
    fn prop_integrate_sin_even_cos_odd_no_panic(
        m in even_power().prop_filter("m should be small for test speed", |&m| m <= 4),
        n in odd_power()
    ) {
        let mut st = Store::new();
        let x = st.sym("x");
        let sinx = st.func("sin", vec![x]);
        let cosx = st.func("cos", vec![x]);
        let m_e = st.int(m);
        let n_e = st.int(n);
        let sin_m = st.pow(sinx, m_e);
        let cos_n = st.pow(cosx, n_e);
        let product = st.mul(vec![sin_m, cos_n]);

        let integral = integrate(&mut st, product, "x");
        prop_assert!(integral.is_some(), "Integration should succeed for sin^{}(x) * cos^{}(x)", m, n);

        let integral_id = integral.unwrap();
        let _derivative = diff(&mut st, integral_id, "x");
    }

    /// Property: ∫ sin^(2k)(x) * cos^(2l)(x) dx completes without panic
    #[test]
    fn prop_integrate_sin_even_cos_even_no_panic(
        m in even_power().prop_filter("keep small for speed", |&m| m <= 4),
        n in even_power().prop_filter("keep small for speed", |&n| n <= 4)
    ) {
        let mut st = Store::new();
        let x = st.sym("x");
        let sinx = st.func("sin", vec![x]);
        let cosx = st.func("cos", vec![x]);
        let m_e = st.int(m);
        let n_e = st.int(n);
        let sin_m = st.pow(sinx, m_e);
        let cos_n = st.pow(cosx, n_e);
        let product = st.mul(vec![sin_m, cos_n]);

        let integral = integrate(&mut st, product, "x");
        prop_assert!(integral.is_some(), "Integration should succeed for sin^{}(x) * cos^{}(x)", m, n);

        let integral_id = integral.unwrap();
        let _derivative = diff(&mut st, integral_id, "x");
    }

    /// Property: ∫ c * sin^n(x) dx with rational coefficient completes without panic
    #[test]
    fn prop_integrate_sin_with_coeff_no_panic(
        n in even_power().prop_filter("keep small", |&n| n <= 6),
        c_num in -5i64..=5i64,
        c_den in 1i64..=5i64
    ) {
        prop_assume!(c_num != 0); // Skip zero coefficient

        let mut st = Store::new();
        let x = st.sym("x");
        let sinx = st.func("sin", vec![x]);
        let n_e = st.int(n);
        let sin_n = st.pow(sinx, n_e);
        let coeff = st.rat(c_num, c_den);
        let expr = st.mul(vec![coeff, sin_n]);

        let integral = integrate(&mut st, expr, "x");
        prop_assert!(integral.is_some(), "Integration should succeed for {} * sin^{}(x)", format!("{}/{}", c_num, c_den), n);

        let integral_id = integral.unwrap();
        let _derivative = diff(&mut st, integral_id, "x");
    }
}
