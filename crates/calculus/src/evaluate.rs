//! Constant evaluation and folding for symbolic expressions
//!
//! Provides utilities to evaluate expressions involving only constants
//! to their concrete numeric values. This is essential for definite
//! integrals and numerical computations.

use arith::{q_add, q_mul};
use expr_core::{ExprId, Op, Payload, Store};

/// Attempts to evaluate an expression to a rational constant
///
/// Returns Some((numerator, denominator)) if the expression evaluates
/// to a concrete rational value, None otherwise.
///
/// # Examples
/// - 2 + 3 → (5, 1)
/// - 1/2 * 4 → (2, 1)
/// - 3² → (9, 1)
/// - x + 1 → None (contains variable)
pub fn try_eval_constant(store: &Store, expr: ExprId) -> Option<(i64, i64)> {
    match (&store.get(expr).op, &store.get(expr).payload) {
        (Op::Integer, Payload::Int(n)) => Some((*n, 1)),
        (Op::Rational, Payload::Rat(p, q)) => Some((*p, *q)),
        (Op::Symbol, _) => None, // Variables cannot be evaluated
        (Op::Add, _) => {
            let children = &store.get(expr).children;
            let mut sum = (0i64, 1i64);
            for &child in children {
                let val = try_eval_constant(store, child)?;
                sum = q_add(sum, val);
            }
            Some(sum)
        }
        (Op::Mul, _) => {
            let children = &store.get(expr).children;
            let mut product = (1i64, 1i64);
            for &child in children {
                let val = try_eval_constant(store, child)?;
                product = q_mul(product, val);
            }
            Some(product)
        }
        (Op::Pow, _) => {
            let children = &store.get(expr).children;
            if children.len() != 2 {
                return None;
            }
            let base = try_eval_constant(store, children[0])?;
            let exp = try_eval_constant(store, children[1])?;

            // Only handle integer exponents for now
            if exp.1 != 1 {
                return None;
            }

            eval_pow(base, exp.0)
        }
        (Op::Function, _) => {
            // Functions require more complex evaluation (sin, cos, exp, etc.)
            // For now, return None
            None
        }
        _ => None,
    }
}

/// Evaluates base^exp for rational base and integer exponent
fn eval_pow(base: (i64, i64), exp: i64) -> Option<(i64, i64)> {
    if exp == 0 {
        return Some((1, 1));
    }

    if exp < 0 {
        // base^(-n) = 1 / base^n
        let pos_pow = eval_pow(base, -exp)?;
        return Some((pos_pow.1, pos_pow.0)); // Flip numerator and denominator
    }

    // Positive integer exponent: multiply base by itself exp times
    let mut result = (1i64, 1i64);
    for _ in 0..exp {
        result = q_mul(result, base);
        // Check for overflow (saturating arithmetic in q_mul protects this)
        if result.0 == i64::MAX || result.1 == i64::MAX {
            return None; // Overflow
        }
    }
    Some(result)
}

/// Evaluates an expression and converts to f64 if possible
///
/// Useful for numerical applications and visualization.
pub fn try_eval_float(store: &Store, expr: ExprId) -> Option<f64> {
    let (num, den) = try_eval_constant(store, expr)?;
    Some(num as f64 / den as f64)
}

