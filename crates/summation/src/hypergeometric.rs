//! Hypergeometric term recognition
//!
//! A term t(k) is hypergeometric if t(k+1)/t(k) is a rational function of k.
//! This module provides functions to recognize and manipulate hypergeometric terms.

use expr_core::{ExprId, Op, Payload, Store};
use simplify::simplify;

/// Check if a term is hypergeometric
///
/// A term t(k) is hypergeometric if the ratio t(k+1)/t(k) is a rational function.
pub fn is_hypergeometric(store: &mut Store, term: ExprId, var: &str) -> bool {
    // Quick pattern: base^k where base does not depend on k
    if is_pow_const_var(store, term, var) {
        return true;
    }

    /// Detect pattern c^k where c does not depend on the summation variable
    fn is_pow_const_var(store: &Store, expr: ExprId, var: &str) -> bool {
        match store.get(expr).op {
            Op::Pow => {
                let ch = &store.get(expr).children;
                if ch.len() == 2 {
                    let base = ch[0];
                    let exp = ch[1];
                    if let (Op::Symbol, Payload::Sym(s)) =
                        (&store.get(exp).op, &store.get(exp).payload)
                    {
                        return s == var && !depends_on_var(store, base, var);
                    }
                }
                false
            }
            _ => false,
        }
    }

    // Product of hypergeometric/rational factors remains hypergeometric
    if let Op::Mul = store.get(term).op {
        let children = store.get(term).children.clone();
        let ok = children
            .into_iter()
            .all(|c| is_pow_const_var(store, c, var) || is_rational_function(store, c, var));
        if ok {
            return true;
        }
    }

    let ratio = compute_ratio(store, term, var);
    ratio.is_some() && is_rational_function(store, ratio.unwrap(), var)
}

/// Compute the ratio t(k+1)/t(k)
fn compute_ratio(store: &mut Store, term: ExprId, var: &str) -> Option<ExprId> {
    // Substitute k+1 for k
    let k = store.sym(var);
    let one = store.int(1);
    let k_plus_1 = store.add(vec![k, one]);

    let term_shifted = substitute(store, term, var, k_plus_1)?;

    // Compute term_shifted / term
    let minus_one = store.int(-1);
    let inv_term = store.pow(term, minus_one);
    let ratio = store.mul(vec![term_shifted, inv_term]);

    Some(simplify(store, ratio))
}

/// Substitute var with replacement in expr
fn substitute(store: &mut Store, expr: ExprId, var: &str, replacement: ExprId) -> Option<ExprId> {
    match (&store.get(expr).op, &store.get(expr).payload) {
        (Op::Symbol, Payload::Sym(s)) => {
            if s == var {
                Some(replacement)
            } else {
                Some(expr)
            }
        }
        (Op::Integer, _) | (Op::Rational, _) => Some(expr),
        _ => {
            let children = store.get(expr).children.clone();
            let new_children: Option<Vec<_>> =
                children.iter().map(|&c| substitute(store, c, var, replacement)).collect();

            let new_children = new_children?;
            let new_expr = match store.get(expr).op {
                Op::Add => store.add(new_children),
                Op::Mul => store.mul(new_children),
                Op::Pow => {
                    if new_children.len() == 2 {
                        store.pow(new_children[0], new_children[1])
                    } else {
                        return None;
                    }
                }
                Op::Function => {
                    let func_name = if let Payload::Func(ref name) = store.get(expr).payload {
                        name.clone()
                    } else {
                        return None;
                    };
                    store.func(&func_name, new_children)
                }
                _ => return None,
            };

            Some(simplify(store, new_expr))
        }
    }
}

