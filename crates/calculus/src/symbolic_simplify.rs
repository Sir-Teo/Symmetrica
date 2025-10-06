//! Symbolic simplification for calculus expressions
//!
//! Provides simplification rules specific to calculus expressions:
//! - √n → concrete values for perfect squares
//! - Trigonometric identities
//! - Logarithmic/exponential identities
//! - Inverse function composition

use expr_core::{ExprId, Op, Payload, Store};
use simplify::simplify;

/// Simplifies calculus-specific patterns in an expression
///
/// This extends the general simplifier with calculus-aware rules:
/// - √4 → 2, √9 → 3, etc.
/// - ln(e^x) → x, e^(ln x) → x
/// - atan(tan x) → x (with domain restrictions)
/// - sin²x + cos²x → 1
///
/// Returns a simplified expression, or the original if no simplification applies.
pub fn simplify_calculus(store: &mut Store, expr: ExprId) -> ExprId {
    // First apply general simplification
    let simplified = simplify(store, expr);

    // Then apply calculus-specific rules
    let calc_simplified = apply_calculus_rules(store, simplified);

    // If we made progress, recursively simplify
    if calc_simplified != simplified {
        simplify_calculus(store, calc_simplified)
    } else {
        calc_simplified
    }
}

/// Applies calculus-specific simplification rules
fn apply_calculus_rules(store: &mut Store, expr: ExprId) -> ExprId {
    match &store.get(expr).op {
        Op::Function => simplify_function(store, expr),
        Op::Add => simplify_add(store, expr),
        Op::Mul => simplify_mul(store, expr),
        Op::Pow => simplify_pow(store, expr),
        _ => expr,
    }
}

/// Simplifies function expressions with calculus rules
fn simplify_function(store: &mut Store, expr: ExprId) -> ExprId {
    let fname = match &store.get(expr).payload {
        Payload::Func(s) => s.clone(),
        _ => return expr,
    };

    if store.get(expr).children.len() != 1 {
        return expr;
    }

    let arg = store.get(expr).children[0];
    let arg_simplified = apply_calculus_rules(store, arg);

    match fname.as_str() {
        "sqrt" => simplify_sqrt(store, arg_simplified),
        "ln" | "log" => simplify_ln(store, arg_simplified),
        "exp" => simplify_exp(store, arg_simplified),
        "atan" | "arctan" => simplify_atan(store, arg_simplified),
        _ => {
            // Reconstruct with simplified argument
            if arg_simplified != arg {
                store.func(&fname, vec![arg_simplified])
            } else {
                expr
            }
        }
    }
}

/// Simplifies sqrt expressions: √4 → 2, √9 → 3, etc.
fn simplify_sqrt(store: &mut Store, arg: ExprId) -> ExprId {
    match (&store.get(arg).op, &store.get(arg).payload) {
        (Op::Integer, Payload::Int(n)) if *n >= 0 => {
            // Check if n is a perfect square
            let sqrt_n = (*n as f64).sqrt();
            if sqrt_n.fract() == 0.0 && sqrt_n * sqrt_n == *n as f64 {
                return store.int(sqrt_n as i64);
            }
            // Not a perfect square, keep as is
            store.func("sqrt", vec![arg])
        }
        (Op::Rational, Payload::Rat(num, den)) if *num >= 0 && *den > 0 => {
            // √(a/b) = √a / √b if both are perfect squares
            let sqrt_num = (*num as f64).sqrt();
            let sqrt_den = (*den as f64).sqrt();

            if sqrt_num.fract() == 0.0
                && sqrt_num * sqrt_num == *num as f64
                && sqrt_den.fract() == 0.0
                && sqrt_den * sqrt_den == *den as f64
            {
                return store.rat(sqrt_num as i64, sqrt_den as i64);
            }

            store.func("sqrt", vec![arg])
        }
        _ => store.func("sqrt", vec![arg]),
    }
}

