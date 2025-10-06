//! Infinite Products and Product Evaluation
//!
//! This module provides functions for evaluating infinite products
//! and product expressions symbolically.
//!
//! # Examples
//!
//! ```
//! use summation::evaluate_finite_product;
//! use expr_core::Store;
//!
//! let mut st = Store::new();
//! let k = st.sym("k");
//! let n = st.sym("n");
//!
//! // Product notation: ∏(k=1 to n) k = n!
//! ```

use expr_core::{ExprId, Op, Payload, Store};
use simplify::simplify;

/// Evaluate a finite product: ∏(var=lower..upper) expr
///
/// Returns the closed form if available, otherwise returns the product expression.
pub fn evaluate_finite_product(
    store: &mut Store,
    expr: ExprId,
    _var: &str,
    lower: ExprId,
    upper: ExprId,
) -> Option<ExprId> {
    // Try to recognize common product patterns

    // Pattern 1: ∏(k=1 to n) k = n!
    if is_simple_variable(store, expr, _var) {
        return try_factorial_product(store, lower, upper);
    }

    // Pattern 2: ∏(k=1 to n) (a + k*d) - arithmetic sequence product
    if let Some((a, d_opt)) = extract_linear_term(store, expr, _var) {
        let d = d_opt.unwrap_or_else(|| store.int(1));
        return try_pochhammer_product(store, a, d, lower, upper);
    }

    // Pattern 3: ∏(k=0 to n) r^k = r^(n(n+1)/2) for constant r
    if let Some(base) = extract_power_term(store, expr, _var) {
        return try_geometric_product(store, base, lower, upper);
    }

    None
}

/// Check if expression is just the variable itself
fn is_simple_variable(store: &Store, expr: ExprId, var: &str) -> bool {
    matches!((&store.get(expr).op, &store.get(expr).payload), 
             (Op::Symbol, Payload::Sym(s)) if s == var)
}

/// Try to compute factorial: ∏(k=1 to n) k = n!
fn try_factorial_product(store: &mut Store, lower: ExprId, upper: ExprId) -> Option<ExprId> {
    // Check if lower bound is 1
    if !matches!((&store.get(lower).op, &store.get(lower).payload), (Op::Integer, Payload::Int(1)))
    {
        return None;
    }

    // For factorial, we need symbolic representation
    // In a full implementation, this would create a factorial function
    // For now, return None to indicate we can't simplify further

    // If upper is a concrete small integer, compute directly
    if let (Op::Integer, Payload::Int(n)) = (&store.get(upper).op, &store.get(upper).payload) {
        if *n >= 1 && *n <= 20 {
            let mut result = 1i64;
            for k in 1..=*n {
                result = result.saturating_mul(k);
                if result < 0 {
                    // Overflow
                    return None;
                }
            }
            return Some(store.int(result));
        }
    }

    None
}

/// Extract linear term: a + k*d from expression
/// Returns (a, d, is_k_simple) where is_k_simple indicates if it's just k (true) or d*k (false)
fn extract_linear_term(store: &Store, expr: ExprId, var: &str) -> Option<(ExprId, Option<ExprId>)> {
    if store.get(expr).op != Op::Add {
        return None;
    }

    let children = &store.get(expr).children;
    if children.len() != 2 {
        return None;
    }

    // Try to find a constant term and a k*d term
    for i in 0..2 {
        let j = 1 - i;
        let potential_const = children[i];
        let potential_linear = children[j];

        // Check if potential_const doesn't depend on var
        if depends_on_var(store, potential_const, var) {
            continue;
        }

        // Check if potential_linear is k or d*k
        if is_simple_variable(store, potential_linear, var) {
            // Simple k case, d = 1 (will be created by caller)
            return Some((potential_const, None));
        }

        // Check for d*k pattern
        if store.get(potential_linear).op == Op::Mul {
            let mul_children = &store.get(potential_linear).children;
            if mul_children.len() == 2 {
                for &mc in mul_children {
                    if is_simple_variable(store, mc, var) {
                        let other = mul_children.iter().find(|&&c| c != mc).copied()?;
                        if !depends_on_var(store, other, var) {
                            return Some((potential_const, Some(other)));
                        }
                    }
                }
            }
        }
    }

    None
}

