//! Domain-aware rewrites (Phase I start)
//!
//! Safe rules under assumptions::Context (symbol-level properties only):
//! - exp(ln(x)) -> x when x > 0 (x is a Symbol and Context says Positive)
//! - ln(x^n) -> n*ln(x) when x > 0 and n is an integer literal
//! - sqrt(x^2) -> x when x > 0
//!
//! Notes:
//! - We conservatively require `x` to be a single `Symbol` for property checks.
//! - We perform bottom-up rewriting and apply these top-level rules once per call.

use crate::ac::{match_expr, Pat};
use assumptions::{Context as AssumptionsContext, Prop, Truth};
use expr_core::{ExprId, Op, Payload, Store};

pub fn rewrite_domain(store: &mut Store, id: ExprId, ctx: &AssumptionsContext) -> ExprId {
    let rewritten = rewrite_children(store, id, ctx);
    if let Some(out) = try_rules(store, rewritten, ctx) {
        out
    } else {
        rewritten
    }
}

fn rewrite_children(store: &mut Store, id: ExprId, ctx: &AssumptionsContext) -> ExprId {
    match store.get(id).op {
        Op::Add => {
            let ch = store.get(id).children.clone();
            let mut v = Vec::with_capacity(ch.len());
            for c in ch {
                v.push(rewrite_domain(store, c, ctx));
            }
            store.add(v)
        }
        Op::Mul => {
            let ch = store.get(id).children.clone();
            let mut v = Vec::with_capacity(ch.len());
            for c in ch {
                v.push(rewrite_domain(store, c, ctx));
            }
            store.mul(v)
        }
        Op::Pow => {
            let base = store.get(id).children[0];
            let exp = store.get(id).children[1];
            let b = rewrite_domain(store, base, ctx);
            let e = rewrite_domain(store, exp, ctx);
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
                v.push(rewrite_domain(store, c, ctx));
            }
            store.func(name, v)
        }
        _ => id,
    }
}

fn symbol_name(store: &Store, id: ExprId) -> Option<&str> {
    let n = store.get(id);
    if let (Op::Symbol, Payload::Sym(name)) = (&n.op, &n.payload) {
        Some(name.as_str())
    } else {
        None
    }
}

fn is_positive_sym(store: &Store, id: ExprId, ctx: &AssumptionsContext) -> bool {
    if let Some(name) = symbol_name(store, id) {
        matches!(ctx.has(name, Prop::Positive), Truth::True)
    } else {
        false
    }
}

