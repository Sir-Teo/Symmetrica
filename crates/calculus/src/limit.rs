//! Limit evaluation for symbolic expressions
//!
//! Provides utilities to compute limits of expressions as variables approach
//! specific values or infinity. Essential for improper integrals and asymptotic analysis.

use crate::evaluate::try_eval_constant;
use expr_core::{ExprId, Op, Payload, Store};

/// Point at which to evaluate a limit
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LimitPoint {
    /// Limit as x → a for finite a
    Finite(i64),
    /// Limit as x → +∞
    PositiveInfinity,
    /// Limit as x → -∞
    NegativeInfinity,
}

/// Result of a limit computation
#[derive(Debug, Clone, PartialEq)]
pub enum LimitResult {
    /// Limit exists and equals a finite rational value
    Finite((i64, i64)),
    /// Limit is +∞
    PositiveInfinity,
    /// Limit is -∞
    NegativeInfinity,
    /// Limit does not exist or cannot be determined
    Undefined,
}

/// Computes the limit of an expression as var approaches a point
///
/// Uses algebraic techniques for polynomial and rational expressions.
/// For transcendental functions, uses known limit rules.
///
/// # Examples
/// - lim[x→∞] 1/x = 0
/// - lim[x→∞] x² = ∞
/// - lim[x→0] sin(x)/x = 1 (requires L'Hôpital's rule, future)
pub fn limit(store: &Store, expr: ExprId, var: &str, point: LimitPoint) -> LimitResult {
    // Try to evaluate as constant first (if no dependence on var)
    if let Some(val) = try_eval_constant(store, expr) {
        return LimitResult::Finite(val);
    }

    match point {
        LimitPoint::Finite(a) => limit_at_finite(store, expr, var, a),
        LimitPoint::PositiveInfinity => limit_at_infinity(store, expr, var, true),
        LimitPoint::NegativeInfinity => limit_at_infinity(store, expr, var, false),
    }
}

/// Computes limit as var → a for finite a
fn limit_at_finite(store: &Store, expr: ExprId, _var: &str, _a: i64) -> LimitResult {
    // For now, simple evaluation by substitution
    // Full implementation would need L'Hôpital's rule for indeterminate forms

    // If expression is constant, return it
    if let Some(val) = try_eval_constant(store, expr) {
        return LimitResult::Finite(val);
    }

    // Otherwise, return Undefined (requires substitution + indeterminate form handling)
    LimitResult::Undefined
}

/// Computes limit as var → ±∞
fn limit_at_infinity(store: &Store, expr: ExprId, var: &str, positive: bool) -> LimitResult {
    match &store.get(expr).op {
        Op::Integer | Op::Rational => {
            // Constants have finite limits
            if let Some(val) = try_eval_constant(store, expr) {
                LimitResult::Finite(val)
            } else {
                LimitResult::Undefined
            }
        }
        Op::Symbol => {
            if let Payload::Sym(s) = &store.get(expr).payload {
                if s == var {
                    // lim[x→±∞] x = ±∞
                    if positive {
                        LimitResult::PositiveInfinity
                    } else {
                        LimitResult::NegativeInfinity
                    }
                } else {
                    // Other variables are treated as constants
                    LimitResult::Undefined
                }
            } else {
                LimitResult::Undefined
            }
        }
        Op::Add => {
            // For sums, limit is sum of limits (when they exist)
            let children = &store.get(expr).children;
            let mut result = LimitResult::Finite((0, 1));

            for &child in children {
                let child_limit = limit_at_infinity(store, child, var, positive);
                result = add_limits(result, child_limit);
                if matches!(result, LimitResult::Undefined) {
                    return LimitResult::Undefined;
                }
            }
            result
        }
        Op::Mul => {
            // For products, limit is product of limits
            let children = &store.get(expr).children;
            let mut result = LimitResult::Finite((1, 1));

            for &child in children {
                let child_limit = limit_at_infinity(store, child, var, positive);
                result = mul_limits(result, child_limit);
                if matches!(result, LimitResult::Undefined) {
                    return LimitResult::Undefined;
                }
            }
            result
        }
        Op::Pow => {
            let children = &store.get(expr).children;
            if children.len() != 2 {
                return LimitResult::Undefined;
            }

            let base_limit = limit_at_infinity(store, children[0], var, positive);
            let exp_limit = limit_at_infinity(store, children[1], var, positive);

            pow_limit(base_limit, exp_limit)
        }
        Op::Function => {
            // Handle common transcendental functions
            if let Payload::Func(fname) = &store.get(expr).payload {
                if store.get(expr).children.len() != 1 {
                    return LimitResult::Undefined;
                }
                let arg = store.get(expr).children[0];
                let arg_limit = limit_at_infinity(store, arg, var, positive);

                match fname.as_str() {
                    "exp" => {
                        // lim[x→∞] e^x = ∞, lim[x→-∞] e^x = 0
                        match arg_limit {
                            LimitResult::PositiveInfinity => LimitResult::PositiveInfinity,
                            LimitResult::NegativeInfinity => LimitResult::Finite((0, 1)),
                            LimitResult::Finite(_val) => {
                                // e^finite = finite (but we can't compute it symbolically)
                                LimitResult::Undefined
                            }
                            LimitResult::Undefined => LimitResult::Undefined,
                        }
                    }
                    "ln" | "log" => {
                        // lim[x→∞] ln(x) = ∞, lim[x→0⁺] ln(x) = -∞
                        match arg_limit {
                            LimitResult::PositiveInfinity => LimitResult::PositiveInfinity,
                            LimitResult::Finite((0, _)) => LimitResult::NegativeInfinity,
                            _ => LimitResult::Undefined,
                        }
                    }
                    "sin" | "cos" => {
                        // Oscillating functions have no limit at infinity
                        match arg_limit {
                            LimitResult::PositiveInfinity | LimitResult::NegativeInfinity => {
                                LimitResult::Undefined
                            }
                            _ => LimitResult::Undefined,
                        }
                    }
                    _ => LimitResult::Undefined,
                }
            } else {
                LimitResult::Undefined
            }
        }
        _ => LimitResult::Undefined,
    }
}