/// Check if expression depends on a variable
fn depends_on_var(store: &Store, expr: ExprId, var: &str) -> bool {
    match &store.get(expr).op {
        Op::Symbol => {
            if let Payload::Sym(s) = &store.get(expr).payload {
                return s == var;
            }
            false
        }
        Op::Integer | Op::Rational => false,
        _ => {
            let children = &store.get(expr).children;
            children.iter().any(|&c| depends_on_var(store, c, var))
        }
    }
}

/// Try Pochhammer-style product: ∏(k=1 to n) (a + k*d)
fn try_pochhammer_product(
    _store: &mut Store,
    _a: ExprId,
    _d: ExprId,
    _lower: ExprId,
    _upper: ExprId,
) -> Option<ExprId> {
    // This would use the Pochhammer symbol or Gamma function
    // For now, return None (not implemented)
    None
}

/// Extract power term: r^k from expression
fn extract_power_term(store: &Store, expr: ExprId, var: &str) -> Option<ExprId> {
    if store.get(expr).op != Op::Pow {
        return None;
    }

    let children = &store.get(expr).children;
    if children.len() != 2 {
        return None;
    }

    let base = children[0];
    let exp = children[1];

    // Check if exponent is the variable
    if is_simple_variable(store, exp, var) && !depends_on_var(store, base, var) {
        return Some(base);
    }

    None
}

/// Try geometric product: ∏(k=0 to n) r^k = r^(n(n+1)/2)
fn try_geometric_product(
    store: &mut Store,
    base: ExprId,
    lower: ExprId,
    upper: ExprId,
) -> Option<ExprId> {
    // Check if lower bound is 0
    if !matches!((&store.get(lower).op, &store.get(lower).payload), (Op::Integer, Payload::Int(0)))
    {
        return None;
    }

    // Exponent is sum of 0+1+2+...+n = n(n+1)/2
    let one = store.int(1);
    let n_plus_1 = store.add(vec![upper, one]);
    let n_times_n_plus_1 = store.mul(vec![upper, n_plus_1]);
    let half = store.rat(1, 2);
    let exponent = store.mul(vec![half, n_times_n_plus_1]);

    let result = store.pow(base, exponent);
    Some(simplify(store, result))
}

/// Connection to Gamma function: ∏(k=0 to n-1) (x+k) = Γ(x+n)/Γ(x)
///
/// This is the Pochhammer symbol representation.
pub fn product_to_gamma_ratio(store: &mut Store, x: ExprId, n: ExprId) -> Option<ExprId> {
    // Create symbolic representation: gamma(x+n) / gamma(x)
    let x_plus_n = store.add(vec![x, n]);
    let gamma_xn = store.func("gamma", vec![x_plus_n]);
    let gamma_x = store.func("gamma", vec![x]);
    let neg_one = store.int(-1);
    let inv_gamma_x = store.pow(gamma_x, neg_one);

    Some(store.mul(vec![gamma_xn, inv_gamma_x]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factorial_small() {
        let mut st = Store::new();
        let k = st.sym("k");
        let one = st.int(1);
        let five = st.int(5);

        // ∏(k=1 to 5) k = 120
        let result = evaluate_finite_product(&mut st, k, "k", one, five);

        assert!(result.is_some());
        if let Some(r) = result {
            assert!(matches!(
                (&st.get(r).op, &st.get(r).payload),
                (Op::Integer, Payload::Int(120))
            ));
        }
    }

    #[test]
    fn test_geometric_product() {
        let mut st = Store::new();
        let k = st.sym("k");
        let n = st.sym("n");
        let _two = st.int(2);
        let zero = st.int(0);
        let two_base = st.int(2);

        // ∏(k=0 to n) 2^k = 2^(n(n+1)/2)
        let term = st.pow(two_base, k);
        let result = evaluate_finite_product(&mut st, term, "k", zero, n);

        assert!(result.is_some());
        if let Some(r) = result {
            assert_eq!(st.get(r).op, Op::Pow);
        }
    }

    #[test]
    fn test_product_to_gamma() {
        let mut st = Store::new();
        let x = st.sym("x");
        let n = st.sym("n");

        let result = product_to_gamma_ratio(&mut st, x, n);

        assert!(result.is_some());
        if let Some(r) = result {
            // Should be gamma(x+n) / gamma(x)
            assert_eq!(st.get(r).op, Op::Mul);
        }
    }
}
