#![deny(warnings)]
//! simplify: explicit passes on top of expr_core canonical constructors.
//! v0: recursive simplify; collect-like-terms for Add; basic Pow/Mul cleanups.

use arith::{rat_add, rat_mul};
use assumptions::{Context, Prop, Truth};
use expr_core::{ExprId, Op, Payload, Store};

/// Simplify with a default assumptions context.
pub fn simplify(store: &mut Store, id: ExprId) -> ExprId {
    let ctx = Context::default();
    simplify_with(store, id, &ctx)
}

/// Simplify with an explicit assumptions context.
pub fn simplify_with(store: &mut Store, id: ExprId, ctx: &Context) -> ExprId {
    simplify_rec(store, id, ctx)
}

fn simplify_rec(store: &mut Store, id: ExprId, _ctx: &Context) -> ExprId {
    match store.get(id).op {
        Op::Add => simplify_add(store, id, _ctx),
        Op::Mul => simplify_mul(store, id, _ctx),
        Op::Pow => {
            let (b_id, e_id) = {
                let n = store.get(id);
                (n.children[0], n.children[1])
            };
            let b = simplify_rec(store, b_id, _ctx);
            let e = simplify_rec(store, e_id, _ctx);
            // Guarded and default: (x^2)^(1/2) -> x if x is a positive symbol, otherwise |x|
            if let (Op::Rational, Payload::Rat(n, d)) = (&store.get(e).op, &store.get(e).payload) {
                if *n == 1 && *d == 2 {
                    if let Op::Pow = store.get(b).op {
                        let bb = store.get(b).children[0];
                        let ee = store.get(b).children[1];
                        if matches!(
                            (&store.get(ee).op, &store.get(ee).payload),
                            (Op::Integer, Payload::Int(2))
                        ) {
                            if is_positive_symbol(_ctx, store, bb) {
                                return bb;
                            } else {
                                // Unknown sign: return abs(x)
                                return store.func("abs", vec![bb]);
                            }
                        }
                    }
                }
            }
            store.pow(b, e)
        }
        Op::Function => {
            let name = match &store.get(id).payload {
                Payload::Func(s) => s.clone(),
                _ => "<f>".into(),
            };
            let child_ids = {
                let n = store.get(id);
                n.children.clone()
            };
            let args =
                child_ids.into_iter().map(|c| simplify_rec(store, c, _ctx)).collect::<Vec<_>>();
            // Specific rewrite: ln(exp(u)) -> u
            if name == "ln" && args.len() == 1 {
                let a = args[0];
                if let (Op::Function, Payload::Func(fname)) =
                    (&store.get(a).op, &store.get(a).payload)
                {
                    if fname == "exp" {
                        // exp has one arg by constructor; return its arg
                        let inner = store.get(a).children[0];
                        return inner;
                    }
                }
                // Guarded quotient rule: ln(x * y^-1) -> ln x - ln y when x,y are positive symbols (either factor order)
                if let Op::Mul = store.get(a).op {
                    let factors = store.get(a).children.clone();
                    if factors.len() == 2 {
                        let f0 = factors[0];
                        let f1 = factors[1];
                        // Try (f0, f1) and (f1, f0)
                        let pairs = [(f0, f1), (f1, f0)];
                        for (x_like, y_pow_like) in pairs {
                            if let Op::Pow = store.get(y_pow_like).op {
                                let base_y = store.get(y_pow_like).children[0];
                                let exp_y = store.get(y_pow_like).children[1];
                                if matches!(
                                    (&store.get(exp_y).op, &store.get(exp_y).payload),
                                    (Op::Integer, Payload::Int(-1))
                                ) && is_positive_symbol(_ctx, store, x_like)
                                    && is_positive_symbol(_ctx, store, base_y)
                                {
                                    let ln_x = store.func("ln", vec![x_like]);
                                    let ln_y = store.func("ln", vec![base_y]);
                                    let neg1 = store.int(-1);
                                    let neg_ln_y = store.mul(vec![neg1, ln_y]);
                                    return store.add(vec![ln_x, neg_ln_y]);
                                }
                            }
                        }
                    }
                }
                // Guarded power rule: ln(x^k) -> k * ln(x) when x is positive symbol and k is integer or rational
                if let Op::Pow = store.get(a).op {
                    let base = store.get(a).children[0];
                    let expo = store.get(a).children[1];
                    if is_positive_symbol(_ctx, store, base) {
                        match (&store.get(expo).op, &store.get(expo).payload) {
                            (Op::Integer, _) | (Op::Rational, _) => {
                                let ln_base = store.func("ln", vec![base]);
                                return store.mul(vec![expo, ln_base]);
                            }
                            _ => {}
                        }
                    }
                }
                // Guarded product rule: ln(x*y*...) -> ln x + ln y + ... if all factors are positive symbols
                if let Op::Mul = store.get(a).op {
                    let factors = store.get(a).children.clone();
                    if !factors.is_empty()
                        && factors.iter().all(|&f| is_positive_symbol(_ctx, store, f))
                    {
                        let mut logs: Vec<ExprId> = Vec::with_capacity(factors.len());
                        for &f in &factors {
                            logs.push(store.func("ln", vec![f]));
                        }
                        return store.add(logs);
                    }
                }
            }
            // Guarded rewrite: exp(ln(u)) -> u when u is a positive symbol by assumptions
            if name == "exp" && args.len() == 1 {
                let a = args[0];
                if let (Op::Function, Payload::Func(fname)) =
                    (&store.get(a).op, &store.get(a).payload)
                {
                    if fname == "ln" {
                        let u = store.get(a).children[0];
                        if is_positive_symbol(_ctx, store, u) {
                            return u;
                        }
                    }
                }
            }
            store.func(name, args)
        }
        _ => id,
    }
}

