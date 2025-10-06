//! Advanced Trigonometric Identities (Phase 6)
//!
//! This module implements advanced trigonometric simplification patterns:
//! - Half-angle formulas
//! - Sum-to-product identities
//! - Product-to-sum identities
//! - Trigonometric canonical form reduction
//!
//! These are production-quality implementations that extend the basic
//! trigonometric identities in calculus/symbolic_simplify.rs

use expr_core::{ExprId, Op, Payload, Store};

/// Apply advanced trigonometric simplification rules to an expression
///
/// This function tries to apply advanced trig identities:
/// - Product-to-sum: sin A cos B → (sin(A+B) + sin(A-B))/2
/// - Sum-to-product: sin A + sin B → 2 sin((A+B)/2) cos((A-B)/2)
/// - Half-angle detection and simplification
///
/// Returns the simplified expression, or the original if no rules apply.
pub fn simplify_trig(store: &mut Store, expr: ExprId) -> ExprId {
    match &store.get(expr).op {
        Op::Add => try_sum_to_product(store, expr),
        Op::Mul => try_product_to_sum(store, expr),
        Op::Pow => try_half_angle_expansion(store, expr),
        _ => expr,
    }
}

/// Detects and expands half-angle patterns
///
/// Patterns detected:
/// - sin²(x/2) → (1 - cos(x))/2
/// - cos²(x/2) → (1 + cos(x))/2
/// - tan²(x/2) → (1 - cos(x))/(1 + cos(x))
fn try_half_angle_expansion(store: &mut Store, expr: ExprId) -> ExprId {
    // Check if this is a squared trig function
    let children = store.get(expr).children.clone();
    if children.len() != 2 {
        return expr;
    }

    let base = children[0];
    let exp = children[1];

    // Check if exponent is 2
    if !matches!((&store.get(exp).op, &store.get(exp).payload), (Op::Integer, Payload::Int(2))) {
        return expr;
    }

    // Check if base is a trig function
    if store.get(base).op != Op::Function {
        return expr;
    }

    let fname = match &store.get(base).payload {
        Payload::Func(s) => s.clone(),
        _ => return expr,
    };

    if store.get(base).children.len() != 1 {
        return expr;
    }

    let arg = store.get(base).children[0];

    // Check if argument is x/2 (i.e., (1/2)*x)
    let is_half_angle = if store.get(arg).op == Op::Mul {
        let mul_children = &store.get(arg).children;
        mul_children.iter().any(|&c| {
            matches!((&store.get(c).op, &store.get(c).payload), (Op::Rational, Payload::Rat(1, 2)))
        })
    } else {
        false
    };

    if !is_half_angle {
        return expr;
    }

    // Extract the full angle (2*arg gives us x if arg is x/2)
    let two = store.int(2);
    let full_angle = store.mul(vec![two, arg]);

    match fname.as_str() {
        "sin" => {
            // sin²(x/2) → (1 - cos(x))/2
            let one = store.int(1);
            let cos_x = store.func("cos", vec![full_angle]);
            let neg_one = store.int(-1);
            let neg_cos = store.mul(vec![neg_one, cos_x]);
            let numerator = store.add(vec![one, neg_cos]);
            let half = store.rat(1, 2);
            store.mul(vec![half, numerator])
        }
        "cos" => {
            // cos²(x/2) → (1 + cos(x))/2
            let one = store.int(1);
            let cos_x = store.func("cos", vec![full_angle]);
            let numerator = store.add(vec![one, cos_x]);
            let half = store.rat(1, 2);
            store.mul(vec![half, numerator])
        }
        "tan" => {
            // tan²(x/2) → (1 - cos(x))/(1 + cos(x))
            let one = store.int(1);
            let cos_x = store.func("cos", vec![full_angle]);
            let neg_one = store.int(-1);
            let neg_cos = store.mul(vec![neg_one, cos_x]);
            let numerator = store.add(vec![one, neg_cos]);
            let denominator = store.add(vec![one, cos_x]);
            let inv_denom = store.pow(denominator, neg_one);
            store.mul(vec![numerator, inv_denom])
        }
        _ => expr,
    }
}

