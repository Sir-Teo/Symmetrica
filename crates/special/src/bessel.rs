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

/// BesselY (Neumann function) - second kind
pub struct BesselYFunction;

impl SpecialFunction for BesselYFunction {
    fn name(&self) -> &str {
        "BesselY"
    }

    fn arity(&self) -> usize {
        2 // BesselY(nu, x)
    }

    /// Numerical evaluation using relation: Y_n(x) = (J_n(x)cos(nπ) - J_{-n}(x))/sin(nπ)
    /// For integer n, use limit formula
    fn eval(&self, args: &[f64]) -> Option<f64> {
        if args.len() != 2 {
            return None;
        }

        let nu = args[0];
        let x = args[1];

        if x <= 0.0 {
            return None; // Y_ν undefined for x ≤ 0
        }

        // Only handle integer orders for now
        if nu.fract() != 0.0 {
            return None;
        }

        let n = nu as i32;
        let bessel_j = BesselJFunction;

        // For small x, use series expansion (simplified)
        // Y_0(x) ≈ (2/π)[ln(x/2) + γ]J_0(x) + ... (Euler-Mascheroni constant γ ≈ 0.5772)
        if n == 0 {
            let j0 = bessel_j.eval(&[0.0, x])?;
            let gamma_euler = 0.5772156649015329;
            let term1 = (2.0 / std::f64::consts::PI) * (x / 2.0).ln() * j0;
            let term2 = (2.0 / std::f64::consts::PI) * gamma_euler * j0;
            // Simplified approximation
            return Some(term1 + term2);
        }

        // For n > 0, use recurrence or approximation
        // This is a placeholder - full implementation would use proper series/asymptotics
        None
    }

    /// Derivative: d/dx Y_ν(x) = (Y_{ν-1}(x) - Y_{ν+1}(x))/2
    fn derivative(&self, store: &mut Store, args: &[ExprId], arg_index: usize) -> Option<ExprId> {
        if args.len() != 2 || arg_index != 1 {
            return None;
        }

        let nu = args[0];
        let x = args[1];

        let one = store.int(1);
        let neg_one = store.int(-1);
        let neg_one_term = store.mul(vec![neg_one, one]);
        let nu_minus_1 = store.add(vec![nu, neg_one_term]);
        let y_prev = store.func("BesselY", vec![nu_minus_1, x]);

        let nu_plus_1 = store.add(vec![nu, one]);
        let y_next = store.func("BesselY", vec![nu_plus_1, x]);

        let neg_y_next = store.mul(vec![neg_one, y_next]);
        let diff = store.add(vec![y_prev, neg_y_next]);
        let half = store.rat(1, 2);
        Some(store.mul(vec![half, diff]))
    }

    fn series(&self, _store: &mut Store, _args: &[ExprId], _order: usize) -> Option<ExprId> {
        None // Series expansion deferred
    }
}

/// Modified Bessel function of the first kind I_ν(x)
pub struct BesselIFunction;

impl SpecialFunction for BesselIFunction {
    fn name(&self) -> &str {
        "BesselI"
    }

    fn arity(&self) -> usize {
        2 // BesselI(nu, x)
    }

    /// Numerical evaluation: I_n(x) = i^{-n} J_n(ix) for real x
    /// Series: I_n(x) = Σ_{k=0}^∞ (x/2)^{n+2k} / (k! (n+k)!)
    fn eval(&self, args: &[f64]) -> Option<f64> {
        if args.len() != 2 {
            return None;
        }

        let nu = args[0];
        let x = args[1];

        // Only handle integer orders
        if nu.fract() != 0.0 {
            return None;
        }

        let n = nu as i32;
        if n < 0 {
            // I_{-n}(x) = I_n(x) for integer n
            return self.eval(&[(-n) as f64, x]);
        }

        // Series expansion: I_n(x) = Σ_{k=0}^∞ (x/2)^{n+2k} / (k! (n+k)!)
        let half_x = x / 2.0;
        let mut sum = 0.0;
        let mut term = half_x.powi(n) / factorial(n as u32) as f64;

        for k in 0..50 {
            sum += term;
            if term.abs() < 1e-15 * sum.abs() {
                break;
            }
            term *= half_x * half_x / ((k + 1) as f64 * (n as f64 + k as f64 + 1.0));
        }

        Some(sum)
    }

    /// Derivative: d/dx I_ν(x) = (I_{ν-1}(x) + I_{ν+1}(x))/2
    fn derivative(&self, store: &mut Store, args: &[ExprId], arg_index: usize) -> Option<ExprId> {
        if args.len() != 2 || arg_index != 1 {
            return None;
        }

        let nu = args[0];
        let x = args[1];

        let one = store.int(1);
        let neg_one = store.int(-1);
        let neg_one_term = store.mul(vec![neg_one, one]);
        let nu_minus_1 = store.add(vec![nu, neg_one_term]);
        let i_prev = store.func("BesselI", vec![nu_minus_1, x]);

        let nu_plus_1 = store.add(vec![nu, one]);
        let i_next = store.func("BesselI", vec![nu_plus_1, x]);

        let sum = store.add(vec![i_prev, i_next]);
        let half = store.rat(1, 2);
        Some(store.mul(vec![half, sum]))
    }

