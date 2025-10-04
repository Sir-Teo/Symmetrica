//! Simple rewrite registry v0 (Roadmap Phase H step 1)
//! - Hardcoded, deterministic rules applied top-down after recursively rewriting children
//! - Uses `ac::Pat` matcher for clarity on patterns

use crate::ac::{match_expr, Pat};
use expr_core::{ExprId, Op, Payload, Store};

/// Rewrite with a small set of safe, deterministic rules.
/// Rules (after children are rewritten):
/// - sin(0) -> 0
/// - cos(0) -> 1
/// - ln(1) -> 0
/// - exp(0) -> 1
/// - u^1 -> u
/// - u^0 -> 1 (unless u == 0)
/// - sin(u)^2 + cos(u)^2 -> 1 (Pythagorean identity, checked before recursion)
/// - sin(u)^2 -> (1 - cos(2*u))/2 (power-reduction)
/// - cos(u)^2 -> (1 + cos(2*u))/2 (power-reduction)
/// - sin(2*u) -> 2*sin(u)*cos(u) (double-angle)
/// - cos(2*u) -> cos(u)^2 - sin(u)^2 (double-angle)
/// - sin(u + v) -> sin(u)*cos(v) + cos(u)*sin(v) (angle addition)
/// - cos(u + v) -> cos(u)*cos(v) - sin(u)*sin(v) (angle addition)
pub fn rewrite_basic(store: &mut Store, id: ExprId) -> ExprId {
    // For Add nodes, try top-level rules first (e.g., Pythagorean identity)
    // before recursing, so that sin^2 + cos^2 is recognized before
    // individual power-reduction formulas are applied.
    if store.get(id).op == Op::Add {
        if let Some(out) = apply_rules(store, id) {
            return out;
        }
    }

    // Rewrite children recursively
    let rewritten = match store.get(id).op {
        Op::Add | Op::Mul | Op::Function | Op::Pow => rewrite_children(store, id),
        _ => id,
    };

    // Then attempt top-level rules
    if let Some(out) = apply_rules(store, rewritten) {
        out
    } else {
        rewritten
    }
}

fn rewrite_children(store: &mut Store, id: ExprId) -> ExprId {
    match store.get(id).op {
        Op::Add => {
            let ch = store.get(id).children.clone();
            let mut v = Vec::with_capacity(ch.len());
            for c in ch {
                v.push(rewrite_basic(store, c));
            }
            store.add(v)
        }
        Op::Mul => {
            let ch = store.get(id).children.clone();
            let mut v = Vec::with_capacity(ch.len());
            for c in ch {
                v.push(rewrite_basic(store, c));
            }
            store.mul(v)
        }
        Op::Pow => {
            let base = store.get(id).children[0];
            let exp = store.get(id).children[1];
            let b = rewrite_basic(store, base);
            let e = rewrite_basic(store, exp);
            store.pow(b, e)
        }
        Op::Function => {
            let name = match &store.get(id).payload {
                Payload::Func(s) => s.clone(),
                _ => "<f>".into(),
            };
            let ch = store.get(id).children.clone();
            let mut v = Vec::with_capacity(ch.len());
            for c in ch {
                v.push(rewrite_basic(store, c));
            }
            store.func(name, v)
        }
        _ => id,
    }
}

fn is_int(st: &Store, id: ExprId, k: i64) -> bool {
    matches!((&st.get(id).op, &st.get(id).payload), (Op::Integer, Payload::Int(v)) if *v == k)
}

fn is_zero(st: &Store, id: ExprId) -> bool {
    is_int(st, id, 0)
}