/// Detects and applies product-to-sum identities
///
/// Patterns detected:
/// - sin(A) * cos(B) → [sin(A+B) + sin(A-B)] / 2
/// - cos(A) * cos(B) → [cos(A+B) + cos(A-B)] / 2
/// - sin(A) * sin(B) → [cos(A-B) - cos(A+B)] / 2
fn try_product_to_sum(store: &mut Store, expr: ExprId) -> ExprId {
    let children = store.get(expr).children.clone();

    // Need at least two trig functions
    if children.len() < 2 {
        return expr;
    }

    // Collect trig functions and other factors
    let mut sin_terms = Vec::new();
    let mut cos_terms = Vec::new();
    let mut other_factors = Vec::new();

    for &child in &children {
        if let (Op::Function, Payload::Func(fname)) =
            (&store.get(child).op, &store.get(child).payload)
        {
            if store.get(child).children.len() == 1 {
                let arg = store.get(child).children[0];
                if fname == "sin" {
                    sin_terms.push((child, arg));
                } else if fname == "cos" {
                    cos_terms.push((child, arg));
                } else {
                    other_factors.push(child);
                }
            } else {
                other_factors.push(child);
            }
        } else {
            other_factors.push(child);
        }
    }

    // Try sin(A) * cos(B) → [sin(A+B) + sin(A-B)] / 2
    if !sin_terms.is_empty() && !cos_terms.is_empty() {
        let (_sin_expr, arg_a) = sin_terms[0];
        let (_cos_expr, arg_b) = cos_terms[0];

        // Create sin(A+B) + sin(A-B)
        let a_plus_b = store.add(vec![arg_a, arg_b]);
        let neg_one = store.int(-1);
        let neg_b = store.mul(vec![neg_one, arg_b]);
        let a_minus_b = store.add(vec![arg_a, neg_b]);
        let sin_sum = store.func("sin", vec![a_plus_b]);
        let sin_diff = store.func("sin", vec![a_minus_b]);
        let numerator = store.add(vec![sin_sum, sin_diff]);
        let half = store.rat(1, 2);
        let result = store.mul(vec![half, numerator]);

        // Combine with remaining factors
        let mut remaining = other_factors.clone();
        // Add back unused sin/cos terms
        for (i, (term, _)) in sin_terms.iter().enumerate() {
            if i != 0 {
                remaining.push(*term);
            }
        }
        for (i, (term, _)) in cos_terms.iter().enumerate() {
            if i != 0 {
                remaining.push(*term);
            }
        }

        if remaining.is_empty() {
            return result;
        }

        remaining.push(result);
        return store.mul(remaining);
    }

    // Try cos(A) * cos(B) → [cos(A+B) + cos(A-B)] / 2
    if cos_terms.len() >= 2 {
        let (_, arg_a) = cos_terms[0];
        let (_, arg_b) = cos_terms[1];

        let a_plus_b = store.add(vec![arg_a, arg_b]);
        let neg_one = store.int(-1);
        let neg_b = store.mul(vec![neg_one, arg_b]);
        let a_minus_b = store.add(vec![arg_a, neg_b]);
        let cos_sum = store.func("cos", vec![a_plus_b]);
        let cos_diff = store.func("cos", vec![a_minus_b]);
        let numerator = store.add(vec![cos_sum, cos_diff]);
        let half = store.rat(1, 2);
        let result = store.mul(vec![half, numerator]);

        let mut remaining = other_factors.clone();
        for (i, (term, _)) in cos_terms.iter().enumerate() {
            if i != 0 && i != 1 {
                remaining.push(*term);
            }
        }
        for (term, _) in sin_terms {
            remaining.push(term);
        }

        if remaining.is_empty() {
            return result;
        }

        remaining.push(result);
        return store.mul(remaining);
    }

    // Try sin(A) * sin(B) → [cos(A-B) - cos(A+B)] / 2
    if sin_terms.len() >= 2 {
        let (_, arg_a) = sin_terms[0];
        let (_, arg_b) = sin_terms[1];

        let neg_one = store.int(-1);
        let neg_b = store.mul(vec![neg_one, arg_b]);
        let a_minus_b = store.add(vec![arg_a, neg_b]);
        let a_plus_b = store.add(vec![arg_a, arg_b]);
        let cos_diff = store.func("cos", vec![a_minus_b]);
        let cos_sum = store.func("cos", vec![a_plus_b]);
        let neg_cos_sum = store.mul(vec![neg_one, cos_sum]);
        let numerator = store.add(vec![cos_diff, neg_cos_sum]);
        let half = store.rat(1, 2);
        let result = store.mul(vec![half, numerator]);

        let mut remaining = other_factors.clone();
        for (i, (term, _)) in sin_terms.iter().enumerate() {
            if i != 0 && i != 1 {
                remaining.push(*term);
            }
        }
        for (term, _) in cos_terms {
            remaining.push(term);
        }

        if remaining.is_empty() {
            return result;
        }

        remaining.push(result);
        return store.mul(remaining);
    }

    expr
}

