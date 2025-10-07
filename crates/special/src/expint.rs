//! Exponential Integrals
//!
//! Implements:
//! - Ei(z): Exponential integral Ei(z) = -∫_{-z}^∞ e^(-t)/t dt
//! - E₁(z): E₁(z) = ∫_z^∞ e^(-t)/t dt
//! - Related: li(z) = Ei(ln z) (logarithmic integral)
//!
//! Properties:
//! - Ei(z) = -E₁(-z) for real z
//! - d/dz Ei(z) = e^z/z
//! - d/dz E₁(z) = -e^(-z)/z

use super::SpecialFunction;
use expr_core::{ExprId, Store};

pub struct EiFunction;

impl SpecialFunction for EiFunction {
    fn name(&self) -> &str {
        "Ei"
    }

    fn arity(&self) -> usize {
        1
    }

    /// Numerical evaluation (placeholder)
    fn eval(&self, args: &[f64]) -> Option<f64> {
        if args.len() != 1 {
            return None;
        }

        let z = args[0];

        if z == 0.0 {
            // Ei(0) is undefined (has a logarithmic singularity)
            return None;
        }

        // TODO: Implement numerical evaluation via series or continued fraction
        // For now, return None
        None
    }

    /// Derivative: d/dz Ei(z) = e^z/z
    fn derivative(&self, store: &mut Store, args: &[ExprId], _arg_index: usize) -> Option<ExprId> {
        if args.len() != 1 {
            return None;
        }

        let z = args[0];

        // exp(z) / z = exp(z) * z^(-1)
        let exp_z = store.func("exp", vec![z]);
        let neg_one = store.int(-1);
        let z_inv = store.pow(z, neg_one);

        Some(store.mul(vec![exp_z, z_inv]))
    }

    /// Series expansion (not implemented yet)
    fn series(&self, _store: &mut Store, _args: &[ExprId], _order: usize) -> Option<ExprId> {
        // TODO: Implement series expansion
        // Ei(z) = γ + ln|z| + Σ(z^n/(n·n!)) for n=1,2,3,...
        None
    }
}

/// Create an Ei function expression
pub fn ei(store: &mut Store, arg: ExprId) -> ExprId {
    store.func("Ei", vec![arg])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ei_undefined_at_zero() {
        let e = EiFunction;
        assert_eq!(e.eval(&[0.0]), None);
    }

    #[test]
    fn ei_symbolic() {
        let mut st = Store::new();
        let x = st.sym("x");
        let ex = ei(&mut st, x);

        assert!(st.to_string(ex).contains("Ei"));
    }

    #[test]
    fn ei_derivative_symbolic() {
        let mut st = Store::new();
        let x = st.sym("x");

        let e = EiFunction;
        let deriv = e.derivative(&mut st, &[x], 0).unwrap();

        // Should be exp(x)/x
        let result = st.to_string(deriv);
        assert!(result.contains("exp"));
        assert!(result.contains("x"));
    }
}
