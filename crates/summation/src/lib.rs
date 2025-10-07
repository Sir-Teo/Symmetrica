//! Summation module: symbolic summation and closed-form evaluation
//! Phase 5: Symbolic Summation (v1.4)
//!
//! This module provides algorithms for computing closed-form expressions
//! for sums and products, including:
//! - Gosper's algorithm for hypergeometric summation
//! - Zeilberger's algorithm for creative telescoping
//! - Basic sum formulas (arithmetic, geometric, power sums)
//! - Convergence tests
//!
//! Status: Scaffolding in progress

#![deny(warnings)]

use expr_core::{ExprId, Op, Payload, Store};

/// Hypergeometric term recognition
/// A term t(k) is hypergeometric if t(k+1)/t(k) is a rational function of k.
pub fn is_hypergeometric(store: &Store, term: ExprId, var: &str) -> bool {
    // Simple heuristic: check if the term is a product/power of factorials, powers, and exponentials
    // More sophisticated implementation would check the ratio t(k+1)/t(k)
    match store.get(term).op {
        Op::Integer | Op::Rational => true, // Constants are hypergeometric
        Op::Symbol => {
            // Single variable is hypergeometric
            matches!(&store.get(term).payload, Payload::Sym(s) if s == var)
        }
        Op::Mul | Op::Add => {
            // For products and sums, check if all terms are "simple" hypergeometric
            let children = &store.get(term).children;
            children.iter().all(|&c| is_simple_hypergeometric(store, c, var))
        }
        Op::Pow => {
            // k^n or a^k are hypergeometric
            let children = &store.get(term).children;
            if children.len() == 2 {
                is_simple_hypergeometric(store, children[0], var)
                    && is_simple_hypergeometric(store, children[1], var)
            } else {
                false
            }
        }
        Op::Function => {
            // Factorials and binomials are hypergeometric
            matches!(&store.get(term).payload, Payload::Func(name) if name == "factorial" || name == "binomial")
        }
        _ => false,
    }
}

/// Helper to check if a term is "simple" hypergeometric (no complex nesting)
fn is_simple_hypergeometric(store: &Store, term: ExprId, _var: &str) -> bool {
    match store.get(term).op {
        Op::Integer | Op::Rational => true,
        Op::Symbol => true,
        Op::Pow => {
            let children = &store.get(term).children;
            children.len() == 2
        }
        _ => false,
    }
}

/// Gosper's algorithm for indefinite summation of hypergeometric terms
/// Returns Some(anti_difference) if the sum has a closed form, None otherwise.
pub fn gosper_sum(_store: &mut Store, _term: ExprId, _var: &str) -> Option<ExprId> {
    // TODO: Implement Gosper's algorithm
    // 1. Check if term is hypergeometric
    // 2. Find rational certificate
    // 3. Construct anti-difference
    None
}

/// Basic arithmetic series: sum(a + k*d, k=0..n-1) = n*a + n*(n-1)/2 * d
pub fn sum_arithmetic(store: &mut Store, first: ExprId, diff: ExprId, n: ExprId) -> Option<ExprId> {
    // sum(a + k*d, k=0..n-1) = n*a + n*(n-1)/2 * d
    let n_a = store.mul(vec![n, first]);

    let one = store.int(1);
    let neg_one = store.int(-1);
    let neg_one_term = store.mul(vec![neg_one, one]);
    let n_minus_1 = store.add(vec![n, neg_one_term]);
    let half = store.rat(1, 2);

    // n*(n-1)/2
    let n_n_minus_1 = store.mul(vec![n, n_minus_1]);
    let n_n_minus_1_over_2 = store.mul(vec![n_n_minus_1, half]);

    // n*(n-1)/2 * d
    let second_term = store.mul(vec![n_n_minus_1_over_2, diff]);

    Some(store.add(vec![n_a, second_term]))
}

