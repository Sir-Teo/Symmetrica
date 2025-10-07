//! Lambert W Function
//!
//! The Lambert W function W(z) is the inverse of f(w) = w·e^w.
//! It satisfies: W(z)·e^{W(z)} = z
//!
//! Applications:
//! - Solving transcendental equations of the form a·e^{bx} = c·x + d
//! - Delay differential equations
//! - Combinatorics and tree enumeration
//!
//! This implementation provides:
//! - W_0(x): Principal branch for x ≥ -1/e
//! - Numeric evaluation using Halley's method
//! - Symbolic differentiation: dW/dz = W(z)/(z(1 + W(z)))

use super::SpecialFunction;
use expr_core::{ExprId, Store};

pub struct LambertWFunction;

impl SpecialFunction for LambertWFunction {
    fn name(&self) -> &str {
        "LambertW"
    }

    fn arity(&self) -> usize {
        1
    }

    /// Numerical evaluation using Halley's method
    /// W_0(x) for x >= -1/e ≈ -0.3679
    fn eval(&self, args: &[f64]) -> Option<f64> {
        if args.len() != 1 {
            return None;
        }

        let x = args[0];

        // Domain check: W_0 is defined for x >= -1/e
        const NEG_INV_E: f64 = -0.367_879_441_171_442_3;
        if x < NEG_INV_E {
            return None;
        }

        // Special cases
        if x == 0.0 {
            return Some(0.0);
        }
        if x == std::f64::consts::E {
            return Some(1.0);
        }

        // Initial guess based on asymptotic behavior
        let mut w = if x < 1.0 {
            // For small x, W(x) ≈ x
            x
        } else {
            // For large x, W(x) ≈ ln(x) - ln(ln(x))
            let ln_x = x.ln();
            ln_x - ln_x.ln()
        };

        // Newton's method: w_{n+1} = w_n - f(w_n) / f'(w_n)
        // where f(w) = w·e^w - x and f'(w) = e^w(w+1)
        for _ in 0..100 {
            let ew = w.exp();
            let wew = w * ew;
            let f = wew - x;

            if f.abs() < 1e-14 {
                break;
            }

            let fp = ew * (w + 1.0);
            let delta = f / fp;
            w -= delta;

            if delta.abs() < 1e-14 {
                break;
            }
        }

        Some(w)
    }

    /// Derivative: dW/dz = W(z) / (z(1 + W(z)))
    fn derivative(&self, store: &mut Store, args: &[ExprId], _arg_index: usize) -> Option<ExprId> {
        if args.len() != 1 {
            return None;
        }

        let z = args[0];

        // W(z)
        let w_z = store.func("LambertW", vec![z]);

        // 1 + W(z)
        let one = store.int(1);
        let one_plus_w = store.add(vec![one, w_z]);

        // z(1 + W(z))
        let denom = store.mul(vec![z, one_plus_w]);

        // W(z) / (z(1 + W(z)))
        let neg_one = store.int(-1);
        let inv_denom = store.pow(denom, neg_one);
        Some(store.mul(vec![w_z, inv_denom]))
    }

    /// Series expansion around z=0: W(z) = Σ (-n)^{n-1}/n! z^n
    fn series(&self, store: &mut Store, args: &[ExprId], order: usize) -> Option<ExprId> {
        if args.len() != 1 {
            return None;
        }

        let z = args[0];

        if order == 0 {
            return Some(store.int(0));
        }

        // W(z) = z - z^2 + 3z^3/2 - 8z^4/3 + 125z^5/24 - ...
        // General term: (-n)^{n-1}/n! z^n
        let mut terms: Vec<ExprId> = Vec::new();

        for n in 1..=order.min(6) {
            // Compute (-n)^{n-1}
            let neg_n = -(n as i64);
            let mut coeff_num = 1i128;
            for _ in 0..(n - 1) {
                coeff_num = coeff_num.saturating_mul(neg_n as i128);
                if coeff_num == 0 || coeff_num.unsigned_abs() > i64::MAX as u128 {
                    break;
                }
            }

            if coeff_num == 0 || coeff_num.unsigned_abs() > i64::MAX as u128 {
                break;
            }

            // Compute n!
            let mut fact = 1i128;
            for i in 1..=n {
                fact = fact.saturating_mul(i as i128);
                if fact == 0 || fact.unsigned_abs() > i64::MAX as u128 {
                    break;
                }
            }

            if fact == 0 || fact.unsigned_abs() > i64::MAX as u128 {
                break;
            }

            let coeff = store.rat(coeff_num as i64, fact as i64);
            let n_i = store.int(n as i64);
            let pow = store.pow(z, n_i);
            let term = store.mul(vec![coeff, pow]);
            terms.push(term);
        }

        if terms.is_empty() {
            Some(store.int(0))
        } else {
            Some(store.add(terms))
        }
    }
}

/// Create a LambertW function expression
pub fn lambert_w(store: &mut Store, arg: ExprId) -> ExprId {
    store.func("LambertW", vec![arg])
}

#[cfg(test)]
mod tests {
    use super::*;
    use expr_core::Store;

    #[test]
    fn lambert_w_at_zero() {
        let w = LambertWFunction;
        assert_eq!(w.eval(&[0.0]), Some(0.0));
    }

    #[test]
    fn lambert_w_at_e() {
        let w = LambertWFunction;
        let result = w.eval(&[std::f64::consts::E]).unwrap();
        assert!((result - 1.0).abs() < 1e-10);
    }

    #[test]
    fn lambert_w_at_one() {
        let w = LambertWFunction;
        // W(1) should exist
        let result = w.eval(&[1.0]);
        assert!(result.is_some());
    }

    #[test]
    fn lambert_w_negative_domain() {
        let w = LambertWFunction;
        // W(-0.2) should work (> -1/e)
        let result = w.eval(&[-0.2]);
        assert!(result.is_some());

        // W(-0.5) should fail (< -1/e)
        let result = w.eval(&[-0.5]);
        assert!(result.is_none());
    }

    #[test]
    fn lambert_w_derivative_symbolic() {
        let mut st = Store::new();
        let x = st.sym("x");

        let w = LambertWFunction;
        let deriv = w.derivative(&mut st, &[x], 0).unwrap();

        let result = st.to_string(deriv);
        assert!(result.contains("LambertW"));
    }

    #[test]
    fn lambert_w_series() {
        let mut st = Store::new();
        let x = st.sym("x");

        let w = LambertWFunction;
        let series = w.series(&mut st, &[x], 5).unwrap();

        // Should have terms up to x^5
        let s = st.to_string(series);
        assert!(s.contains("x"));
    }

    #[test]
    fn lambert_w_identity_check() {
        let w = LambertWFunction;
        let x = 2.0;
        let w_x = w.eval(&[x]).unwrap();

        // Verify: W(x) * e^{W(x)} = x
        let identity = w_x * w_x.exp();
        assert!((identity - x).abs() < 1e-10);
    }
}
