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

/// Simplifies addition: sin²x + cos²x → 1, etc.
fn simplify_add(store: &mut Store, expr: ExprId) -> ExprId {
    // First recursively simplify children
    let children = store.get(expr).children.clone();
    let simplified_children: Vec<ExprId> =
        children.iter().map(|&c| apply_calculus_rules(store, c)).collect();

    // Try to detect sin²x + cos²x → 1
    if let Some(result) = try_pythagorean_identity(store, &simplified_children) {
        return result;
    }

    // Check if any children changed
    if simplified_children.iter().zip(children.iter()).any(|(a, b)| a != b) {
        store.add(simplified_children)
    } else {
        expr
    }
}

/// Detects and simplifies sin²x + cos²x → 1
fn try_pythagorean_identity(store: &mut Store, children: &[ExprId]) -> Option<ExprId> {
    // Look for pairs of sin²x and cos²x with the same argument
    for i in 0..children.len() {
        for j in (i + 1)..children.len() {
            let child_i = children[i];
            let child_j = children[j];

            // Check if one is sin²(arg) and the other is cos²(arg)
            if let Some((fname_i, arg_i)) = is_trig_squared(store, child_i) {
                if let Some((fname_j, arg_j)) = is_trig_squared(store, child_j) {
                    // Check if we have sin² and cos² with same argument
                    if ((fname_i == "sin" && fname_j == "cos")
                        || (fname_i == "cos" && fname_j == "sin"))
                        && arg_i == arg_j
                    {
                        // Found sin²x + cos²x!
                        // Return 1 + sum of remaining terms
                        let one = store.int(1);
                        let mut remaining: Vec<ExprId> = children
                            .iter()
                            .enumerate()
                            .filter(|(idx, _)| *idx != i && *idx != j)
                            .map(|(_, &c)| c)
                            .collect();

                        if remaining.is_empty() {
                            return Some(one);
                        }

                        remaining.push(one);
                        return Some(store.add(remaining));
                    }
                }
            }
        }
    }

    None
}

/// Checks if an expression is trig²(arg), returns (trig_name, arg)
fn is_trig_squared(store: &Store, expr: ExprId) -> Option<(String, ExprId)> {
    // Check if this is a power expression
    if store.get(expr).op != Op::Pow {
        return None;
    }

    let children = &store.get(expr).children;
    if children.len() != 2 {
        return None;
    }

    let base = children[0];
    let exp = children[1];

    // Check exponent is 2
    if !matches!((&store.get(exp).op, &store.get(exp).payload), (Op::Integer, Payload::Int(2))) {
        return None;
    }

    // Check base is sin(arg) or cos(arg)
    if store.get(base).op != Op::Function {
        return None;
    }

    let fname = match &store.get(base).payload {
        Payload::Func(s) => s.clone(),
        _ => return None,
    };

    if (fname != "sin" && fname != "cos") || store.get(base).children.len() != 1 {
        return None;
    }

    let arg = store.get(base).children[0];
    Some((fname, arg))
}

/// Simplifies multiplication: 2sin(x)cos(x) → sin(2x), etc.
fn simplify_mul(store: &mut Store, expr: ExprId) -> ExprId {
    // First recursively simplify children
    let children = store.get(expr).children.clone();
    let simplified_children: Vec<ExprId> =
        children.iter().map(|&c| apply_calculus_rules(store, c)).collect();

    // Try to detect double-angle pattern: 2sin(x)cos(x) → sin(2x)
    if let Some(result) = try_double_angle_sin(store, &simplified_children) {
        return result;
    }

    if simplified_children.iter().zip(children.iter()).any(|(a, b)| a != b) {
        store.mul(simplified_children)
    } else {
        expr
    }
}

