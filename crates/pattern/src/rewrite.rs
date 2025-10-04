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
        // Result should contain cos(2*(x+1))
        let result_str = st.to_string(result);
        assert!(result_str.contains("cos"));
        assert!(result_str.contains("1/2"));
    }
}