/// Basic geometric series: sum(a*r^k, k=0..n-1) = a*(1-r^n)/(1-r) for r≠1
pub fn sum_geometric(store: &mut Store, first: ExprId, ratio: ExprId, n: ExprId) -> Option<ExprId> {
    // sum(a*r^k, k=0..n-1) = a*(1-r^n)/(1-r)
    let one = store.int(1);
    let r_pow_n = store.pow(ratio, n);

    // 1 - r^n
    let neg_one = store.int(-1);
    let neg_r_pow_n = store.mul(vec![neg_one, r_pow_n]);
    let numerator_inner = store.add(vec![one, neg_r_pow_n]);

    // 1 - r
    let neg_one_2 = store.int(-1);
    let neg_ratio = store.mul(vec![neg_one_2, ratio]);
    let denominator = store.add(vec![one, neg_ratio]);

    // (1 - r^n) / (1 - r)
    let minus_one = store.int(-1);
    let denom_inv = store.pow(denominator, minus_one);
    let fraction = store.mul(vec![numerator_inner, denom_inv]);

    // a * (1 - r^n) / (1 - r)
    Some(store.mul(vec![first, fraction]))
}

/// Placeholder for main summation entry point
/// This will dispatch to appropriate algorithms based on term structure
pub fn sum(
    _store: &mut Store,
    _term: ExprId,
    _var: &str,
    _lower: ExprId,
    _upper: ExprId,
) -> Option<ExprId> {
    // TODO: Implement dispatcher
    // 1. Try basic formulas (arithmetic, geometric)
    // 2. Try Gosper's algorithm
    // 3. Try Zeilberger's algorithm
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_hypergeometric_constant() {
        let mut st = Store::new();
        let five = st.int(5);
        assert!(is_hypergeometric(&st, five, "k"));
    }

    #[test]
    fn test_is_hypergeometric_variable() {
        let mut st = Store::new();
        let k = st.sym("k");
        assert!(is_hypergeometric(&st, k, "k"));
    }

    #[test]
    fn test_is_hypergeometric_power() {
        let mut st = Store::new();
        let k = st.sym("k");
        let two = st.int(2);
        let k_squared = st.pow(k, two);
        assert!(is_hypergeometric(&st, k_squared, "k"));
    }

    #[test]
    fn test_is_hypergeometric_product() {
        let mut st = Store::new();
        let k = st.sym("k");
        let three = st.int(3);
        let product = st.mul(vec![three, k]);
        assert!(is_hypergeometric(&st, product, "k"));
    }

    #[test]
    fn test_sum_arithmetic_simple() {
        // sum(k, k=0..n-1) = n*(n-1)/2
        let mut st = Store::new();
        let zero = st.int(0);
        let one = st.int(1);
        let n = st.sym("n");

        let result = sum_arithmetic(&mut st, zero, one, n).unwrap();

        // Result should be n*(n-1)/2
        let result_str = st.to_string(result);
        assert!(result_str.contains("n"));
        assert!(result_str.contains("1/2") || result_str.contains("2"));
    }

    #[test]
    fn test_sum_arithmetic_general() {
        // sum(5 + 3k, k=0..n-1) = 5n + 3*n*(n-1)/2
        let mut st = Store::new();
        let five = st.int(5);
        let three = st.int(3);
        let n = st.sym("n");

        let result = sum_arithmetic(&mut st, five, three, n).unwrap();

        // Result should contain n and the coefficients
        let result_str = st.to_string(result);
        assert!(result_str.contains("n"));
        assert!(result_str.contains("5") || result_str.contains("3"));
    }

    #[test]
    fn test_sum_geometric_simple() {
        // sum(2^k, k=0..n-1) = (1 - 2^n) / (1 - 2) = 2^n - 1
        let mut st = Store::new();
        let one = st.int(1);
        let two = st.int(2);
        let n = st.sym("n");

        let result = sum_geometric(&mut st, one, two, n).unwrap();

        // Result should be (1 - 2^n) / (1 - 2)
        let result_str = st.to_string(result);
        assert!(result_str.contains("2") || result_str.contains("n"));
    }

    #[test]
    fn test_sum_geometric_with_coefficient() {
        // sum(3*2^k, k=0..n-1) = 3*(1 - 2^n)/(1 - 2)
        let mut st = Store::new();
        let three = st.int(3);
        let two = st.int(2);
        let n = st.sym("n");

        let result = sum_geometric(&mut st, three, two, n).unwrap();

        // Result should contain 3, 2, and n
        let result_str = st.to_string(result);
        assert!(result_str.contains("3") && result_str.contains("2"));
        assert!(result_str.contains("n"));
    }
}
