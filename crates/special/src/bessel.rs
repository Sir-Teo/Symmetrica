//! Bessel Functions
//!
//! Implements:
//! - J_ν(x): Bessel function of the first kind
//! - Y_ν(x): Bessel function of the second kind (Neumann function)
//! - I_ν(x): Modified Bessel function of the first kind
//! - K_ν(x): Modified Bessel function of the second kind
//!
//! Properties:
//! - J_ν satisfies: x²y'' + xy' + (x² - ν²)y = 0
//! - Recurrence: J_{ν-1}(x) + J_{ν+1}(x) = (2ν/x)J_ν(x)
//! - d/dx J_ν(x) = (J_{ν-1}(x) - J_{ν+1}(x))/2

use super::SpecialFunction;
use expr_core::{ExprId, Store};

pub struct BesselJFunction;

impl SpecialFunction for BesselJFunction {
    fn name(&self) -> &str {
        "BesselJ"
    }

    fn arity(&self) -> usize {
        2 // BesselJ(nu, x)
    }

    /// Numerical evaluation using series expansion for small x
    fn eval(&self, args: &[f64]) -> Option<f64> {
        if args.len() != 2 {
            return None;
        }

        let nu = args[0];
        let x = args[1];

        // Only handle integer orders for now
        if nu.fract() != 0.0 {
            return None;
        }

        let n = nu as i32;
        if n < 0 {
            // J_{-n}(x) = (-1)^n J_n(x)
            let sign = if n % 2 == 0 { 1.0 } else { -1.0 };
            return self.eval(&[(-n) as f64, x]).map(|v| sign * v);
        }

        // Series expansion: J_n(x) = Σ_{k=0}^∞ (-1)^k (x/2)^{n+2k} / (k! Γ(n+k+1))
        let half_x = x / 2.0;
        let mut sum = 0.0;
        let mut term = half_x.powi(n) / factorial(n as u32) as f64;

        for k in 0..50 {
            sum += term;
            if term.abs() < 1e-15 * sum.abs() {
                break;
            }
            term *= -half_x * half_x / ((k + 1) as f64 * (n as f64 + k as f64 + 1.0));
        }

        Some(sum)
    }

    /// Derivative: d/dx J_ν(x) = (J_{ν-1}(x) - J_{ν+1}(x))/2
    fn derivative(&self, store: &mut Store, args: &[ExprId], arg_index: usize) -> Option<ExprId> {
        if args.len() != 2 || arg_index != 1 {
            return None;
        }

        let nu = args[0];
        let x = args[1];

        // J_{ν-1}(x)
        let one = store.int(1);
        let neg_one = store.int(-1);
        let neg_one_term = store.mul(vec![neg_one, one]);
        let nu_minus_1 = store.add(vec![nu, neg_one_term]);
        let j_prev = store.func("BesselJ", vec![nu_minus_1, x]);

        // J_{ν+1}(x)
        let nu_plus_1 = store.add(vec![nu, one]);
        let j_next = store.func("BesselJ", vec![nu_plus_1, x]);

        // (J_{ν-1}(x) - J_{ν+1}(x))/2
        let neg_j_next = store.mul(vec![neg_one, j_next]);
        let diff = store.add(vec![j_prev, neg_j_next]);
        let half = store.rat(1, 2);
        Some(store.mul(vec![half, diff]))
    }

    /// Series expansion around x=0
    fn series(&self, store: &mut Store, args: &[ExprId], order: usize) -> Option<ExprId> {
        if args.len() != 2 {
            return None;
        }

        let nu = args[0];
        let x = args[1];

        // Only handle integer nu for series
        let n = match (&store.get(nu).op, &store.get(nu).payload) {
            (expr_core::Op::Integer, expr_core::Payload::Int(k)) if *k >= 0 => *k as usize,
            _ => return None,
        };

        if order < n {
            return Some(store.int(0));
        }

        // J_n(x) = Σ_{k=0}^∞ (-1)^k (x/2)^{n+2k} / (k! (n+k)!)
        let mut terms: Vec<ExprId> = Vec::new();
        let half = store.rat(1, 2);
        let half_x = store.mul(vec![x, half]);

        let mut k = 0;
        loop {
            let p = n + 2 * k;
            if p > order {
                break;
            }

            let mut fact_k = 1i128;
            for i in 1..=k {
                fact_k = fact_k.saturating_mul(i as i128);
                if fact_k == 0 || fact_k.unsigned_abs() > i64::MAX as u128 {
                    break;
                }
            }
            if fact_k == 0 || fact_k.unsigned_abs() > i64::MAX as u128 {
                break;
            }

            let mut fact_nk = 1i128;
            for i in 1..=(n + k) {
                fact_nk = fact_nk.saturating_mul(i as i128);
                if fact_nk == 0 || fact_nk.unsigned_abs() > i64::MAX as u128 {
                    break;
                }
            }
            if fact_nk == 0 || fact_nk.unsigned_abs() > i64::MAX as u128 {
                break;
            }

            let denom = fact_k.saturating_mul(fact_nk);
            if denom == 0 || denom.unsigned_abs() > i64::MAX as u128 {
                break;
            }

            let sign = if k.is_multiple_of(2) { 1 } else { -1 };
            let coeff = store.rat(sign, denom as i64);
            let p_i = store.int(p as i64);
            let pow = store.pow(half_x, p_i);
            let term = store.mul(vec![coeff, pow]);
            terms.push(term);

            k += 1;
        }

        if terms.is_empty() {
            Some(store.int(0))
        } else {
            Some(store.add(terms))
        }
    }
}

fn factorial(n: u32) -> u128 {
    (1..=n as u128).product::<u128>().max(1)
}

/// Create a BesselJ function expression
pub fn bessel_j(store: &mut Store, nu: ExprId, x: ExprId) -> ExprId {
    store.func("BesselJ", vec![nu, x])
}

#[cfg(test)]
mod tests {
    use super::*;
    use expr_core::Store;

    #[test]
    fn bessel_j0_at_zero() {
        let b = BesselJFunction;
        assert_eq!(b.eval(&[0.0, 0.0]), Some(1.0));
    }

    #[test]
    fn bessel_j1_at_zero() {
        let b = BesselJFunction;
        let result = b.eval(&[1.0, 0.0]).unwrap();
        assert!(result.abs() < 1e-10);
    }

    #[test]
    fn bessel_j0_small_x() {
        let b = BesselJFunction;
        // J_0(1) ≈ 0.7651976866
        let result = b.eval(&[0.0, 1.0]).unwrap();
        assert!((result - 0.7651976866).abs() < 1e-6);
    }

    #[test]
    fn bessel_j_derivative_symbolic() {
        let mut st = Store::new();
        let nu = st.int(0);
        let x = st.sym("x");

        let b = BesselJFunction;
        let deriv = b.derivative(&mut st, &[nu, x], 1).unwrap();

        let result = st.to_string(deriv);
        assert!(result.contains("BesselJ"));
    }

    #[test]
    fn bessel_j_series_order0() {
        let mut st = Store::new();
        let nu = st.int(0);
        let x = st.sym("x");

        let b = BesselJFunction;
        let series = b.series(&mut st, &[nu, x], 4).unwrap();

        // Should have terms up to x^4
        let s = st.to_string(series);
        assert!(s.contains("x"));
    }
}