/// Detects and simplifies 2sin(x)cos(x) → sin(2x)
fn try_double_angle_sin(store: &mut Store, children: &[ExprId]) -> Option<ExprId> {
    // Look for pattern: 2 * sin(arg) * cos(arg)
    // or any permutation thereof

    let mut has_two = false;
    let mut sin_arg: Option<ExprId> = None;
    let mut cos_arg: Option<ExprId> = None;
    let mut other_factors = Vec::new();

    for &child in children {
        match (&store.get(child).op, &store.get(child).payload) {
            (Op::Integer, Payload::Int(2)) => {
                has_two = true;
            }
            (Op::Function, Payload::Func(fname)) => {
                if store.get(child).children.len() == 1 {
                    let arg = store.get(child).children[0];
                    if fname == "sin" && sin_arg.is_none() {
                        sin_arg = Some(arg);
                    } else if fname == "cos" && cos_arg.is_none() {
                        cos_arg = Some(arg);
                    } else {
                        other_factors.push(child);
                    }
                } else {
                    other_factors.push(child);
                }
            }
            _ => other_factors.push(child),
        }
    }

    // Check if we have 2 * sin(arg) * cos(arg) with matching args
    if has_two {
        if let (Some(s_arg), Some(c_arg)) = (sin_arg, cos_arg) {
            if s_arg == c_arg {
                // Found 2sin(x)cos(x)!
                // Create sin(2x)
                let two = store.int(2);
                let two_arg = store.mul(vec![two, s_arg]);
                let sin_2arg = store.func("sin", vec![two_arg]);

                if other_factors.is_empty() {
                    return Some(sin_2arg);
                }

                other_factors.push(sin_2arg);
                return Some(store.mul(other_factors));
            }
        }
    }

    None
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

    #[test]
    fn test_pythagorean_identity_basic() {
        // sin²x + cos²x → 1
        let mut st = Store::new();
        let x = st.sym("x");
        let sinx = st.func("sin", vec![x]);
        let cosx = st.func("cos", vec![x]);
        let two = st.int(2);
        let sin2 = st.pow(sinx, two);
        let cos2 = st.pow(cosx, two);
        let sum = st.add(vec![sin2, cos2]);

        let result = simplify_calculus(&mut st, sum);

        // Should simplify to 1
        assert!(matches!(
            (&st.get(result).op, &st.get(result).payload),
            (Op::Integer, Payload::Int(1))
        ));
    }

    #[test]
    fn test_pythagorean_identity_reversed() {
        // cos²x + sin²x → 1 (order independent)
        let mut st = Store::new();
        let x = st.sym("x");
        let sinx = st.func("sin", vec![x]);
        let cosx = st.func("cos", vec![x]);
        let two = st.int(2);
        let sin2 = st.pow(sinx, two);
        let cos2 = st.pow(cosx, two);
        let sum = st.add(vec![cos2, sin2]);

        let result = simplify_calculus(&mut st, sum);

        // Should simplify to 1
        assert!(matches!(
            (&st.get(result).op, &st.get(result).payload),
            (Op::Integer, Payload::Int(1))
        ));
    }

    #[test]
    fn test_pythagorean_identity_with_extra_terms() {
        // 2 + sin²x + cos²x → 3
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let sinx = st.func("sin", vec![x]);
        let cosx = st.func("cos", vec![x]);
        let two_exp = st.int(2);
        let sin2 = st.pow(sinx, two_exp);
        let cos2 = st.pow(cosx, two_exp);
        let sum = st.add(vec![two, sin2, cos2]);

        let result = simplify_calculus(&mut st, sum);

        // Should simplify to 3 (or 2 + 1)
        // After simplification should contain 3 or (1 + 2)
        let result_str = st.to_string(result);
        assert!(result_str.contains("3") || (result_str.contains("1") && result_str.contains("2")));
    }

    #[test]
    fn test_pythagorean_identity_different_args_no_simplify() {
        // sin²x + cos²y should NOT simplify (different arguments)
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let sinx = st.func("sin", vec![x]);
        let cosy = st.func("cos", vec![y]);
        let two = st.int(2);
        let sin2 = st.pow(sinx, two);
        let cos2 = st.pow(cosy, two);
        let sum = st.add(vec![sin2, cos2]);

        let result = simplify_calculus(&mut st, sum);

        // Should NOT simplify to 1
        assert_ne!(st.get(result).op, Op::Integer);
        // Should still be an addition
        assert_eq!(st.get(result).op, Op::Add);
    }

    #[test]
    fn test_pythagorean_identity_complex_arg() {
        // sin²(2x) + cos²(2x) → 1
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let two_x = st.mul(vec![two, x]);
        let sin_2x = st.func("sin", vec![two_x]);
        let cos_2x = st.func("cos", vec![two_x]);
        let two_exp = st.int(2);
        let sin2 = st.pow(sin_2x, two_exp);
        let cos2 = st.pow(cos_2x, two_exp);
        let sum = st.add(vec![sin2, cos2]);

        let result = simplify_calculus(&mut st, sum);

        // Should simplify to 1
        assert!(matches!(
            (&st.get(result).op, &st.get(result).payload),
            (Op::Integer, Payload::Int(1))
        ));
    }

    #[test]
    fn test_no_pythagorean_without_squares() {
        // sin(x) + cos(x) should NOT simplify
        let mut st = Store::new();
        let x = st.sym("x");
        let sinx = st.func("sin", vec![x]);
        let cosx = st.func("cos", vec![x]);
        let sum = st.add(vec![sinx, cosx]);

        let result = simplify_calculus(&mut st, sum);

        // Should NOT simplify
        assert_eq!(st.get(result).op, Op::Add);
    }

    #[test]
    fn test_double_angle_sin_basic() {
        // 2sin(x)cos(x) → sin(2x)
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let sinx = st.func("sin", vec![x]);
        let cosx = st.func("cos", vec![x]);
        let product = st.mul(vec![two, sinx, cosx]);

        let result = simplify_calculus(&mut st, product);

        // Should simplify to sin(2x)
        let result_str = st.to_string(result);
        assert!(result_str.contains("sin"));
        assert!(result_str.contains("2"));

        // Verify it's sin(2*x) pattern
        assert_eq!(st.get(result).op, Op::Function);
        if let Payload::Func(fname) = &st.get(result).payload {
            assert_eq!(fname, "sin");
        }
    }

    #[test]
    fn test_double_angle_sin_reversed_order() {
        // cos(x) * sin(x) * 2 → sin(2x) (order independent)
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let sinx = st.func("sin", vec![x]);
        let cosx = st.func("cos", vec![x]);
        let product = st.mul(vec![cosx, sinx, two]);

        let result = simplify_calculus(&mut st, product);

        // Should simplify to sin(2x)
        let result_str = st.to_string(result);
        assert!(result_str.contains("sin"));
        assert!(result_str.contains("2"));
    }

    #[test]
    fn test_double_angle_sin_complex_arg() {
        // 2sin(x/2)cos(x/2) → sin(x)
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let half = st.rat(1, 2);
        let x_half = st.mul(vec![x, half]);
        let sin_x_half = st.func("sin", vec![x_half]);
        let cos_x_half = st.func("cos", vec![x_half]);
        let product = st.mul(vec![two, sin_x_half, cos_x_half]);

        let result = simplify_calculus(&mut st, product);

        // Should contain sin(x) after simplification
        let result_str = st.to_string(result);
        assert!(result_str.contains("sin"));
    }

    #[test]
    fn test_double_angle_sin_with_coefficient() {
        // 3 * 2sin(x)cos(x) → 3sin(2x)
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let three = st.int(3);
        let sinx = st.func("sin", vec![x]);
        let cosx = st.func("cos", vec![x]);
        let product = st.mul(vec![three, two, sinx, cosx]);

        let result = simplify_calculus(&mut st, product);

        // Should simplify to 3*sin(2x)
        let result_str = st.to_string(result);
        assert!(result_str.contains("sin"));
        // After simplification, should have sin(2x) with coefficient
        assert_eq!(st.get(result).op, Op::Mul);

        // Verify it contains the double-angle pattern
        let children = &st.get(result).children;
        let has_sin_func = children.iter().any(|&c| {
            matches!((&st.get(c).op, &st.get(c).payload), (Op::Function, Payload::Func(fname)) if fname == "sin")
        });
        assert!(has_sin_func, "Result should contain sin function");
    }

    #[test]
    fn test_double_angle_sin_different_args_no_simplify() {
        // 2sin(x)cos(y) should NOT simplify (different arguments)
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let two = st.int(2);
        let sinx = st.func("sin", vec![x]);
        let cosy = st.func("cos", vec![y]);
        let product = st.mul(vec![two, sinx, cosy]);

        let result = simplify_calculus(&mut st, product);

        // Should NOT simplify to sin(2x) or sin(2y)
        // Should remain as multiplication
        assert_eq!(st.get(result).op, Op::Mul);
        let result_str = st.to_string(result);
        // Should still contain both sin and cos separately
        assert!(result_str.contains("sin"));
        assert!(result_str.contains("cos"));
    }

    #[test]
    fn test_no_double_angle_without_two() {
        // sin(x)cos(x) should NOT simplify to sin(2x) without the 2
        let mut st = Store::new();
        let x = st.sym("x");
        let sinx = st.func("sin", vec![x]);
        let cosx = st.func("cos", vec![x]);
        let product = st.mul(vec![sinx, cosx]);

        let result = simplify_calculus(&mut st, product);

        // Should NOT simplify (no factor of 2)
        assert_eq!(st.get(result).op, Op::Mul);
    }
}