/// Check if expression is a rational function of var
fn is_rational_function(store: &Store, expr: ExprId, var: &str) -> bool {
    match &store.get(expr).op {
        Op::Integer | Op::Rational => true,
        Op::Symbol => {
            if let Payload::Sym(_s) = &store.get(expr).payload {
                // Variable is a rational function (polynomial of degree 1)
                true
            } else {
                // Other symbols are treated as constants
                true
            }
        }
        Op::Add | Op::Mul => {
            // All children must be rational functions
            store.get(expr).children.iter().all(|&c| is_rational_function(store, c, var))
        }
        Op::Pow => {
            let children = &store.get(expr).children;
            if children.len() == 2 {
                let base = children[0];
                let exponent = children[1];

                // Base must be rational function, exponent must be constant or integer
                is_rational_function(store, base, var) && !depends_on_var(store, exponent, var)
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Check if expression depends on variable
fn depends_on_var(store: &Store, expr: ExprId, var: &str) -> bool {
    match (&store.get(expr).op, &store.get(expr).payload) {
        (Op::Symbol, Payload::Sym(s)) => s == var,
        (Op::Integer, _) | (Op::Rational, _) => false,
        _ => store.get(expr).children.iter().any(|&c| depends_on_var(store, c, var)),
    }
}

/// Rationalize a hypergeometric term by computing numerator and denominator polynomials
///
/// Given t(k+1)/t(k) = p(k)/q(k), extract p and q as polynomials.
pub fn rationalize_hypergeometric(
    store: &mut Store,
    term: ExprId,
    var: &str,
) -> Option<(ExprId, ExprId)> {
    let ratio = compute_ratio(store, term, var)?;

    // Try to split ratio into numerator/denominator
    split_rational(store, ratio)
}

/// Split a rational expression into numerator and denominator
fn split_rational(store: &mut Store, expr: ExprId) -> Option<(ExprId, ExprId)> {
    match &store.get(expr).op {
        Op::Mul => {
            let children = &store.get(expr).children;
            let mut num_parts = Vec::new();
            let mut denom_parts = Vec::new();

            for &child in children {
                if let Op::Pow = store.get(child).op {
                    let pow_children = &store.get(child).children;
                    if pow_children.len() == 2 {
                        let base = pow_children[0];
                        let exp = pow_children[1];

                        // Check if exponent is negative
                        if let (Op::Integer, Payload::Int(n)) =
                            (&store.get(exp).op, &store.get(exp).payload)
                        {
                            if *n < 0 {
                                denom_parts.push(base);
                                continue;
                            }
                        }
                    }
                }
                num_parts.push(child);
            }

            let numerator = if num_parts.is_empty() { store.int(1) } else { store.mul(num_parts) };

            let denominator =
                if denom_parts.is_empty() { store.int(1) } else { store.mul(denom_parts) };

            Some((numerator, denominator))
        }
        Op::Pow => {
            let children = &store.get(expr).children;
            if children.len() == 2 {
                let base = children[0];
                let exp = children[1];

                if let (Op::Integer, Payload::Int(n)) =
                    (&store.get(exp).op, &store.get(exp).payload)
                {
                    if *n < 0 {
                        return Some((store.int(1), base));
                    }
                }
            }
            Some((expr, store.int(1)))
        }
        _ => Some((expr, store.int(1))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_hypergeometric_factorial() {
        let mut st = Store::new();
        let k = st.sym("k");

        // Term: k! (represented as a product, but we'll use k as a simple test)
        // The ratio (k+1)!/k! = k+1 is rational

        // For testing, use k itself (ratio (k+1)/k is rational)
        assert!(is_hypergeometric(&mut st, k, "k"));
    }

    #[test]
    fn test_substitute_simple() {
        let mut st = Store::new();
        let k = st.sym("k");
        let two = st.int(2);

        // k + 2
        let expr = st.add(vec![k, two]);

        // Substitute k with 5
        let five = st.int(5);
        let result = substitute(&mut st, expr, "k", five).expect("substitution");

        // Should be 5 + 2 = 7
        let result_str = st.to_string(result);
        assert!(result_str.contains("7") || result_str.contains("5"));
    }
}