fn is_positive_symbol(ctx: &Context, store: &Store, id: ExprId) -> bool {
    if let (Op::Symbol, Payload::Sym(s)) = (&store.get(id).op, &store.get(id).payload) {
        return matches!(ctx.has(s, Prop::Positive), Truth::True);
    }
    false
}

fn simplify_add(store: &mut Store, id: ExprId, ctx: &Context) -> ExprId {
    // First simplify children
    let child_ids = {
        let n = store.get(id);
        n.children.clone()
    };
    let mut terms = Vec::new();
    for c in child_ids {
        terms.push(simplify_rec(store, c, ctx));
    }
    // Split each term into (coeff, base), then collect coefficients per base
    use std::collections::HashMap;
    let mut map: HashMap<ExprId, (i64, i64)> = HashMap::new(); // base -> rational coeff (num, den)
    for t in terms {
        let (coeff, base) = split_coeff(store, t);
        let entry = map.entry(base).or_insert((0, 1));
        *entry = rat_add(*entry, coeff);
    }

    // Rebuild sum; numeric-only terms are under base==1
    let mut new_terms: Vec<ExprId> = Vec::new();
    for (base, (n, d)) in map {
        if n == 0 {
            continue;
        }
        let term = if is_one(store, base) {
            store.rat(n, d)
        } else if n == 1 && d == 1 {
            base
        } else {
            let coeff = store.rat(n, d);
            store.mul(vec![coeff, base])
        };
        new_terms.push(term);
    }
    if new_terms.is_empty() {
        return store.int(0);
    }
    store.add(new_terms)
}

fn simplify_mul(store: &mut Store, id: ExprId, ctx: &Context) -> ExprId {
    let child_ids = {
        let n = store.get(id);
        n.children.clone()
    };
    let mut factors = Vec::new();
    for c in child_ids {
        factors.push(simplify_rec(store, c, ctx));
    }

    // Merge powers with same base: x^a * x^b -> x^(a+b)
    use std::collections::HashMap;
    let mut exp_map: HashMap<ExprId, ExprId> = HashMap::new();
    let mut passthrough: Vec<ExprId> = Vec::new();
    for f in factors {
        // Skip numeric factors from power-collection (expr_core::mul already folded them)
        let (base, exp_opt) = match (&store.get(f).op, &store.get(f).payload) {
            (Op::Pow, _) => {
                let n = store.get(f);
                (n.children[0], Some(n.children[1]))
            }
            (Op::Integer, _) | (Op::Rational, _) => {
                passthrough.push(f);
                continue;
            }
            _ => (f, Some(store.int(1))),
        };

        if let Some(e) = exp_opt {
            let acc = exp_map.remove(&base).unwrap_or_else(|| store.int(0));
            let sum = store.add(vec![acc, e]);
            // Re-simplify the exponent sum to keep it tidy
            let sum_s = simplify_rec(store, sum, ctx);
            exp_map.insert(base, sum_s);
        } else {
            passthrough.push(f);
        }
    }

    let mut rebuilt: Vec<ExprId> = passthrough;
    for (base, exp) in exp_map {
        // If exponent is 1, just emit the base
        let term = if is_one(store, exp) { base } else { store.pow(base, exp) };
        rebuilt.push(term);
    }
    store.mul(rebuilt)
}