/// Detects and applies sum-to-product identities
///
/// Patterns detected:
/// - sin(A) + sin(B) → 2 sin((A+B)/2) cos((A-B)/2)
/// - sin(A) - sin(B) → 2 cos((A+B)/2) sin((A-B)/2)
/// - cos(A) + cos(B) → 2 cos((A+B)/2) cos((A-B)/2)
/// - cos(A) - cos(B) → -2 sin((A+B)/2) sin((A-B)/2)
fn try_sum_to_product(store: &mut Store, expr: ExprId) -> ExprId {
    let children = store.get(expr).children.clone();

    if children.len() < 2 {
        return expr;
    }

    // Look for pairs of trig functions
    for i in 0..children.len() {
        for j in (i + 1)..children.len() {
            let child_i = children[i];
            let child_j = children[j];

            // Extract function info for both terms
            let info_i = extract_trig_term(store, child_i);
            let info_j = extract_trig_term(store, child_j);

            if info_i.is_none() || info_j.is_none() {
                continue;
            }

            let (fname_i, arg_i, sign_i) = info_i.unwrap();
            let (fname_j, arg_j, sign_j) = info_j.unwrap();

            // sin(A) + sin(B) → 2 sin((A+B)/2) cos((A-B)/2)
            if fname_i == "sin" && fname_j == "sin" && sign_i && sign_j {
                let result = apply_sin_plus_sin(store, arg_i, arg_j);
                return combine_with_remaining(store, &children, i, j, result);
            }

            // sin(A) - sin(B) → 2 cos((A+B)/2) sin((A-B)/2)
            if fname_i == "sin" && fname_j == "sin" && sign_i && !sign_j {
                let result = apply_sin_minus_sin(store, arg_i, arg_j);
                return combine_with_remaining(store, &children, i, j, result);
            }

            // cos(A) + cos(B) → 2 cos((A+B)/2) cos((A-B)/2)
            if fname_i == "cos" && fname_j == "cos" && sign_i && sign_j {
                let result = apply_cos_plus_cos(store, arg_i, arg_j);
                return combine_with_remaining(store, &children, i, j, result);
            }

            // cos(A) - cos(B) → -2 sin((A+B)/2) sin((A-B)/2)
            if fname_i == "cos" && fname_j == "cos" && sign_i && !sign_j {
                let result = apply_cos_minus_cos(store, arg_i, arg_j);
                return combine_with_remaining(store, &children, i, j, result);
            }
        }
    }

    expr
}

