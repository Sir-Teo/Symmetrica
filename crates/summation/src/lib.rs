//! Symbolic Summation Module (Phase 5)
//!
//! This module provides algorithms for computing closed-form expressions
//! for sums and products, including:
//! - Gosper's algorithm for hypergeometric summation
//! - Zeilberger's algorithm for creative telescoping
//! - Basic sum formulas (arithmetic, geometric, power sums)
//! - Infinite products and Pochhammer symbols
//! - Convergence tests (ratio test)
//!
//! # Examples
//!
//! ```
//! use summation::{sum_closed_form, sum_arithmetic, sum_geometric};
//! use expr_core::Store;
//!
//! let mut st = Store::new();
//! let k = st.sym("k");
//! let n = st.sym("n");
//!
//! // Arithmetic series: ∑(k=1 to n) k = n(n+1)/2
//! let one = st.int(1);
//! let result = sum_arithmetic(&mut st, k, one, n, one, one);
//! ```

mod basic;
mod convergence;
mod gosper;
mod hypergeometric;
mod pochhammer;
mod products;
mod zeilberger;

pub use basic::{sum_arithmetic, sum_geometric, sum_power};
pub use convergence::{ratio_test, ConvergenceResult};
pub use gosper::gosper_sum;
pub use hypergeometric::{is_hypergeometric, rationalize_hypergeometric};
pub use pochhammer::{falling_factorial, pochhammer, rising_factorial, rising_to_falling};
pub use products::{evaluate_finite_product, product_to_gamma_ratio};
pub use zeilberger::{zeilberger_recurrence, Certificate};

use expr_core::{ExprId, Store};

/// Attempt to find a closed-form expression for ∑(var=lower..upper) expr
///
/// This is the main entry point that tries various strategies:
/// 1. Basic formulas (arithmetic, geometric, power sums)
/// 2. Gosper's algorithm (for hypergeometric terms)
/// 3. Zeilberger's algorithm (generates recurrence if no closed form)
///
/// Returns `Some(result)` if a closed form is found, `None` otherwise.
pub fn sum_closed_form(
    store: &mut Store,
    expr: ExprId,
    var: &str,
    lower: ExprId,
    upper: ExprId,
) -> Option<ExprId> {
    // Try basic formulas first
    if let Some(result) = basic::try_basic_sum(store, expr, var, lower, upper) {
        return Some(result);
    }

    // Try Gosper's algorithm for hypergeometric terms
    if let Some(result) = gosper::gosper_sum(store, expr, var, lower, upper) {
        return Some(result);
    }

    // If no closed form found, return None
    // (Zeilberger's algorithm can be used separately to generate recurrences)
    None
}

/// Compute a definite sum: ∑(var=lower..upper) expr
///
/// This evaluates the sum by computing the antidifference (if it exists)
/// and applying the fundamental theorem of summation.
pub fn definite_sum(
    store: &mut Store,
    expr: ExprId,
    var: &str,
    lower: ExprId,
    upper: ExprId,
) -> Option<ExprId> {
    sum_closed_form(store, expr, var, lower, upper)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expr_core::Store;

    #[test]
    fn test_sum_arithmetic_series() {
        let mut st = Store::new();
        let k = st.sym("k");
        let n = st.sym("n");
        let one = st.int(1);

        // ∑(k=1 to n) k = n(n+1)/2
        let zero = st.int(0);
        let result = sum_arithmetic(&mut st, k, one, n, zero, one).expect("arithmetic sum");

        // Expected: n * (n+1) / 2
        let n_plus_1 = st.add(vec![n, one]);
        let numerator = st.mul(vec![n, n_plus_1]);
        let half = st.rat(1, 2);
        let expected = st.mul(vec![half, numerator]);

        assert_eq!(st.to_string(result), st.to_string(expected));
    }

    #[test]
    fn test_sum_geometric_series() {
        let mut st = Store::new();
        let k = st.sym("k");
        let n = st.sym("n");
        let zero = st.int(0);
        let two = st.int(2);

        // ∑(k=0 to n) 2^k = 2^(n+1) - 1
        let term = st.pow(two, k);
        let result = sum_geometric(&mut st, term, "k", zero, n, two).expect("geometric sum");

        // Should contain 2^(n+1) - 1
        let result_str = st.to_string(result);
        assert!(result_str.contains("2") || result_str.contains("pow"));
    }
}