/// Split term into (coeff rational, base expr) where term == coeff * base
fn split_coeff(store: &mut Store, id: ExprId) -> ((i64, i64), ExprId) {
    match (&store.get(id).op, &store.get(id).payload) {
        (Op::Integer, Payload::Int(k)) => (((*k), 1), store.int(1)),
        (Op::Rational, Payload::Rat(n, d)) => (((*n), (*d)), store.int(1)),
        (Op::Mul, _) => {
            let mut coeff = (1i64, 1i64);
            let mut rest: Vec<ExprId> = Vec::new();
            let child_ids = {
                let n = store.get(id);
                n.children.clone()
            };
            for f in child_ids {
                match (&store.get(f).op, &store.get(f).payload) {
                    (Op::Integer, Payload::Int(k)) => {
                        coeff = rat_mul(coeff, (*k, 1));
                    }
                    (Op::Rational, Payload::Rat(n, d)) => {
                        coeff = rat_mul(coeff, (*n, *d));
                    }
                    _ => rest.push(f),
                }
            }
            let base = if rest.is_empty() { store.int(1) } else { store.mul(rest) };
            (coeff, base)
        }
        _ => ((1, 1), id),
    }
}

fn is_one(store: &Store, id: ExprId) -> bool {
    matches!((&store.get(id).op, &store.get(id).payload), (Op::Integer, Payload::Int(1)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idempotent_and_collect_like_terms() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let two_x = st.mul(vec![two, x]);
        let three = st.int(3);
        let three_x = st.mul(vec![three, x]);
        let half = st.rat(1, 2);
        let half_x = st.mul(vec![half, x]);
        let expr = st.add(vec![two_x, three_x, half_x, half]);

        let s1 = simplify(&mut st, expr);
        let s2 = simplify(&mut st, s1);
        assert_eq!(s1, s2, "simplify must be idempotent");

        // Expected: (2+3+1/2)x + 1/2 = (11/2)x + 1/2
        let coeff = st.rat(11, 2);
        let term = st.mul(vec![coeff, x]);
        let half2 = st.rat(1, 2);
        let expected = st.add(vec![term, half2]);
        assert_eq!(s1, expected);
    }

    #[test]
    fn combine_powers_simple() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let p2 = st.pow(x, two);
        let three = st.int(3);
        let p3 = st.pow(x, three);
        let e = st.mul(vec![p2, p3]);
        let s = super::simplify(&mut st, e);
        let five = st.int(5);
        let expected = st.pow(x, five);
        assert_eq!(s, expected);
    }

    #[test]
    fn combine_powers_with_unit_base() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let p2 = st.pow(x, two);
        let e = st.mul(vec![p2, x]);
        let s = super::simplify(&mut st, e);
        let three = st.int(3);
        let expected = st.pow(x, three);
        assert_eq!(s, expected);
    }

    #[test]
    fn combine_powers_and_coefficients() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let three = st.int(3);
        let twoe = st.int(2);
        let p2 = st.pow(x, twoe);
        let threee = st.int(3);
        let p3 = st.pow(x, threee);
        let e = st.mul(vec![two, p2, three, p3]);
        let s = super::simplify(&mut st, e);
        let six = st.int(6);
        let five = st.int(5);
        let px5 = st.pow(x, five);
        let expected = st.mul(vec![six, px5]);
        assert_eq!(s, expected);
    }

    #[test]
    fn cancel_like_terms_to_zero() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let m_two = st.int(-2);
        let two_x = st.mul(vec![two, x]);
        let m_two_x = st.mul(vec![m_two, x]);
        let expr = st.add(vec![two_x, m_two_x]);
        let s = super::simplify(&mut st, expr);
        assert_eq!(s, st.int(0));
    }

    #[test]
    fn combine_multiple_powers_and_plain_factors() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let two = st.int(2);
        let three = st.int(3);
        let p2 = st.pow(x, two);
        let p3 = st.pow(x, three);
        let p2y = st.mul(vec![p2, y]);
        let p3y = st.mul(vec![p3, y]);
        let expr = st.mul(vec![p2y, p3y]);
        let s = super::simplify(&mut st, expr);
        let five = st.int(5);
        let two_e = st.int(2);
        let px5 = st.pow(x, five);
        let y2 = st.pow(y, two_e);
        let expected = st.mul(vec![px5, y2]);
        assert_eq!(s, expected);
    }

    #[test]
    fn fold_numeric_rationals_in_add() {
        let mut st = Store::new();
        let half = st.rat(1, 2);
        let third = st.rat(1, 3);
        let expr = st.add(vec![half, third]);
        let s = super::simplify(&mut st, expr);
        assert_eq!(s, st.rat(5, 6));
    }

    #[test]
    fn simplify_inside_function_arguments() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let zero = st.int(0);
        let one = st.int(1);
        let arg1 = st.add(vec![x, zero]);
        let arg2 = st.mul(vec![one, y]);
        let f = st.func("f", vec![arg1, arg2]);
        let s = super::simplify(&mut st, f);
        let expected = st.func("f", vec![x, y]);
        assert_eq!(s, expected);
    }

    #[test]
    fn ln_exp_simplifies() {
        let mut st = Store::new();
        let x = st.sym("x");
        let one = st.int(1);
        let xp1 = st.add(vec![x, one]);
        let ex = st.func("exp", vec![xp1]);
        let ln_ex = st.func("ln", vec![ex]);
        let s = super::simplify(&mut st, ln_ex);
        let one2 = st.int(1);
        let expected = st.add(vec![x, one2]);
        assert_eq!(s, expected);
    }

    #[test]
    fn exp_ln_simplifies_with_positive_assumption() {
        let mut st = Store::new();
        let x = st.sym("x");
        let ln_x = st.func("ln", vec![x]);
        let ex = st.func("exp", vec![ln_x]);
        let mut ctx = assumptions::Context::new();
        ctx.assume("x", Prop::Positive);
        let s = super::simplify_with(&mut st, ex, &ctx);
        assert_eq!(s, x);
    }

    #[test]
    fn sqrt_x_sq_to_x_with_positive_assumption() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let half = st.rat(1, 2);
        let sqrt_x2 = st.pow(x2, half);
        let mut ctx = assumptions::Context::new();
        ctx.assume("x", Prop::Positive);
        let s = super::simplify_with(&mut st, sqrt_x2, &ctx);
        assert_eq!(s, x);
    }

    #[test]
    fn sqrt_x_sq_to_abs_without_assumption() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let half = st.rat(1, 2);
        let sqrt_x2 = st.pow(x2, half);
        // Without assumptions, sqrt(x^2) should become |x|
        let s = super::simplify(&mut st, sqrt_x2);
        let absx = st.func("abs", vec![x]);
        assert_eq!(s, absx);
    }

    #[test]
    fn ln_quotient_rule_with_positivity() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let m1 = st.int(-1);
        let inv_y = st.pow(y, m1);
        let prod = st.mul(vec![x, inv_y]);
        let ln_expr = st.func("ln", vec![prod]);
        let mut ctx = assumptions::Context::new();
        ctx.assume("x", Prop::Positive);
        ctx.assume("y", Prop::Positive);
        let s = super::simplify_with(&mut st, ln_expr, &ctx);
        let ln_x = st.func("ln", vec![x]);
        let ln_y = st.func("ln", vec![y]);
        let m1b = st.int(-1);
        let neg_ln_y = st.mul(vec![m1b, ln_y]);
        let expected = st.add(vec![ln_x, neg_ln_y]);
        assert_eq!(s, expected);
    }

    #[test]
    fn ln_power_rule_with_positivity() {
        let mut st = Store::new();
        let x = st.sym("x");
        let three = st.int(3);
        let x3 = st.pow(x, three);
        let ln_expr = st.func("ln", vec![x3]);
        let mut ctx = assumptions::Context::new();
        ctx.assume("x", Prop::Positive);
        let s = super::simplify_with(&mut st, ln_expr, &ctx);
        let ln_x = st.func("ln", vec![x]);
        let expected = st.mul(vec![three, ln_x]);
        assert_eq!(st.to_string(s), st.to_string(expected));
    }

    #[test]
    fn ln_product_rule_with_positivity() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let prod = st.mul(vec![x, y]);
        let ln_expr = st.func("ln", vec![prod]);
        let mut ctx = assumptions::Context::new();
        ctx.assume("x", Prop::Positive);
        ctx.assume("y", Prop::Positive);
        let s = super::simplify_with(&mut st, ln_expr, &ctx);
        let ln_x = st.func("ln", vec![x]);
        let ln_y = st.func("ln", vec![y]);
        let expected = st.add(vec![ln_x, ln_y]);
        assert_eq!(st.to_string(s), st.to_string(expected));
    }

    #[test]
    fn simplify_pow_rational_non_matching() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let third = st.rat(1, 3);
        let expr = st.pow(x2, third);
        let s = super::simplify(&mut st, expr);
        // Should not simplify without positivity assumption
        assert!(st.to_string(s).contains("^"));
    }

    #[test]
    fn simplify_unknown_function() {
        let mut st = Store::new();
        let x = st.sym("x");
        let fx = st.func("unknown", vec![x]);
        let s = super::simplify(&mut st, fx);
        assert_eq!(s, fx);
    }
}
