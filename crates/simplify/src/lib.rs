#![deny(warnings)]
//! simplify: explicit passes on top of expr_core canonical constructors.
//! v0: recursive simplify; collect-like-terms for Add; basic Pow/Mul cleanups.

use assumptions::Context;
use expr_core::{ExprId, Op, Payload, Store};

/// Simplify with a default assumptions context.
pub fn simplify(store: &mut Store, id: ExprId) -> ExprId {
    let ctx = Context;
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
            store.func(name, args)
        }
        _ => id,
    }
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

// Local rational ops (mirror expr_core helpers)
fn gcd_i64(mut a: i64, mut b: i64) -> i64 {
    if a == 0 {
        return b.abs();
    }
    if b == 0 {
        return a.abs();
    }
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a.abs()
}
fn normalize_rat(num: i64, den: i64) -> (i64, i64) {
    let mut n = num;
    let mut d = den;
    if d < 0 {
        n = -n;
        d = -d;
    }
    if n == 0 {
        return (0, 1);
    }
    let g = gcd_i64(n.abs(), d);
    (n / g, d / g)
}
fn rat_add(a: (i64, i64), b: (i64, i64)) -> (i64, i64) {
    normalize_rat(a.0 * b.1 + b.0 * a.1, a.1 * b.1)
}
fn rat_mul(a: (i64, i64), b: (i64, i64)) -> (i64, i64) {
    normalize_rat(a.0 * b.0, a.1 * b.1)
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
}