    fn series(&self, _store: &mut Store, _args: &[ExprId], _order: usize) -> Option<ExprId> {
        None // Series expansion deferred
    }
}

/// Modified Bessel function of the second kind K_ν(x)
pub struct BesselKFunction;

impl SpecialFunction for BesselKFunction {
    fn name(&self) -> &str {
        "BesselK"
    }

    fn arity(&self) -> usize {
        2 // BesselK(nu, x)
    }

    /// Numerical evaluation: K_n(x) = (π/2) * (I_{-n}(x) - I_n(x)) / sin(nπ)
    /// For integer n, use limit formula
    fn eval(&self, args: &[f64]) -> Option<f64> {
        if args.len() != 2 {
            return None;
        }

        let nu = args[0];
        let x = args[1];

        if x <= 0.0 {
            return None; // K_ν undefined for x ≤ 0
        }

        // Only handle integer orders for now
        if nu.fract() != 0.0 {
            return None;
        }

        let n = nu.abs() as i32;
        let _bessel_i = BesselIFunction;

        // For small x, K_0(x) ≈ -ln(x/2) - γ + O(x²)
        if n == 0 && x < 1.0 {
            let gamma_euler = 0.5772156649015329;
            return Some(-(x / 2.0).ln() - gamma_euler);
        }

        // Placeholder - full implementation would use proper series/asymptotics
        None
    }

    /// Derivative: d/dx K_ν(x) = -(K_{ν-1}(x) + K_{ν+1}(x))/2
    fn derivative(&self, store: &mut Store, args: &[ExprId], arg_index: usize) -> Option<ExprId> {
        if args.len() != 2 || arg_index != 1 {
            return None;
        }

        let nu = args[0];
        let x = args[1];

        let one = store.int(1);
        let neg_one = store.int(-1);
        let neg_one_term = store.mul(vec![neg_one, one]);
        let nu_minus_1 = store.add(vec![nu, neg_one_term]);
        let k_prev = store.func("BesselK", vec![nu_minus_1, x]);

        let nu_plus_1 = store.add(vec![nu, one]);
        let k_next = store.func("BesselK", vec![nu_plus_1, x]);

        let sum = store.add(vec![k_prev, k_next]);
        let half = store.rat(1, 2);
        let neg_half = store.mul(vec![neg_one, half]);
        Some(store.mul(vec![neg_half, sum]))
    }

    fn series(&self, _store: &mut Store, _args: &[ExprId], _order: usize) -> Option<ExprId> {
        None // Series expansion deferred
    }
}

/// Create a BesselY function expression
pub fn bessel_y(store: &mut Store, nu: ExprId, x: ExprId) -> ExprId {
    store.func("BesselY", vec![nu, x])
}

/// Create a BesselI function expression
pub fn bessel_i(store: &mut Store, nu: ExprId, x: ExprId) -> ExprId {
    store.func("BesselI", vec![nu, x])
}

/// Create a BesselK function expression
pub fn bessel_k(store: &mut Store, nu: ExprId, x: ExprId) -> ExprId {
    store.func("BesselK", vec![nu, x])
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

    #[test]
    fn bessel_i_at_zero() {
        let b = BesselIFunction;
        assert_eq!(b.eval(&[0.0, 0.0]), Some(1.0));
    }

    #[test]
    fn bessel_i_small_x() {
        let b = BesselIFunction;
        // I_0(1) ≈ 1.2661
        let result = b.eval(&[0.0, 1.0]).unwrap();
        assert!((result - 1.2661).abs() < 0.001);
    }

    #[test]
    fn bessel_i_derivative_symbolic() {
        let mut st = Store::new();
        let nu = st.int(0);
        let x = st.sym("x");

        let b = BesselIFunction;
        let deriv = b.derivative(&mut st, &[nu, x], 1).unwrap();

        let result = st.to_string(deriv);
        assert!(result.contains("BesselI"));
    }

    #[test]
    fn bessel_y_derivative_symbolic() {
        let mut st = Store::new();
        let nu = st.int(1);
        let x = st.sym("x");

        let b = BesselYFunction;
        let deriv = b.derivative(&mut st, &[nu, x], 1).unwrap();

        let result = st.to_string(deriv);
        assert!(result.contains("BesselY"));
    }

    #[test]
    fn bessel_k_derivative_symbolic() {
        let mut st = Store::new();
        let nu = st.int(0);
        let x = st.sym("x");

        let b = BesselKFunction;
        let deriv = b.derivative(&mut st, &[nu, x], 1).unwrap();

        let result = st.to_string(deriv);
        assert!(result.contains("BesselK"));
    }
}
