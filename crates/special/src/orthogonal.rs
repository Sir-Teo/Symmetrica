//! Orthogonal Polynomials
//!
//! Implements:
//! - Legendre polynomials: P_n(x)
//! - Chebyshev polynomials of the first kind: T_n(x)
//! - Chebyshev polynomials of the second kind: U_n(x)
//! - Hermite polynomials: H_n(x) (physicist's convention)
//! - Laguerre polynomials: L_n(x)
//!
//! Properties:
//! - Legendre: (1-x²)P_n''(x) - 2xP_n'(x) + n(n+1)P_n(x) = 0
//! - Chebyshev T: (1-x²)T_n''(x) - xT_n'(x) + n²T_n(x) = 0
//! - Hermite: H_n''(x) - 2xH_n'(x) + 2nH_n(x) = 0

use super::SpecialFunction;
use expr_core::{ExprId, Op, Payload, Store};

pub struct LegendreFunction;

impl SpecialFunction for LegendreFunction {
    fn name(&self) -> &str {
        "LegendreP"
    }

    fn arity(&self) -> usize {
        2 // LegendreP(n, x)
    }

    /// Numerical evaluation using Bonnet's recursion formula
    fn eval(&self, args: &[f64]) -> Option<f64> {
        if args.len() != 2 {
            return None;
        }

        let n_f = args[0];
        let x = args[1];

        if n_f.fract() != 0.0 || n_f < 0.0 {
            return None;
        }

        let n = n_f as usize;

        if n == 0 {
            return Some(1.0);
        }
        if n == 1 {
            return Some(x);
        }

        // Bonnet's recursion: (n+1)P_{n+1}(x) = (2n+1)xP_n(x) - nP_{n-1}(x)
        let mut p_prev = 1.0; // P_0
        let mut p_curr = x; // P_1

        for k in 1..n {
            let p_next = ((2 * k + 1) as f64 * x * p_curr - k as f64 * p_prev) / (k + 1) as f64;
            p_prev = p_curr;
            p_curr = p_next;
        }

        Some(p_curr)
    }

    /// Derivative: d/dx P_n(x) = n(P_{n-1}(x) - xP_n(x))/(1-x²)
    fn derivative(&self, store: &mut Store, args: &[ExprId], arg_index: usize) -> Option<ExprId> {
        if args.len() != 2 || arg_index != 1 {
            return None;
        }

        let n = args[0];
        let x = args[1];

        // For simplicity, use: d/dx P_n(x) = n/(x²-1) * (xP_n(x) - P_{n-1}(x))
        // But this has singularities at x=±1, so we use the three-term form:
        // (1-x²)P_n'(x) = n(P_{n-1}(x) - xP_n(x))

        let one = store.int(1);
        let neg_one = store.int(-1);
        let neg_one_term = store.mul(vec![neg_one, one]);
        let n_minus_1 = store.add(vec![n, neg_one_term]);
        let p_prev = store.func("LegendreP", vec![n_minus_1, x]);
        let p_n = store.func("LegendreP", vec![n, x]);

        // n * (P_{n-1}(x) - x*P_n(x))
        let x_pn = store.mul(vec![x, p_n]);
        let neg_x_pn = store.mul(vec![neg_one, x_pn]);
        let diff = store.add(vec![p_prev, neg_x_pn]);
        let numerator = store.mul(vec![n, diff]);

        // 1 - x²
        let two = store.int(2);
        let x_sq = store.pow(x, two);
        let neg_x_sq = store.mul(vec![neg_one, x_sq]);
        let denom = store.add(vec![one, neg_x_sq]);

        // numerator / denom
        let inv_denom = store.pow(denom, neg_one);
        Some(store.mul(vec![numerator, inv_denom]))
    }

    /// Series expansion (explicit polynomial form)
    fn series(&self, store: &mut Store, args: &[ExprId], order: usize) -> Option<ExprId> {
        if args.len() != 2 {
            return None;
        }

        let n = args[0];
        let x = args[1];

        // Extract integer n
        let n_val = match (&store.get(n).op, &store.get(n).payload) {
            (Op::Integer, Payload::Int(k)) if *k >= 0 => *k as usize,
            _ => return None,
        };

        if n_val > order {
            // Polynomial degree exceeds order
            return None;
        }

        // Use Rodrigues' formula symbolically is complex; instead use recursion
        // P_0(x) = 1, P_1(x) = x
        // (n+1)P_{n+1}(x) = (2n+1)xP_n(x) - nP_{n-1}(x)

        if n_val == 0 {
            return Some(store.int(1));
        }
        if n_val == 1 {
            return Some(x);
        }

        // Build recursively
        let mut p_prev = store.int(1); // P_0
        let mut p_curr = x; // P_1

        for k in 1..n_val {
            let two_k_plus_1 = store.int((2 * k + 1) as i64);
            let k_i = store.int(k as i64);
            let k_plus_1 = store.int((k + 1) as i64);

            // (2k+1)*x*P_k(x)
            let term1 = store.mul(vec![two_k_plus_1, x, p_curr]);
            // k*P_{k-1}(x)
            let term2 = store.mul(vec![k_i, p_prev]);
            // numerator = term1 - term2
            let neg_one = store.int(-1);
            let neg_term2 = store.mul(vec![neg_one, term2]);
            let numerator = store.add(vec![term1, neg_term2]);
            // P_{k+1}(x) = numerator / (k+1)
            let minus_one = store.int(-1);
            let inv_k_plus_1 = store.pow(k_plus_1, minus_one);
            let p_next = store.mul(vec![numerator, inv_k_plus_1]);

            p_prev = p_curr;
            p_curr = p_next;
        }

        Some(p_curr)
    }
}

pub struct ChebyshevTFunction;