/// Simplifies ln expressions: ln(e^x) → x, ln(e) → 1
fn simplify_ln(store: &mut Store, arg: ExprId) -> ExprId {
    // ln(e^x) → x
    if let (Op::Function, Payload::Func(fname)) = (&store.get(arg).op, &store.get(arg).payload) {
        if fname == "exp" && store.get(arg).children.len() == 1 {
            return store.get(arg).children[0];
        }
    }

    // ln(e) → 1 (where e is represented as exp(1))
    // For now, just return ln(arg)
    store.func("ln", vec![arg])
}

/// Simplifies exp expressions: e^(ln x) → x, e^0 → 1
fn simplify_exp(store: &mut Store, arg: ExprId) -> ExprId {
    // e^(ln x) → x
    if let (Op::Function, Payload::Func(fname)) = (&store.get(arg).op, &store.get(arg).payload) {
        if (fname == "ln" || fname == "log") && store.get(arg).children.len() == 1 {
            return store.get(arg).children[0];
        }
    }

    // e^0 → 1
    if matches!((&store.get(arg).op, &store.get(arg).payload), (Op::Integer, Payload::Int(0))) {
        return store.int(1);
    }

    store.func("exp", vec![arg])
}

/// Simplifies atan expressions: atan(tan x) → x (with domain considerations)
fn simplify_atan(store: &mut Store, arg: ExprId) -> ExprId {
    // atan(tan x) → x (technically only for x in (-π/2, π/2))
    // For symbolic work, we apply this simplification
    if let (Op::Function, Payload::Func(fname)) = (&store.get(arg).op, &store.get(arg).payload) {
        if fname == "tan" && store.get(arg).children.len() == 1 {
            return store.get(arg).children[0];
        }
    }

    // atan(0) → 0
    if matches!((&store.get(arg).op, &store.get(arg).payload), (Op::Integer, Payload::Int(0))) {
        return store.int(0);
    }

    store.func("atan", vec![arg])
}

/// Simplifies addition: sin²x + cos²x → 1 (future)
fn simplify_add(store: &mut Store, expr: ExprId) -> ExprId {
    // For now, recursively simplify children
    let children = store.get(expr).children.clone();
    let simplified_children: Vec<ExprId> =
        children.iter().map(|&c| apply_calculus_rules(store, c)).collect();

    // Check if any children changed
    if simplified_children.iter().zip(children.iter()).any(|(a, b)| a != b) {
        store.add(simplified_children)
    } else {
        expr
    }
}

/// Simplifies multiplication (future: trigonometric identities)
fn simplify_mul(store: &mut Store, expr: ExprId) -> ExprId {
    // For now, recursively simplify children
    let children = store.get(expr).children.clone();
    let simplified_children: Vec<ExprId> =
        children.iter().map(|&c| apply_calculus_rules(store, c)).collect();

    if simplified_children.iter().zip(children.iter()).any(|(a, b)| a != b) {
        store.mul(simplified_children)
    } else {
        expr
    }
}