/// Adds two limit results
fn add_limits(a: LimitResult, b: LimitResult) -> LimitResult {
    use arith::q_add;
    match (a, b) {
        (LimitResult::Finite(v1), LimitResult::Finite(v2)) => LimitResult::Finite(q_add(v1, v2)),
        (LimitResult::PositiveInfinity, LimitResult::PositiveInfinity) => {
            LimitResult::PositiveInfinity
        }
        (LimitResult::NegativeInfinity, LimitResult::NegativeInfinity) => {
            LimitResult::NegativeInfinity
        }
        (LimitResult::PositiveInfinity, LimitResult::Finite(_))
        | (LimitResult::Finite(_), LimitResult::PositiveInfinity) => LimitResult::PositiveInfinity,
        (LimitResult::NegativeInfinity, LimitResult::Finite(_))
        | (LimitResult::Finite(_), LimitResult::NegativeInfinity) => LimitResult::NegativeInfinity,
        // ∞ - ∞ is undefined
        (LimitResult::PositiveInfinity, LimitResult::NegativeInfinity)
        | (LimitResult::NegativeInfinity, LimitResult::PositiveInfinity) => LimitResult::Undefined,
        _ => LimitResult::Undefined,
    }
}

/// Multiplies two limit results
fn mul_limits(a: LimitResult, b: LimitResult) -> LimitResult {
    use arith::q_mul;
    match (a, b) {
        (LimitResult::Finite(v1), LimitResult::Finite(v2)) => LimitResult::Finite(q_mul(v1, v2)),
        (LimitResult::PositiveInfinity, LimitResult::PositiveInfinity) => {
            LimitResult::PositiveInfinity
        }
        (LimitResult::NegativeInfinity, LimitResult::NegativeInfinity) => {
            LimitResult::PositiveInfinity
        }
        (LimitResult::PositiveInfinity, LimitResult::NegativeInfinity)
        | (LimitResult::NegativeInfinity, LimitResult::PositiveInfinity) => {
            LimitResult::NegativeInfinity
        }
        (LimitResult::PositiveInfinity, LimitResult::Finite((n, _)))
        | (LimitResult::Finite((n, _)), LimitResult::PositiveInfinity) => {
            if n > 0 {
                LimitResult::PositiveInfinity
            } else if n < 0 {
                LimitResult::NegativeInfinity
            } else {
                LimitResult::Undefined // 0 * ∞
            }
        }
        (LimitResult::NegativeInfinity, LimitResult::Finite((n, _)))
        | (LimitResult::Finite((n, _)), LimitResult::NegativeInfinity) => {
            if n > 0 {
                LimitResult::NegativeInfinity
            } else if n < 0 {
                LimitResult::PositiveInfinity
            } else {
                LimitResult::Undefined // 0 * ∞
            }
        }
        _ => LimitResult::Undefined,
    }
}