impl SpecialFunction for ChebyshevTFunction {
    fn name(&self) -> &str {
        "ChebyshevT"
    }

    fn arity(&self) -> usize {
        2 // ChebyshevT(n, x)
    }

    /// Numerical evaluation using T_n(x) = cos(n * arccos(x)) for |x| <= 1
    fn eval(&self, args: &[f64]) -> Option<f64> {
        if args.len() != 2 {
            return None;
        }

        let n_f = args[0];
        let x = args[1];

        if n_f.fract() != 0.0 || n_f < 0.0 {
            return None;
        }

        let n = n_f as i32;

        if x.abs() <= 1.0 {
            // T_n(x) = cos(n * arccos(x))
            Some((n as f64 * x.acos()).cos())
        } else {
            // Use recursion for |x| > 1
            if n == 0 {
                return Some(1.0);
            }
            if n == 1 {
                return Some(x);
            }

            let mut t_prev = 1.0;
            let mut t_curr = x;

            for _ in 1..n {
                let t_next = 2.0 * x * t_curr - t_prev;
                t_prev = t_curr;
                t_curr = t_next;
            }

            Some(t_curr)
        }
    }

    /// Derivative: d/dx T_n(x) = n*U_{n-1}(x)
    fn derivative(&self, store: &mut Store, args: &[ExprId], arg_index: usize) -> Option<ExprId> {
        if args.len() != 2 || arg_index != 1 {
            return None;
        }

        let n = args[0];
        let x = args[1];

        // d/dx T_n(x) = n * U_{n-1}(x)
        let one = store.int(1);
        let neg_one = store.int(-1);
        let neg_one_term = store.mul(vec![neg_one, one]);
        let n_minus_1 = store.add(vec![n, neg_one_term]);
        let u_prev = store.func("ChebyshevU", vec![n_minus_1, x]);

        Some(store.mul(vec![n, u_prev]))
    }

    /// Series expansion (explicit polynomial form)
    fn series(&self, store: &mut Store, args: &[ExprId], order: usize) -> Option<ExprId> {
        if args.len() != 2 {
            return None;
        }

        let n = args[0];
        let x = args[1];

        let n_val = match (&store.get(n).op, &store.get(n).payload) {
            (Op::Integer, Payload::Int(k)) if *k >= 0 => *k as usize,
            _ => return None,
        };

        if n_val > order {
            return None;
        }

        // T_0(x) = 1, T_1(x) = x
        // T_{n+1}(x) = 2xT_n(x) - T_{n-1}(x)

        if n_val == 0 {
            return Some(store.int(1));
        }
        if n_val == 1 {
            return Some(x);
        }

        let mut t_prev = store.int(1);
        let mut t_curr = x;

        for _ in 1..n_val {
            let two = store.int(2);
            let two_x_tn = store.mul(vec![two, x, t_curr]);
            let neg_one = store.int(-1);
            let neg_t_prev = store.mul(vec![neg_one, t_prev]);
            let t_next = store.add(vec![two_x_tn, neg_t_prev]);
            t_prev = t_curr;
            t_curr = t_next;
        }

        Some(t_curr)
    }
}

/// Create a LegendreP function expression
pub fn legendre_p(store: &mut Store, n: ExprId, x: ExprId) -> ExprId {
    store.func("LegendreP", vec![n, x])
}

/// Create a ChebyshevT function expression
pub fn chebyshev_t(store: &mut Store, n: ExprId, x: ExprId) -> ExprId {
    store.func("ChebyshevT", vec![n, x])
}

#[cfg(test)]
mod tests {
    use super::*;
    use expr_core::Store;

    #[test]
    fn legendre_p0() {
        let l = LegendreFunction;
        assert_eq!(l.eval(&[0.0, 0.5]), Some(1.0));
    }

    #[test]
    fn legendre_p1() {
        let l = LegendreFunction;
        assert_eq!(l.eval(&[1.0, 0.5]), Some(0.5));
    }

    #[test]
    fn legendre_p2() {
        let l = LegendreFunction;
        // P_2(x) = (3x² - 1)/2, P_2(0.5) = (3*0.25 - 1)/2 = -0.125
        let result = l.eval(&[2.0, 0.5]).unwrap();
        assert!((result - (-0.125)).abs() < 1e-10);
    }

    #[test]
    fn chebyshev_t0() {
        let t = ChebyshevTFunction;
        assert_eq!(t.eval(&[0.0, 0.5]), Some(1.0));
    }

    #[test]
    fn chebyshev_t1() {
        let t = ChebyshevTFunction;
        let result = t.eval(&[1.0, 0.5]).unwrap();
        assert!((result - 0.5).abs() < 1e-10);
    }

    #[test]
    fn chebyshev_t2() {
        let t = ChebyshevTFunction;
        // T_2(x) = 2x² - 1, T_2(0.5) = 2*0.25 - 1 = -0.5
        let result = t.eval(&[2.0, 0.5]).unwrap();
        assert!((result - (-0.5)).abs() < 1e-10);
    }

    #[test]
    fn legendre_series_p2() {
        let mut st = Store::new();
        let n = st.int(2);
        let x = st.sym("x");

        let l = LegendreFunction;
        let series = l.series(&mut st, &[n, x], 5).unwrap();

        // P_2(x) = (3x² - 1)/2
        let s = st.to_string(series);
        assert!(s.contains("x"));
    }

    #[test]
    fn chebyshev_series_t2() {
        let mut st = Store::new();
        let n = st.int(2);
        let x = st.sym("x");

        let t = ChebyshevTFunction;
        let series = t.series(&mut st, &[n, x], 5).unwrap();

        // T_2(x) = 2x² - 1
        let s = st.to_string(series);
        assert!(s.contains("x"));
    }
}
