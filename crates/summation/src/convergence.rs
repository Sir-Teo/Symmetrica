//! Convergence tests for infinite series in the summation crate
//!
//! Provides simple ratio test for series of hypergeometric type.

use crate::hypergeometric::rationalize_hypergeometric;
use expr_core::{ExprId, Store};
use polys::expr_to_unipoly;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvergenceResult {
    Convergent,
    Divergent,
    Inconclusive,
}
/// Ratio test using lim_{k->∞} |a_{k+1}/a_k| for hypergeometric terms a_k
///
/// If limit L < 1 -> Convergent
/// If L > 1 -> Divergent
/// If L == 1 or cannot determine -> Inconclusive
pub fn ratio_test(store: &mut Store, term: ExprId, var: &str) -> Option<ConvergenceResult> {
    // First try general hypergeometric rationalization
    if let Some((p, q)) = rationalize_hypergeometric(store, term, var) {
        if let (Some(p_poly), Some(q_poly)) =
            (expr_to_unipoly(store, p, var), expr_to_unipoly(store, q, var))
        {
            let dp = p_poly.degree().unwrap_or(0);
            let dq = q_poly.degree().unwrap_or(0);

            if dp < dq {
                return Some(ConvergenceResult::Convergent);
            }
            if dp > dq {
                return Some(ConvergenceResult::Divergent);
            }
            let lp = p_poly.leading_coeff();
            let lq = q_poly.leading_coeff();
            if lq.is_zero() {
                return None;
            }
            let ap = (lp.0 as i128).abs() * (lq.1 as i128);
            let aq = (lq.0 as i128).abs() * (lp.1 as i128);
            return if ap < aq {
                Some(ConvergenceResult::Convergent)
            } else if ap > aq {
                Some(ConvergenceResult::Divergent)
            } else {
                Some(ConvergenceResult::Inconclusive)
            };
        }
        // Fallback to pow-case on the original term if p/q not convertible to polynomials
        // (e.g., geometric series ratio)
        use expr_core::{Op, Payload};
        if let Op::Pow = store.get(term).op {
            let ch = store.get(term).children.clone();
            if ch.len() == 2 {
                let base = ch[0];
                let exp = ch[1];
                if let (Op::Symbol, Payload::Sym(s)) = (&store.get(exp).op, &store.get(exp).payload)
                {
                    if s == var {
                        match (&store.get(base).op, &store.get(base).payload) {
                            (Op::Integer, Payload::Int(n)) => {
                                let an = (*n as i128).abs();
                                if an < 1 {
                                    return Some(ConvergenceResult::Convergent);
                                }
                                if an > 1 {
                                    return Some(ConvergenceResult::Divergent);
                                }
                                return Some(ConvergenceResult::Inconclusive);
                            }
                            (Op::Rational, Payload::Rat(n, d)) => {
                                let an = (*n as i128).abs();
                                let ad = (*d as i128).abs().max(1);
                                if an < ad {
                                    return Some(ConvergenceResult::Convergent);
                                }
                                if an > ad {
                                    return Some(ConvergenceResult::Divergent);
                                }
                                return Some(ConvergenceResult::Inconclusive);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        return None;
    }

    // Fallback: detect pure geometric term r^k with base independent of k
    use expr_core::{Op, Payload};
    if let Op::Pow = store.get(term).op {
        let ch = store.get(term).children.clone();
        if ch.len() == 2 {
            let base = ch[0];
            let exp = ch[1];
            if let (Op::Symbol, Payload::Sym(s)) = (&store.get(exp).op, &store.get(exp).payload) {
                if s == var {
                    // Evaluate |base| comparison with 1 if base is rational/integer
                    match (&store.get(base).op, &store.get(base).payload) {
                        (Op::Integer, Payload::Int(n)) => {
                            let an = (*n as i128).abs();
                            if an < 1 {
                                return Some(ConvergenceResult::Convergent);
                            }
                            if an > 1 {
                                return Some(ConvergenceResult::Divergent);
                            }
                            return Some(ConvergenceResult::Inconclusive);
                        }
                        (Op::Rational, Payload::Rat(n, d)) => {
                            let an = (*n as i128).abs();
                            let ad = (*d as i128).abs().max(1);
                            if an < ad {
                                return Some(ConvergenceResult::Convergent);
                            }
                            if an > ad {
                                return Some(ConvergenceResult::Divergent);
                            }
                            return Some(ConvergenceResult::Inconclusive);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    // Otherwise, unable to determine
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ratio_test_convergent_geo() {
        let mut st = Store::new();
        let k = st.sym("k");
        let half = st.rat(1, 2);
        let term = st.pow(half, k); // (1/2)^k
        let res = ratio_test(&mut st, term, "k").expect("ratio");
        assert_eq!(res, ConvergenceResult::Convergent);
    }

    #[test]
    fn test_ratio_test_divergent_geo() {
        let mut st = Store::new();
        let k = st.sym("k");
        let two = st.int(2);
        let term = st.pow(two, k); // 2^k
        let res = ratio_test(&mut st, term, "k").expect("ratio");
        assert_eq!(res, ConvergenceResult::Divergent);
    }

    #[test]
    fn test_ratio_test_inconclusive_geo() {
        let mut st = Store::new();
        let k = st.sym("k");
        let one = st.int(1);
        let term = st.pow(one, k); // 1^k
        let res = ratio_test(&mut st, term, "k").expect("ratio");
        assert_eq!(res, ConvergenceResult::Inconclusive);
    }
}