fn try_rules(store: &mut Store, id: ExprId, ctx: &AssumptionsContext) -> Option<ExprId> {
    // Rule: exp(ln(x)) -> x when x>0 (x is a single Symbol known positive)
    {
        let pat = Pat::Function(
            "exp".into(),
            vec![Pat::Function("ln".into(), vec![Pat::Any("x".into())])],
        );
        if let Some(b) = match_expr(store, &pat, id) {
            let x = *b.get("x").unwrap();
            if is_positive_sym(store, x, ctx) {
                return Some(x);
            }
        }
    }

    // Rule: ln(x^n) -> n*ln(x) when x>0 and n is integer
    {
        let pat = Pat::Function(
            "ln".into(),
            vec![Pat::Pow(Box::new(Pat::Any("x".into())), Box::new(Pat::Any("n".into())))],
        );
        if let Some(b) = match_expr(store, &pat, id) {
            let x = *b.get("x").unwrap();
            let n_id = *b.get("n").unwrap();
            // Extract integer exponent value first to end immutable borrow before mutating `store`.
            let k_opt = {
                let nnode = store.get(n_id);
                if let (Op::Integer, Payload::Int(k)) = (&nnode.op, &nnode.payload) {
                    Some(*k)
                } else {
                    None
                }
            };
            if let Some(k) = k_opt {
                if is_positive_sym(store, x, ctx) {
                    let ln_x = store.func("ln", vec![x]);
                    let coeff = store.int(k);
                    return Some(store.mul(vec![coeff, ln_x]));
                }
            }
        }
    }

    // Rule: sqrt(x^2) -> x when x>0
    {
        let pat = Pat::Function(
            "sqrt".into(),
            vec![Pat::Pow(Box::new(Pat::Any("x".into())), Box::new(Pat::Integer(2)))],
        );
        if let Some(b) = match_expr(store, &pat, id) {
            let x = *b.get("x").unwrap();
            if is_positive_sym(store, x, ctx) {
                return Some(x);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exp_ln_positive_rewrites() {
        let mut st = Store::new();
        let mut ctx = AssumptionsContext::new();
        let x = st.sym("x");
        ctx.assume("x", Prop::Positive);
        let lnx = st.func("ln", vec![x]);
        let expr = st.func("exp", vec![lnx]);
        let out = rewrite_domain(&mut st, expr, &ctx);
        assert_eq!(out, x);
    }

    #[test]
    fn exp_ln_without_assumption_no_change() {
        let mut st = Store::new();
        let ctx = AssumptionsContext::new();
        let x = st.sym("x");
        let lnx = st.func("ln", vec![x]);
        let expr = st.func("exp", vec![lnx]);
        let out = rewrite_domain(&mut st, expr, &ctx);
        assert_eq!(out, expr);
    }

    #[test]
    fn ln_pow_to_mul_when_positive() {
        let mut st = Store::new();
        let mut ctx = AssumptionsContext::new();
        let x = st.sym("x");
        ctx.assume("x", Prop::Positive);
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let ln_x2 = st.func("ln", vec![x2]);
        let out = rewrite_domain(&mut st, ln_x2, &ctx);
        // expect 2*ln(x)
        let ln_x = st.func("ln", vec![x]);
        let two2 = st.int(2);
        let expected = st.mul(vec![two2, ln_x]);
        assert_eq!(out, expected);
    }

    #[test]
    fn sqrt_x2_to_x_when_positive() {
        let mut st = Store::new();
        let mut ctx = AssumptionsContext::new();
        let x = st.sym("x");
        ctx.assume("x", Prop::Positive);
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let sqrt_x2 = st.func("sqrt", vec![x2]);
        let out = rewrite_domain(&mut st, sqrt_x2, &ctx);
        assert_eq!(out, x);
    }

    #[test]
    fn sqrt_x2_no_change_without_assumption() {
        let mut st = Store::new();
        let ctx = AssumptionsContext::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let sqrt_x2 = st.func("sqrt", vec![x2]);
        let out = rewrite_domain(&mut st, sqrt_x2, &ctx);
        assert_eq!(out, sqrt_x2); // No change without positivity assumption
    }

    #[test]
    fn ln_pow_no_change_with_rational_exponent() {
        let mut st = Store::new();
        let mut ctx = AssumptionsContext::new();
        let x = st.sym("x");
        ctx.assume("x", Prop::Positive);
        let half = st.rat(1, 2);
        let x_half = st.pow(x, half);
        let ln_x_half = st.func("ln", vec![x_half]);
        let out = rewrite_domain(&mut st, ln_x_half, &ctx);
        // Should not apply rule since exponent is not integer
        assert_eq!(out, ln_x_half);
    }

    #[test]
    fn nested_domain_rewrites() {
        let mut st = Store::new();
        let mut ctx = AssumptionsContext::new();
        let x = st.sym("x");
        ctx.assume("x", Prop::Positive);
        // exp(ln(x)) + exp(ln(x)) with x>0 -> x + x
        let lnx1 = st.func("ln", vec![x]);
        let exp1 = st.func("exp", vec![lnx1]);
        let lnx2 = st.func("ln", vec![x]);
        let exp2 = st.func("exp", vec![lnx2]);
        let expr = st.add(vec![exp1, exp2]);

        let out = rewrite_domain(&mut st, expr, &ctx);
        let expected = st.add(vec![x, x]);
        assert_eq!(out, expected);
    }

    #[test]
    fn domain_rewrite_in_mul() {
        let mut st = Store::new();
        let mut ctx = AssumptionsContext::new();
        let x = st.sym("x");
        ctx.assume("x", Prop::Positive);
        let two = st.int(2);
        // 2 * exp(ln(x))
        let lnx = st.func("ln", vec![x]);
        let exp_lnx = st.func("exp", vec![lnx]);
        let expr = st.mul(vec![two, exp_lnx]);

        let out = rewrite_domain(&mut st, expr, &ctx);
        let expected = st.mul(vec![two, x]);
        assert_eq!(out, expected);
    }

    #[test]
    fn ln_pow_with_negative_exponent() {
        let mut st = Store::new();
        let mut ctx = AssumptionsContext::new();
        let x = st.sym("x");
        ctx.assume("x", Prop::Positive);
        let neg_two = st.int(-2);
        let x_neg2 = st.pow(x, neg_two);
        let ln_expr = st.func("ln", vec![x_neg2]);
        let out = rewrite_domain(&mut st, ln_expr, &ctx);
        // Should produce -2 * ln(x)
        let lnx = st.func("ln", vec![x]);
        let neg_two_2 = st.int(-2);
        let expected = st.mul(vec![neg_two_2, lnx]);
        assert_eq!(out, expected);
    }
}