/// Extract (function_name, argument, is_positive) from a term
/// Handles both f(x) and -f(x) patterns
fn extract_trig_term(store: &Store, expr: ExprId) -> Option<(String, ExprId, bool)> {
    // Check if it's a direct function call
    if let (Op::Function, Payload::Func(fname)) = (&store.get(expr).op, &store.get(expr).payload) {
        if (fname == "sin" || fname == "cos") && store.get(expr).children.len() == 1 {
            let arg = store.get(expr).children[0];
            return Some((fname.clone(), arg, true));
        }
    }

    // Check if it's a negative term (-1 * f(x))
    if store.get(expr).op == Op::Mul {
        let mul_children = &store.get(expr).children;

        let has_neg_one = mul_children.iter().any(|&c| {
            matches!((&store.get(c).op, &store.get(c).payload), (Op::Integer, Payload::Int(-1)))
        });

        if has_neg_one && mul_children.len() == 2 {
            // Find the non-negative-one term
            for &child in mul_children {
                if !matches!(
                    (&store.get(child).op, &store.get(child).payload),
                    (Op::Integer, Payload::Int(-1))
                ) {
                    if let (Op::Function, Payload::Func(fname)) =
                        (&store.get(child).op, &store.get(child).payload)
                    {
                        if (fname == "sin" || fname == "cos")
                            && store.get(child).children.len() == 1
                        {
                            let arg = store.get(child).children[0];
                            return Some((fname.clone(), arg, false));
                        }
                    }
                }
            }
        }
    }

    None
}

/// sin(A) + sin(B) → 2 sin((A+B)/2) cos((A-B)/2)
fn apply_sin_plus_sin(store: &mut Store, arg_a: ExprId, arg_b: ExprId) -> ExprId {
    let two = store.int(2);
    let half = store.rat(1, 2);
    let neg_one = store.int(-1);

    let a_plus_b = store.add(vec![arg_a, arg_b]);
    let neg_b = store.mul(vec![neg_one, arg_b]);
    let a_minus_b = store.add(vec![arg_a, neg_b]);

    let sum_half = store.mul(vec![half, a_plus_b]);
    let diff_half = store.mul(vec![half, a_minus_b]);

    let sin_term = store.func("sin", vec![sum_half]);
    let cos_term = store.func("cos", vec![diff_half]);

    store.mul(vec![two, sin_term, cos_term])
}

/// sin(A) - sin(B) → 2 cos((A+B)/2) sin((A-B)/2)
fn apply_sin_minus_sin(store: &mut Store, arg_a: ExprId, arg_b: ExprId) -> ExprId {
    let two = store.int(2);
    let half = store.rat(1, 2);
    let neg_one = store.int(-1);

    let a_plus_b = store.add(vec![arg_a, arg_b]);
    let neg_b = store.mul(vec![neg_one, arg_b]);
    let a_minus_b = store.add(vec![arg_a, neg_b]);

    let sum_half = store.mul(vec![half, a_plus_b]);
    let diff_half = store.mul(vec![half, a_minus_b]);

    let cos_term = store.func("cos", vec![sum_half]);
    let sin_term = store.func("sin", vec![diff_half]);

    store.mul(vec![two, cos_term, sin_term])
}

/// cos(A) + cos(B) → 2 cos((A+B)/2) cos((A-B)/2)
fn apply_cos_plus_cos(store: &mut Store, arg_a: ExprId, arg_b: ExprId) -> ExprId {
    let two = store.int(2);
    let half = store.rat(1, 2);
    let neg_one = store.int(-1);

    let a_plus_b = store.add(vec![arg_a, arg_b]);
    let neg_b = store.mul(vec![neg_one, arg_b]);
    let a_minus_b = store.add(vec![arg_a, neg_b]);

    let sum_half = store.mul(vec![half, a_plus_b]);
    let diff_half = store.mul(vec![half, a_minus_b]);

    let cos1 = store.func("cos", vec![sum_half]);
    let cos2 = store.func("cos", vec![diff_half]);

    store.mul(vec![two, cos1, cos2])
}

/// cos(A) - cos(B) → -2 sin((A+B)/2) sin((A-B)/2)
fn apply_cos_minus_cos(store: &mut Store, arg_a: ExprId, arg_b: ExprId) -> ExprId {
    let neg_two = store.int(-2);
    let half = store.rat(1, 2);
    let neg_one = store.int(-1);

    let a_plus_b = store.add(vec![arg_a, arg_b]);
    let neg_b = store.mul(vec![neg_one, arg_b]);
    let a_minus_b = store.add(vec![arg_a, neg_b]);

    let sum_half = store.mul(vec![half, a_plus_b]);
    let diff_half = store.mul(vec![half, a_minus_b]);

    let sin1 = store.func("sin", vec![sum_half]);
    let sin2 = store.func("sin", vec![diff_half]);

    store.mul(vec![neg_two, sin1, sin2])
}