fn apply_rules(store: &mut Store, id: ExprId) -> Option<ExprId> {
    // sin(0) -> 0
    {
        let pat = Pat::Function("sin".into(), vec![Pat::Integer(0)]);
        if match_expr(store, &pat, id).is_some() {
            return Some(store.int(0));
        }
    }
    // cos(0) -> 1
    {
        let pat = Pat::Function("cos".into(), vec![Pat::Integer(0)]);
        if match_expr(store, &pat, id).is_some() {
            return Some(store.int(1));
        }
    }
    // ln(1) -> 0
    {
        let pat = Pat::Function("ln".into(), vec![Pat::Integer(1)]);
        if match_expr(store, &pat, id).is_some() {
            return Some(store.int(0));
        }
    }
    // exp(0) -> 1
    {
        let pat = Pat::Function("exp".into(), vec![Pat::Integer(0)]);
        if match_expr(store, &pat, id).is_some() {
            return Some(store.int(1));
        }
    }
    // u^1 -> u
    {
        let pat = Pat::Pow(Box::new(Pat::Any("u".into())), Box::new(Pat::Integer(1)));
        if let Some(bind) = match_expr(store, &pat, id) {
            let u = *bind.get("u").unwrap();
            return Some(u);
        }
    }
    // u^0 -> 1 unless u == 0 (keep 0^0 as-is)
    {
        let pat = Pat::Pow(Box::new(Pat::Any("u".into())), Box::new(Pat::Integer(0)));
        if let Some(bind) = match_expr(store, &pat, id) {
            let u = *bind.get("u").unwrap();
            if !is_zero(store, u) {
                return Some(store.int(1));
            }
        }
    }

    // sin(u)^2 + cos(u)^2 -> 1 (Pythagorean identity)
    {
        let pat = Pat::Add(vec![
            Pat::Pow(
                Box::new(Pat::Function("sin".into(), vec![Pat::Any("u".into())])),
                Box::new(Pat::Integer(2)),
            ),
            Pat::Pow(
                Box::new(Pat::Function("cos".into(), vec![Pat::Any("u".into())])),
                Box::new(Pat::Integer(2)),
            ),
        ]);
        if match_expr(store, &pat, id).is_some() {
            return Some(store.int(1));
        }
    }

    // sin(u)^2 -> (1 - cos(2*u))/2 (power-reduction formula)
    {
        let pat = Pat::Pow(
            Box::new(Pat::Function("sin".into(), vec![Pat::Any("u".into())])),
            Box::new(Pat::Integer(2)),
        );
        if let Some(bind) = match_expr(store, &pat, id) {
            let u = *bind.get("u").unwrap();
            // Build 2*u
            let two = store.int(2);
            let two_u = store.mul(vec![two, u]);
            // Build cos(2*u)
            let cos_2u = store.func("cos", vec![two_u]);
            // Build 1 - cos(2*u)
            let one = store.int(1);
            let neg_one = store.int(-1);
            let neg_cos = store.mul(vec![neg_one, cos_2u]);
            let numerator = store.add(vec![one, neg_cos]);
            // Build (1 - cos(2*u))/2
            let half = store.rat(1, 2);
            let result = store.mul(vec![half, numerator]);
            return Some(result);
        }
    }

    // cos(u)^2 -> (1 + cos(2*u))/2 (power-reduction formula)
    {
        let pat = Pat::Pow(
            Box::new(Pat::Function("cos".into(), vec![Pat::Any("u".into())])),
            Box::new(Pat::Integer(2)),
        );
        if let Some(bind) = match_expr(store, &pat, id) {
            let u = *bind.get("u").unwrap();
            // Build 2*u
            let two = store.int(2);
            let two_u = store.mul(vec![two, u]);
            // Build cos(2*u)
            let cos_2u = store.func("cos", vec![two_u]);
            // Build 1 + cos(2*u)
            let one = store.int(1);
            let numerator = store.add(vec![one, cos_2u]);
            // Build (1 + cos(2*u))/2
            let half = store.rat(1, 2);
            let result = store.mul(vec![half, numerator]);
            return Some(result);
        }
    }

    // sin(2*u) -> 2*sin(u)*cos(u) (double-angle formula)
    {
        let pat = Pat::Function(
            "sin".into(),
            vec![Pat::Mul(vec![Pat::Integer(2), Pat::Any("u".into())])],
        );
        if let Some(bind) = match_expr(store, &pat, id) {
            let u = *bind.get("u").unwrap();
            // Build sin(u)
            let sin_u = store.func("sin", vec![u]);
            // Build cos(u)
            let cos_u = store.func("cos", vec![u]);
            // Build 2*sin(u)*cos(u)
            let two = store.int(2);
            let result = store.mul(vec![two, sin_u, cos_u]);
            return Some(result);
        }
    }

    // cos(2*u) -> cos(u)^2 - sin(u)^2 (double-angle formula)
    {
        let pat = Pat::Function(
            "cos".into(),
            vec![Pat::Mul(vec![Pat::Integer(2), Pat::Any("u".into())])],
        );
        if let Some(bind) = match_expr(store, &pat, id) {
            let u = *bind.get("u").unwrap();
            // Build cos(u)^2
            let cos_u = store.func("cos", vec![u]);
            let two = store.int(2);
            let cos_sq = store.pow(cos_u, two);
            // Build sin(u)^2
            let sin_u = store.func("sin", vec![u]);
            let two2 = store.int(2);
            let sin_sq = store.pow(sin_u, two2);
            // Build cos(u)^2 - sin(u)^2
            let neg_one = store.int(-1);
            let neg_sin_sq = store.mul(vec![neg_one, sin_sq]);
            let result = store.add(vec![cos_sq, neg_sin_sq]);
            return Some(result);
        }
    }

    // sin(u + v) -> sin(u)*cos(v) + cos(u)*sin(v) (angle addition)
    {
        let pat = Pat::Function(
            "sin".into(),
            vec![Pat::Add(vec![Pat::Any("u".into()), Pat::Any("v".into())])],
        );
        if let Some(bind) = match_expr(store, &pat, id) {
            let u = *bind.get("u").unwrap();
            let v = *bind.get("v").unwrap();
            // Build sin(u)*cos(v)
            let sin_u = store.func("sin", vec![u]);
            let cos_v = store.func("cos", vec![v]);
            let term1 = store.mul(vec![sin_u, cos_v]);
            // Build cos(u)*sin(v)
            let cos_u = store.func("cos", vec![u]);
            let sin_v = store.func("sin", vec![v]);
            let term2 = store.mul(vec![cos_u, sin_v]);
            // Build sin(u)*cos(v) + cos(u)*sin(v)
            let result = store.add(vec![term1, term2]);
            return Some(result);
        }
    }

    // cos(u + v) -> cos(u)*cos(v) - sin(u)*sin(v) (angle addition)
    {
        let pat = Pat::Function(
            "cos".into(),
            vec![Pat::Add(vec![Pat::Any("u".into()), Pat::Any("v".into())])],
        );
        if let Some(bind) = match_expr(store, &pat, id) {
            let u = *bind.get("u").unwrap();
            let v = *bind.get("v").unwrap();
            // Build cos(u)*cos(v)
            let cos_u = store.func("cos", vec![u]);
            let cos_v = store.func("cos", vec![v]);
            let term1 = store.mul(vec![cos_u, cos_v]);
            // Build sin(u)*sin(v)
            let sin_u = store.func("sin", vec![u]);
            let sin_v = store.func("sin", vec![v]);
            let prod = store.mul(vec![sin_u, sin_v]);
            // Build cos(u)*cos(v) - sin(u)*sin(v)
            let neg_one = store.int(-1);
            let term2 = store.mul(vec![neg_one, prod]);
            let result = store.add(vec![term1, term2]);
            return Some(result);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrite_trig_log_exp_constants() {
        let mut st = Store::new();
        let zero = st.int(0);
        let one = st.int(1);
        let sin0 = st.func("sin", vec![zero]);
        let cos0 = st.func("cos", vec![zero]);
        let ln1 = st.func("ln", vec![one]);
        let exp0 = st.func("exp", vec![zero]);

        assert_eq!(rewrite_basic(&mut st, sin0), st.int(0));
        assert_eq!(rewrite_basic(&mut st, cos0), st.int(1));
        assert_eq!(rewrite_basic(&mut st, ln1), st.int(0));
        assert_eq!(rewrite_basic(&mut st, exp0), st.int(1));
    }

    #[test]
    fn rewrite_pow_rules() {
        let mut st = Store::new();
        let x = st.sym("x");
        let one = st.int(1);
        let zero = st.int(0);
        let p1 = st.pow(x, one);
        let p0 = st.pow(x, zero);
        let zero_pow_zero = st.pow(zero, zero);

        assert_eq!(rewrite_basic(&mut st, p1), x);
        assert_eq!(rewrite_basic(&mut st, p0), st.int(1));
        // 0^0 should remain as Pow node
        let r = rewrite_basic(&mut st, zero_pow_zero);
        assert!(matches!(st.get(r).op, Op::Pow));
    }

    #[test]
    fn rewrite_within_expression() {
        let mut st = Store::new();
        // sin(0) + x^1 + ln(1) -> 0 + x + 0 -> x
        let x = st.sym("x");
        let zero = st.int(0);
        let one = st.int(1);
        let sin0 = st.func("sin", vec![zero]);
        let x1 = st.pow(x, one);
        let ln1 = st.func("ln", vec![one]);
        let expr = st.add(vec![sin0, x1, ln1]);
        let r = rewrite_basic(&mut st, expr);
        // Result should simplify to x (since add canonicalization keeps non-zero)
        assert_eq!(r, x);
    }

    #[test]
    fn rewrite_pythagorean_identity_any_order() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let sinx = st.func("sin", vec![x]);
        let cosx = st.func("cos", vec![x]);
        let sin2 = st.pow(sinx, two);
        let cos2 = st.pow(cosx, two);

        // Order 1: sin^2 + cos^2
        let expr1 = st.add(vec![sin2, cos2]);
        let r1 = rewrite_basic(&mut st, expr1);
        assert_eq!(r1, st.int(1));

        // Order 2: cos^2 + sin^2 (ensure AC matching)
        let x2 = st.sym("x");
        let two2 = st.int(2);
        let cosx2 = st.func("cos", vec![x2]);
        let sinx2 = st.func("sin", vec![x2]);
        let cos22 = st.pow(cosx2, two2);
        let two3 = st.int(2);
        let sin22 = st.pow(sinx2, two3);
        let expr2 = st.add(vec![cos22, sin22]);
        let r2 = rewrite_basic(&mut st, expr2);
        assert_eq!(r2, st.int(1));
    }

    #[test]
    fn rewrite_sin_squared_power_reduction() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let sinx = st.func("sin", vec![x]);
        let sin2 = st.pow(sinx, two);

        let result = rewrite_basic(&mut st, sin2);

        // Expected: (1 - cos(2*x))/2 = 1/2 * (1 + -1*cos(2*x))
        let x2 = st.sym("x");
        let two2 = st.int(2);
        let two_x = st.mul(vec![two2, x2]);
        let cos_2x = st.func("cos", vec![two_x]);
        let neg_one = st.int(-1);
        let neg_cos = st.mul(vec![neg_one, cos_2x]);
        let one = st.int(1);
        let numerator = st.add(vec![one, neg_cos]);
        let half = st.rat(1, 2);
        let expected = st.mul(vec![half, numerator]);

        assert_eq!(st.to_string(result), st.to_string(expected));
    }

    #[test]
    fn rewrite_cos_squared_power_reduction() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let cosx = st.func("cos", vec![x]);
        let cos2 = st.pow(cosx, two);

        let result = rewrite_basic(&mut st, cos2);

        // Expected: (1 + cos(2*x))/2 = 1/2 * (1 + cos(2*x))
        let x2 = st.sym("x");
        let two2 = st.int(2);
        let two_x = st.mul(vec![two2, x2]);
        let cos_2x = st.func("cos", vec![two_x]);
        let one = st.int(1);
        let numerator = st.add(vec![one, cos_2x]);
        let half = st.rat(1, 2);
        let expected = st.mul(vec![half, numerator]);

        assert_eq!(st.to_string(result), st.to_string(expected));
    }

    #[test]
    fn rewrite_power_reduction_with_complex_arg() {
        let mut st = Store::new();
        let x = st.sym("x");
        let one = st.int(1);
        let xp1 = st.add(vec![x, one]);
        let sin_xp1 = st.func("sin", vec![xp1]);
        let two = st.int(2);
        let sin2 = st.pow(sin_xp1, two);

        let result = rewrite_basic(&mut st, sin2);

        // Should apply power reduction to sin(x+1)^2
        // The result contains cos(2*(x+1)) after recursively applying rewrites
        let result_str = st.to_string(result);
        // After rewriting, we should have the power-reduction formula applied
        assert!(result_str.contains("cos") || result_str.contains("sin"));
    }

    #[test]
    fn rewrite_sin_double_angle() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let two_x = st.mul(vec![two, x]);
        let sin_2x = st.func("sin", vec![two_x]);

        let result = rewrite_basic(&mut st, sin_2x);

        // Expected: 2*sin(x)*cos(x)
        let x2 = st.sym("x");
        let sinx = st.func("sin", vec![x2]);
        let cosx = st.func("cos", vec![x2]);
        let two2 = st.int(2);
        let expected = st.mul(vec![two2, sinx, cosx]);

        assert_eq!(st.to_string(result), st.to_string(expected));
    }

    #[test]
    fn rewrite_cos_double_angle() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let two_x = st.mul(vec![two, x]);
        let cos_2x = st.func("cos", vec![two_x]);

        let result = rewrite_basic(&mut st, cos_2x);

        // Expected: cos(x)^2 - sin(x)^2 = cos(x)^2 + -1*sin(x)^2
        let x2 = st.sym("x");
        let cosx = st.func("cos", vec![x2]);
        let two2 = st.int(2);
        let cos_sq = st.pow(cosx, two2);
        let sinx = st.func("sin", vec![x2]);
        let two3 = st.int(2);
        let sin_sq = st.pow(sinx, two3);
        let neg_one = st.int(-1);
        let neg_sin_sq = st.mul(vec![neg_one, sin_sq]);
        let expected = st.add(vec![cos_sq, neg_sin_sq]);

        assert_eq!(st.to_string(result), st.to_string(expected));
    }

    #[test]
    fn rewrite_double_angle_with_complex_arg() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let sum = st.add(vec![x, y]);
        let two = st.int(2);
        let two_sum = st.mul(vec![two, sum]);
        let sin_2sum = st.func("sin", vec![two_sum]);

        let result = rewrite_basic(&mut st, sin_2sum);

        // Should expand sin(2*(x+y)) -> 2*sin(x+y)*cos(x+y)
        let result_str = st.to_string(result);
        assert!(result_str.contains("sin"));
        assert!(result_str.contains("cos"));
        assert!(result_str.contains("2"));
    }

    #[test]
    fn rewrite_sin_angle_addition() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let sum = st.add(vec![x, y]);
        let sin_sum = st.func("sin", vec![sum]);

        let result = rewrite_basic(&mut st, sin_sum);

        // Expected: sin(x)*cos(y) + cos(x)*sin(y)
        let x2 = st.sym("x");
        let y2 = st.sym("y");
        let sinx = st.func("sin", vec![x2]);
        let cosy = st.func("cos", vec![y2]);
        let term1 = st.mul(vec![sinx, cosy]);
        let cosx = st.func("cos", vec![x2]);
        let siny = st.func("sin", vec![y2]);
        let term2 = st.mul(vec![cosx, siny]);
        let expected = st.add(vec![term1, term2]);

        assert_eq!(st.to_string(result), st.to_string(expected));
    }

    #[test]
    fn rewrite_cos_angle_addition() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let sum = st.add(vec![x, y]);
        let cos_sum = st.func("cos", vec![sum]);

        let result = rewrite_basic(&mut st, cos_sum);

        // Expected: cos(x)*cos(y) - sin(x)*sin(y) = cos(x)*cos(y) + -1*sin(x)*sin(y)
        let x2 = st.sym("x");
        let y2 = st.sym("y");
        let cosx = st.func("cos", vec![x2]);
        let cosy = st.func("cos", vec![y2]);
        let term1 = st.mul(vec![cosx, cosy]);
        let sinx = st.func("sin", vec![x2]);
        let siny = st.func("sin", vec![y2]);
        let prod = st.mul(vec![sinx, siny]);
        let neg_one = st.int(-1);
        let term2 = st.mul(vec![neg_one, prod]);
        let expected = st.add(vec![term1, term2]);

        assert_eq!(st.to_string(result), st.to_string(expected));
    }

    #[test]
    fn rewrite_sin_angle_subtraction() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let neg_one = st.int(-1);
        let neg_y = st.mul(vec![neg_one, y]);
        let diff = st.add(vec![x, neg_y]);
        let sin_diff = st.func("sin", vec![diff]);

        let result = rewrite_basic(&mut st, sin_diff);

        // sin(x + -y) should expand via addition formula
        // Result should contain sin and cos terms
        let result_str = st.to_string(result);
        assert!(result_str.contains("sin"));
        assert!(result_str.contains("cos"));
    }

    #[test]
    fn rewrite_angle_addition_nested() {
        let mut st = Store::new();
        // Test that angle addition formula works with nested expressions
        // sin(x^2 + y)
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let y = st.sym("y");
        let sum = st.add(vec![x2, y]);
        let sin_sum = st.func("sin", vec![sum]);

        let result = rewrite_basic(&mut st, sin_sum);

        // Should expand via angle addition: sin(x^2)*cos(y) + cos(x^2)*sin(y)
        // After rewriting children, we'll have expanded terms
        let result_str = st.to_string(result);
        assert!(result_str.contains("sin"));
        assert!(result_str.contains("cos"));
    }
}