/// Folds constant subexpressions in an expression tree
///
/// Recursively evaluates constant subexpressions and replaces them
/// with their computed values. This is useful for simplifying expressions
/// before further symbolic manipulation.
///
/// # Examples
/// - (2 + 3) * x → 5 * x
/// - x + (1/2 * 4) → x + 2
pub fn fold_constants(store: &mut Store, expr: ExprId) -> ExprId {
    // Try to evaluate the entire expression first
    if let Some((num, den)) = try_eval_constant(store, expr) {
        return if den == 1 { store.int(num) } else { store.rat(num, den) };
    }

    // Otherwise, recursively fold children
    match &store.get(expr).op {
        Op::Add => {
            let children = &store.get(expr).children.clone();
            let folded: Vec<ExprId> = children.iter().map(|&c| fold_constants(store, c)).collect();
            store.add(folded)
        }
        Op::Mul => {
            let children = &store.get(expr).children.clone();
            let folded: Vec<ExprId> = children.iter().map(|&c| fold_constants(store, c)).collect();
            store.mul(folded)
        }
        Op::Pow => {
            let children = &store.get(expr).children.clone();
            if children.len() == 2 {
                let base = fold_constants(store, children[0]);
                let exp = fold_constants(store, children[1]);
                store.pow(base, exp)
            } else {
                expr
            }
        }
        Op::Function => {
            let fname = if let Payload::Func(f) = &store.get(expr).payload {
                f.clone()
            } else {
                return expr;
            };
            let children = &store.get(expr).children.clone();
            let folded: Vec<ExprId> = children.iter().map(|&c| fold_constants(store, c)).collect();
            store.func(&fname, folded)
        }
        _ => expr,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_integer() {
        let mut st = Store::new();
        let five = st.int(5);
        assert_eq!(try_eval_constant(&st, five), Some((5, 1)));
    }

    #[test]
    fn test_eval_rational() {
        let mut st = Store::new();
        let half = st.rat(1, 2);
        assert_eq!(try_eval_constant(&st, half), Some((1, 2)));
    }

    #[test]
    fn test_eval_add() {
        let mut st = Store::new();
        let two = st.int(2);
        let three = st.int(3);
        let sum = st.add(vec![two, three]);
        assert_eq!(try_eval_constant(&st, sum), Some((5, 1)));
    }

    #[test]
    fn test_eval_mul() {
        let mut st = Store::new();
        let two = st.int(2);
        let three = st.int(3);
        let product = st.mul(vec![two, three]);
        assert_eq!(try_eval_constant(&st, product), Some((6, 1)));
    }

    #[test]
    fn test_eval_pow() {
        let mut st = Store::new();
        let three = st.int(3);
        let two = st.int(2);
        let pow = st.pow(three, two);
        assert_eq!(try_eval_constant(&st, pow), Some((9, 1)));
    }

    #[test]
    fn test_eval_pow_negative() {
        let mut st = Store::new();
        let two = st.int(2);
        let neg_one = st.int(-1);
        let inv = st.pow(two, neg_one);
        assert_eq!(try_eval_constant(&st, inv), Some((1, 2)));
    }

    #[test]
    fn test_eval_rational_arithmetic() {
        let mut st = Store::new();
        let half = st.rat(1, 2);
        let quarter = st.rat(1, 4);
        let sum = st.add(vec![half, quarter]);
        assert_eq!(try_eval_constant(&st, sum), Some((3, 4)));
    }

    #[test]
    fn test_eval_with_variable_fails() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let expr = st.add(vec![x, two]);
        assert_eq!(try_eval_constant(&st, expr), None);
    }

    #[test]
    fn test_eval_float() {
        let mut st = Store::new();
        let three = st.int(3);
        let two = st.int(2);
        let ratio = st.rat(3, 2);

        assert_eq!(try_eval_float(&st, three), Some(3.0));
        assert_eq!(try_eval_float(&st, ratio), Some(1.5));

        let sum = st.add(vec![two, three]);
        assert_eq!(try_eval_float(&st, sum), Some(5.0));
    }

    #[test]
    fn test_fold_constants_simple() {
        let mut st = Store::new();
        let two = st.int(2);
        let three = st.int(3);
        let sum = st.add(vec![two, three]);

        let folded = fold_constants(&mut st, sum);
        assert_eq!(try_eval_constant(&st, folded), Some((5, 1)));
    }

    #[test]
    fn test_fold_constants_with_variable() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let three = st.int(3);
        let const_sum = st.add(vec![two, three]);
        let expr = st.mul(vec![const_sum, x]);

        let folded = fold_constants(&mut st, expr);
        // Should become 5 * x
        let children = &st.get(folded).children;
        assert_eq!(children.len(), 2);
        // One child should evaluate to 5
        let has_five = children.iter().any(|&c| try_eval_constant(&st, c) == Some((5, 1)));
        assert!(has_five);
    }

    #[test]
    fn test_fold_nested_constants() {
        let mut st = Store::new();
        let two = st.int(2);
        let three = st.int(3);
        let four = st.int(4);

        // (2 + 3) * 4 should fold to 5 * 4 = 20
        let inner = st.add(vec![two, three]);
        let outer = st.mul(vec![inner, four]);

        let folded = fold_constants(&mut st, outer);
        assert_eq!(try_eval_constant(&st, folded), Some((20, 1)));
    }
}
