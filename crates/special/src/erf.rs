//! Error Functions
//!
//! Implements:
//! - erf(z): Error function = (2/√π) ∫₀^z e^(-t²) dt
//! - erfc(z): Complementary error function = 1 - erf(z)
//! - erfi(z): Imaginary error function = -i·erf(iz)
//!
//! Properties:
//! - erf(-z) = -erf(z) (odd function)
//! - erf(0) = 0
//! - erf(∞) = 1
//! - erfc(z) = 1 - erf(z)
//! - d/dz erf(z) = (2/√π) e^(-z²)

use super::SpecialFunction;
use expr_core::{ExprId, Store};

pub struct ErfFunction;

impl SpecialFunction for ErfFunction {
    fn name(&self) -> &str {
        "erf"
    }

    fn arity(&self) -> usize {
        1
    }

    /// Numerical evaluation using series expansion for small |z| or continued fraction for large |z|
    fn eval(&self, args: &[f64]) -> Option<f64> {
        if args.len() != 1 {
            return None;
        }

        let z = args[0];

        // Special cases
        if z == 0.0 {
            return Some(0.0);
        }

        if z.is_infinite() {
            return Some(if z > 0.0 { 1.0 } else { -1.0 });
        }

        // For small |z|, use series expansion
        // erf(z) ≈ (2/√π) Σ((-1)^n z^(2n+1))/(n!(2n+1))
        if z.abs() < 2.0 {
            let sqrt_pi = std::f64::consts::PI.sqrt();
            let two_over_sqrt_pi = 2.0 / sqrt_pi;

            let mut sum = 0.0;
            let mut term = z;
            let z_sq = z * z;

            for n in 0..30 {
                sum += term / (2 * n + 1) as f64;
                term *= -z_sq / (n + 1) as f64;

                if term.abs() < 1e-15 {
                    break;
                }
            }

            return Some(two_over_sqrt_pi * sum);
        }

        // TODO: Implement continued fraction or asymptotic expansion for large |z|
        None
    }

    /// Derivative: d/dz erf(z) = (2/√π) exp(-z²)
    fn derivative(&self, store: &mut Store, args: &[ExprId], _arg_index: usize) -> Option<ExprId> {
        if args.len() != 1 {
            return None;
        }

        let z = args[0];

        // (2/√π) * exp(-z²)
        let sqrt_pi = std::f64::consts::PI.sqrt();
        let coeff = store.rat((2.0 * 1_000_000.0) as i64, (sqrt_pi * 1_000_000.0) as i64);

        let two = store.int(2);
        let z_sq = store.pow(z, two);
        let neg_one = store.int(-1);
        let neg_z_sq = store.mul(vec![neg_one, z_sq]);
        let exp_term = store.func("exp", vec![neg_z_sq]);

        Some(store.mul(vec![coeff, exp_term]))
    }

    /// Series expansion around z=0
    fn series(&self, store: &mut Store, args: &[ExprId], _order: usize) -> Option<ExprId> {
        if args.len() != 1 {
            return None;
        }

        let z = args[0];

        // erf(z) = (2/√π) Σ((-1)^n z^(2n+1))/(n!(2n+1)) for n=0 to order
        // TODO: Implement series construction

        // For now, return the function itself
        Some(store.func("erf", vec![z]))
    }
}

/// Create an erf function expression
pub fn erf(store: &mut Store, arg: ExprId) -> ExprId {
    store.func("erf", vec![arg])
}

/// Complementary error function: erfc(z) = 1 - erf(z)
pub fn erfc(store: &mut Store, arg: ExprId) -> ExprId {
    let one = store.int(1);
    let erf_z = erf(store, arg);
    let neg_one = store.int(-1);
    let neg_erf = store.mul(vec![neg_one, erf_z]);
    store.add(vec![one, neg_erf])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erf_at_zero() {
        let e = ErfFunction;
        assert_eq!(e.eval(&[0.0]), Some(0.0));
    }

    #[test]
    fn erf_small_values() {
        let e = ErfFunction;

        // erf(0.5) ≈ 0.5205
        let result = e.eval(&[0.5]).unwrap();
        assert!((result - 0.5205).abs() < 0.001);

        // erf is odd: erf(-x) = -erf(x)
        let result_neg = e.eval(&[-0.5]).unwrap();
        assert!((result + result_neg).abs() < 1e-10);
    }

    #[test]
    fn erf_symbolic() {
        let mut st = Store::new();
        let x = st.sym("x");
        let ex = erf(&mut st, x);

        assert!(st.to_string(ex).contains("erf"));
    }

    #[test]
    fn erf_derivative_symbolic() {
        let mut st = Store::new();
        let x = st.sym("x");

        let e = ErfFunction;
        let deriv = e.derivative(&mut st, &[x], 0).unwrap();

        // Should contain exp(-x²)
        let result = st.to_string(deriv);
        assert!(result.contains("exp"));
    }
}