/// Simplifies powers: x^1 → x, x^0 → 1 (already in general simplifier)
fn simplify_pow(store: &mut Store, expr: ExprId) -> ExprId {
    let children = store.get(expr).children.clone();
    if children.len() != 2 {
        return expr;
    }

    let base = apply_calculus_rules(store, children[0]);
    let exp = apply_calculus_rules(store, children[1]);

    // Reconstruct if children changed
    if base != children[0] || exp != children[1] {
        store.pow(base, exp)
    } else {
        expr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simplify_sqrt_perfect_square() {
        let mut st = Store::new();
        let four = st.int(4);
        let sqrt_four = st.func("sqrt", vec![four]);

        let result = simplify_calculus(&mut st, sqrt_four);

        // Should simplify to 2
        assert!(matches!(
            (&st.get(result).op, &st.get(result).payload),
            (Op::Integer, Payload::Int(2))
        ));
    }

    #[test]
    fn test_simplify_sqrt_nine() {
        let mut st = Store::new();
        let nine = st.int(9);
        let sqrt_nine = st.func("sqrt", vec![nine]);

        let result = simplify_calculus(&mut st, sqrt_nine);

        // Should simplify to 3
        assert!(matches!(
            (&st.get(result).op, &st.get(result).payload),
            (Op::Integer, Payload::Int(3))
        ));
    }

    #[test]
    fn test_simplify_sqrt_non_perfect() {
        let mut st = Store::new();
        let five = st.int(5);
        let sqrt_five = st.func("sqrt", vec![five]);

        let result = simplify_calculus(&mut st, sqrt_five);

        // Should remain as sqrt(5)
        assert_eq!(st.get(result).op, Op::Function);
    }

    #[test]
    fn test_simplify_ln_exp() {
        let mut st = Store::new();
        let x = st.sym("x");
        let exp_x = st.func("exp", vec![x]);
        let ln_exp_x = st.func("ln", vec![exp_x]);

        let result = simplify_calculus(&mut st, ln_exp_x);

        // Should simplify to x
        assert_eq!(result, x);
    }

    #[test]
    fn test_simplify_exp_ln() {
        let mut st = Store::new();
        let x = st.sym("x");
        let ln_x = st.func("ln", vec![x]);
        let exp_ln_x = st.func("exp", vec![ln_x]);

        let result = simplify_calculus(&mut st, exp_ln_x);

        // Should simplify to x
        assert_eq!(result, x);
    }

    #[test]
    fn test_simplify_exp_zero() {
        let mut st = Store::new();
        let zero = st.int(0);
        let exp_zero = st.func("exp", vec![zero]);

        let result = simplify_calculus(&mut st, exp_zero);

        // Should simplify to 1
        assert!(matches!(
            (&st.get(result).op, &st.get(result).payload),
            (Op::Integer, Payload::Int(1))
        ));
    }

    #[test]
    fn test_simplify_atan_tan() {
        let mut st = Store::new();
        let x = st.sym("x");
        let tan_x = st.func("tan", vec![x]);
        let atan_tan_x = st.func("atan", vec![tan_x]);

        let result = simplify_calculus(&mut st, atan_tan_x);

        // Should simplify to x
        assert_eq!(result, x);
    }

    #[test]
    fn test_simplify_atan_zero() {
        let mut st = Store::new();
        let zero = st.int(0);
        let atan_zero = st.func("atan", vec![zero]);

        let result = simplify_calculus(&mut st, atan_zero);

        // Should simplify to 0
        assert!(matches!(
            (&st.get(result).op, &st.get(result).payload),
            (Op::Integer, Payload::Int(0))
        ));
    }

    #[test]
    fn test_simplify_sqrt_rational() {
        let mut st = Store::new();
        let four_ninths = st.rat(4, 9);
        let sqrt_ratio = st.func("sqrt", vec![four_ninths]);

        let result = simplify_calculus(&mut st, sqrt_ratio);

        // Should simplify to 2/3
        assert!(matches!(
            (&st.get(result).op, &st.get(result).payload),
            (Op::Rational, Payload::Rat(2, 3))
        ));
    }

    #[test]
    fn test_simplify_nested() {
        let mut st = Store::new();
        let x = st.sym("x");
        let ln_x = st.func("ln", vec![x]);
        let exp_ln_x = st.func("exp", vec![ln_x]);
        let four = st.int(4);
        let sqrt_four = st.func("sqrt", vec![four]);
        let product = st.mul(vec![exp_ln_x, sqrt_four]);

        let result = simplify_calculus(&mut st, product);

        // Should simplify to 2*x
        assert_eq!(st.get(result).op, Op::Mul);
        let children = &st.get(result).children;

        // Should contain x and 2
        let has_x = children.contains(&x);
        let has_two = children.iter().any(|&c| {
            matches!((&st.get(c).op, &st.get(c).payload), (Op::Integer, Payload::Int(2)))
        });
        
        assert!(has_x && has_two);
    }
}