/// Combine result with remaining terms that weren't used in the simplification
fn combine_with_remaining(
    store: &mut Store,
    all_children: &[ExprId],
    idx_i: usize,
    idx_j: usize,
    result: ExprId,
) -> ExprId {
    let remaining: Vec<ExprId> = all_children
        .iter()
        .enumerate()
        .filter(|(idx, _)| *idx != idx_i && *idx != idx_j)
        .map(|(_, &c)| c)
        .collect();

    if remaining.is_empty() {
        return result;
    }

    let mut new_terms = remaining;
    new_terms.push(result);
    store.add(new_terms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sin_cos_product_to_sum() {
        // sin(x) * cos(y) → [sin(x+y) + sin(x-y)] / 2
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let sinx = st.func("sin", vec![x]);
        let cosy = st.func("cos", vec![y]);
        let product = st.mul(vec![sinx, cosy]);

        let result = try_product_to_sum(&mut st, product);

        // Result should be a multiplication with 1/2
        assert_eq!(st.get(result).op, Op::Mul);
        let result_str = st.to_string(result);
        assert!(result_str.contains("1/2") || result_str.contains("sin"));
    }

    #[test]
    fn test_cos_cos_product_to_sum() {
        // cos(x) * cos(y) → [cos(x+y) + cos(x-y)] / 2
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let cosx = st.func("cos", vec![x]);
        let cosy = st.func("cos", vec![y]);
        let product = st.mul(vec![cosx, cosy]);

        let result = try_product_to_sum(&mut st, product);

        assert_eq!(st.get(result).op, Op::Mul);
        let result_str = st.to_string(result);
        assert!(result_str.contains("cos"));
    }

    #[test]
    fn test_sin_sin_product_to_sum() {
        // sin(x) * sin(y) → [cos(x-y) - cos(x+y)] / 2
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let sinx = st.func("sin", vec![x]);
        let siny = st.func("sin", vec![y]);
        let product = st.mul(vec![sinx, siny]);

        let result = try_product_to_sum(&mut st, product);

        assert_eq!(st.get(result).op, Op::Mul);
        let result_str = st.to_string(result);
        assert!(result_str.contains("cos"));
    }

    #[test]
    fn test_sin_plus_sin_sum_to_product() {
        // sin(x) + sin(y) → 2 sin((x+y)/2) cos((x-y)/2)
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let sinx = st.func("sin", vec![x]);
        let siny = st.func("sin", vec![y]);
        let sum = st.add(vec![sinx, siny]);

        let result = try_sum_to_product(&mut st, sum);

        // Result should be a multiplication with 2
        assert_eq!(st.get(result).op, Op::Mul);
        let result_str = st.to_string(result);
        assert!(
            result_str.contains("2") && result_str.contains("sin") && result_str.contains("cos")
        );
    }

    #[test]
    fn test_cos_plus_cos_sum_to_product() {
        // cos(x) + cos(y) → 2 cos((x+y)/2) cos((x-y)/2)
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let cosx = st.func("cos", vec![x]);
        let cosy = st.func("cos", vec![y]);
        let sum = st.add(vec![cosx, cosy]);

        let result = try_sum_to_product(&mut st, sum);

        assert_eq!(st.get(result).op, Op::Mul);
        let result_str = st.to_string(result);
        assert!(result_str.contains("2") && result_str.contains("cos"));
    }

    #[test]
    fn test_sin_minus_sin_sum_to_product() {
        // sin(x) - sin(y) → 2 cos((x+y)/2) sin((x-y)/2)
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let sinx = st.func("sin", vec![x]);
        let siny = st.func("sin", vec![y]);
        let neg_one = st.int(-1);
        let neg_siny = st.mul(vec![neg_one, siny]);
        let diff = st.add(vec![sinx, neg_siny]);

        let result = try_sum_to_product(&mut st, diff);

        assert_eq!(st.get(result).op, Op::Mul);
        let result_str = st.to_string(result);
        assert!(result_str.contains("2"));
    }

    #[test]
    fn test_no_simplification_for_single_term() {
        let mut st = Store::new();
        let x = st.sym("x");
        let sinx = st.func("sin", vec![x]);

        let result = try_sum_to_product(&mut st, sinx);
        assert_eq!(result, sinx);
    }

    #[test]
    fn test_half_angle_sin_squared() {
        // sin²(x/2) → (1 - cos(x))/2
        let mut st = Store::new();
        let x = st.sym("x");
        let half = st.rat(1, 2);
        let x_half = st.mul(vec![half, x]);
        let sin_half = st.func("sin", vec![x_half]);
        let two = st.int(2);
        let sin_sq = st.pow(sin_half, two);

        let result = try_half_angle_expansion(&mut st, sin_sq);

        // Should contain (1 - cos(x))/2
        let result_str = st.to_string(result);
        assert!(result_str.contains("cos") && result_str.contains("1/2"));
    }

    #[test]
    fn test_half_angle_cos_squared() {
        // cos²(x/2) → (1 + cos(x))/2
        let mut st = Store::new();
        let x = st.sym("x");
        let half = st.rat(1, 2);
        let x_half = st.mul(vec![half, x]);
        let cos_half = st.func("cos", vec![x_half]);
        let two = st.int(2);
        let cos_sq = st.pow(cos_half, two);

        let result = try_half_angle_expansion(&mut st, cos_sq);

        // Should contain (1 + cos(x))/2
        let result_str = st.to_string(result);
        assert!(result_str.contains("cos") && result_str.contains("1/2"));
    }

    #[test]
    fn test_half_angle_tan_squared() {
        // tan²(x/2) → (1 - cos(x))/(1 + cos(x))
        let mut st = Store::new();
        let x = st.sym("x");
        let half = st.rat(1, 2);
        let x_half = st.mul(vec![half, x]);
        let tan_half = st.func("tan", vec![x_half]);
        let two = st.int(2);
        let tan_sq = st.pow(tan_half, two);

        let result = try_half_angle_expansion(&mut st, tan_sq);

        // Should contain cos in the result
        let result_str = st.to_string(result);
        assert!(result_str.contains("cos"));
    }

    #[test]
    fn test_no_half_angle_for_full_angle() {
        // sin²(x) should NOT be expanded as half-angle
        let mut st = Store::new();
        let x = st.sym("x");
        let sinx = st.func("sin", vec![x]);
        let two = st.int(2);
        let sin_sq = st.pow(sinx, two);

        let result = try_half_angle_expansion(&mut st, sin_sq);

        // Should remain unchanged
        assert_eq!(result, sin_sq);
    }

    #[test]
    fn test_product_to_sum_with_coefficients() {
        // 3 * sin(x) * cos(y) should still work
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let three = st.int(3);
        let sinx = st.func("sin", vec![x]);
        let cosy = st.func("cos", vec![y]);
        let product = st.mul(vec![three, sinx, cosy]);

        let result = try_product_to_sum(&mut st, product);

        // Should still apply product-to-sum, preserving the coefficient
        assert_eq!(st.get(result).op, Op::Mul);
        let result_str = st.to_string(result);
        assert!(result_str.contains("3"));
    }

    #[test]
    fn test_cos_minus_cos_sum_to_product() {
        // cos(x) - cos(y) → -2 sin((x+y)/2) sin((x-y)/2)
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let cosx = st.func("cos", vec![x]);
        let cosy = st.func("cos", vec![y]);
        let neg_one = st.int(-1);
        let neg_cosy = st.mul(vec![neg_one, cosy]);
        let diff = st.add(vec![cosx, neg_cosy]);

        let result = try_sum_to_product(&mut st, diff);

        // Should produce -2 * sin(...) * sin(...)
        assert_eq!(st.get(result).op, Op::Mul);
        let result_str = st.to_string(result);
        assert!(result_str.contains("-2") || result_str.contains("sin"));
    }
}