/// Computes limit of base^exp given limits of base and exp
fn pow_limit(base: LimitResult, exp: LimitResult) -> LimitResult {
    match (base, exp) {
        (LimitResult::Finite((b_n, b_d)), LimitResult::Finite((e_n, e_d))) => {
            // Both finite - would need numerical evaluation
            // For now, only handle integer exponents
            if e_d == 1 && e_n >= 0 && e_n <= 10 {
                // Small positive integer exponent
                let mut result = (1i64, 1i64);
                for _ in 0..e_n {
                    result = arith::q_mul(result, (b_n, b_d));
                }
                LimitResult::Finite(result)
            } else {
                LimitResult::Undefined
            }
        }
        (LimitResult::PositiveInfinity, LimitResult::PositiveInfinity) => {
            LimitResult::PositiveInfinity
        }
        (LimitResult::Finite((n, _)), LimitResult::PositiveInfinity) if n.abs() > 1 => {
            // |base| > 1, exp → ∞
            if n > 0 {
                LimitResult::PositiveInfinity
            } else {
                // Oscillates between +∞ and -∞
                LimitResult::Undefined
            }
        }
        (LimitResult::Finite((n, d)), LimitResult::PositiveInfinity) if n.abs() < d.abs() => {
            // |base| < 1, exp → ∞ → 0
            LimitResult::Finite((0, 1))
        }
        (LimitResult::PositiveInfinity, LimitResult::Finite((n, _))) if n > 0 => {
            LimitResult::PositiveInfinity
        }
        (LimitResult::PositiveInfinity, LimitResult::Finite((n, _))) if n < 0 => {
            LimitResult::Finite((0, 1))
        }
        _ => LimitResult::Undefined,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limit_constant() {
        let mut st = Store::new();
        let five = st.int(5);

        let result = limit(&st, five, "x", LimitPoint::PositiveInfinity);
        assert_eq!(result, LimitResult::Finite((5, 1)));
    }

    #[test]
    fn test_limit_variable_at_infinity() {
        let mut st = Store::new();
        let x = st.sym("x");

        let result = limit(&st, x, "x", LimitPoint::PositiveInfinity);
        assert_eq!(result, LimitResult::PositiveInfinity);

        let result_neg = limit(&st, x, "x", LimitPoint::NegativeInfinity);
        assert_eq!(result_neg, LimitResult::NegativeInfinity);
    }

    #[test]
    fn test_limit_reciprocal() {
        let mut st = Store::new();
        let x = st.sym("x");
        let neg_one = st.int(-1);
        let inv_x = st.pow(x, neg_one);

        // lim[x→∞] 1/x = 0
        let result = limit(&st, inv_x, "x", LimitPoint::PositiveInfinity);
        assert_eq!(result, LimitResult::Finite((0, 1)));
    }

    #[test]
    fn test_limit_polynomial() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x_squared = st.pow(x, two);

        // lim[x→∞] x² = ∞
        let result = limit(&st, x_squared, "x", LimitPoint::PositiveInfinity);
        assert_eq!(result, LimitResult::PositiveInfinity);
    }

    #[test]
    fn test_limit_exponential() {
        let mut st = Store::new();
        let x = st.sym("x");
        let exp_x = st.func("exp", vec![x]);

        // lim[x→∞] e^x = ∞
        let result = limit(&st, exp_x, "x", LimitPoint::PositiveInfinity);
        assert_eq!(result, LimitResult::PositiveInfinity);

        // lim[x→-∞] e^x = 0
        let result_neg = limit(&st, exp_x, "x", LimitPoint::NegativeInfinity);
        assert_eq!(result_neg, LimitResult::Finite((0, 1)));
    }

    #[test]
    fn test_limit_sum() {
        let mut st = Store::new();
        let x = st.sym("x");
        let five = st.int(5);
        let expr = st.add(vec![x, five]);

        // lim[x→∞] (x + 5) = ∞
        let result = limit(&st, expr, "x", LimitPoint::PositiveInfinity);
        assert_eq!(result, LimitResult::PositiveInfinity);
    }
}
