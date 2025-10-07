#![deny(warnings)]
//! simplify: explicit passes on top of expr_core canonical constructors.
//! v0: recursive simplify; collect-like-terms for Add; basic Pow/Mul cleanups.
//! v2.0 (Phase 6): Advanced simplification system
//!   - Trigonometric identities (sum-to-product, product-to-sum, half-angle)
//!   - Radical simplification (denesting, rationalization, perfect powers)
//!   - Logarithm expansion/contraction with branch-cut awareness

mod log_simplify;
mod radical_simplify;
mod trig_identities;

pub use log_simplify::{contract_logarithms, simplify_logarithms};
pub use radical_simplify::simplify_radicals;
pub use trig_identities::simplify_trig;

use arith::{rat_add, rat_mul};
use assumptions::{Context, Prop, Truth};
use expr_core::{ExprId, Op, Payload, Store};

/// Simplify with a default assumptions context.
/// Results are memoized in the store to avoid redundant computation.
pub fn simplify(store: &mut Store, id: ExprId) -> ExprId {
    let ctx = Context::default();
    simplify_with(store, id, &ctx)
}

/// Simplify with an explicit assumptions context.
/// Note: Memoization currently only works for default context simplification.
/// When using a custom context, results are not cached to avoid incorrect cache hits.
pub fn simplify_with(store: &mut Store, id: ExprId, ctx: &Context) -> ExprId {
    // Only use cache for default context to avoid incorrect cached results
    // when different assumption contexts are used
    if ctx.is_default() {
        if let Some(cached) = store.get_simplify_cached(id) {
            return cached;
        }
        let result = simplify_full(store, id, ctx);
        store.cache_simplify(id, result);
        result
    } else {
        simplify_full(store, id, ctx)
    }
}

/// Full simplification pipeline: basic + advanced passes
fn simplify_full(store: &mut Store, id: ExprId, ctx: &Context) -> ExprId {
    // Phase 1: Basic simplification (canonical forms, like-term collection)
    let after_basic = simplify_rec(store, id, ctx);

    // Phase 2: Advanced passes (iteratively until fixpoint or max iterations)
    let mut current = after_basic;
    let max_iterations = 3; // Prevent infinite loops

    for _ in 0..max_iterations {
        let before = current;

        // Apply advanced simplifiers in sequence
        // First try calculus-specific simplifiers (includes Pythagorean identity)
        current = apply_calculus_simplify(store, current);
        current = simplify_trig(store, current);
        current = simplify_radicals(store, current);
        current = simplify_logarithms(store, current, ctx);

        // Recursively simplify to catch nested patterns
        current = simplify_rec(store, current, ctx);

        // Check for fixpoint
        if current == before {
            break;
        }
    }

    current
}

/// Apply calculus-specific simplification (Pythagorean, double-angle, hyperbolic)
/// This recursively traverses the expression tree
fn apply_calculus_simplify(store: &mut Store, expr: ExprId) -> ExprId {
    // First recurse into children
    let expr_after_children = match store.get(expr).op {
        Op::Add | Op::Mul => {
            let children = store.get(expr).children.clone();
            let simplified_children: Vec<ExprId> =
                children.iter().map(|&c| apply_calculus_simplify(store, c)).collect::<Vec<_>>();

            // Early exit if children unchanged
            if simplified_children.iter().zip(children.iter()).all(|(a, b)| a == b) {
                expr
            } else {
                match store.get(expr).op {
                    Op::Add => store.add(simplified_children),
                    Op::Mul => store.mul(simplified_children),
                    _ => unreachable!(),
                }
            }
        }
        Op::Pow => {
            let children = store.get(expr).children.clone();
            let base = apply_calculus_simplify(store, children[0]);
            let exp = apply_calculus_simplify(store, children[1]);

            // Early exit if unchanged
            if base == children[0] && exp == children[1] {
                expr
            } else {
                store.pow(base, exp)
            }
        }
        Op::Function => {
            let name = match &store.get(expr).payload {
                Payload::Func(s) => s.clone(),
                _ => return expr,
            };
            let children = store.get(expr).children.clone();
            let simplified_children: Vec<ExprId> =
                children.iter().map(|&c| apply_calculus_simplify(store, c)).collect::<Vec<_>>();

            // Early exit if children unchanged
            if simplified_children.iter().zip(children.iter()).all(|(a, b)| a == b) {
                expr
            } else {
                store.func(name, simplified_children)
            }
        }
        _ => expr,
    };

    // Then apply Pythagorean identity at this level
    apply_pythagorean_identity(store, expr_after_children)
}

/// Inline Pythagorean identity: sin²(x) + cos²(x) → 1
/// Handles Add nodes with multiple children
fn apply_pythagorean_identity(store: &mut Store, expr: ExprId) -> ExprId {
    if store.get(expr).op != Op::Add {
        return expr;
    }

    let children = store.get(expr).children.clone();

    // Look for matching sin²/cos² pairs in the children
    let mut remaining = children.clone();
    let mut found_match = false;

    'outer: for i in 0..children.len() {
        if !is_sin_squared(store, children[i]) {
            continue;
        }
        let sin_arg = store.get(store.get(children[i]).children[0]).children[0];

        // Look for matching cos² term
        for j in 0..children.len() {
            if i == j {
                continue;
            }
            if !is_cos_squared(store, children[j]) {
                continue;
            }
            let cos_arg = store.get(store.get(children[j]).children[0]).children[0];

            if sin_arg == cos_arg {
                // Found a match! Remove both terms and add 1
                remaining.retain(|&id| id != children[i] && id != children[j]);
                remaining.push(store.int(1));
                found_match = true;
                break 'outer;
            }
        }
    }

    if found_match {
        if remaining.is_empty() {
            return store.int(1);
        }
        if remaining.len() == 1 {
            return remaining[0];
        }
        // Recursively apply in case there are more pairs
        let new_expr = store.add(remaining);
        return apply_pythagorean_identity(store, new_expr);
    }

    expr
}

fn is_sin_squared(store: &Store, expr: ExprId) -> bool {
    if store.get(expr).op != Op::Pow {
        return false;
    }
    let children = &store.get(expr).children;
    if children.len() != 2 {
        return false;
    }
    let base = children[0];
    let exp = children[1];

    matches!((&store.get(exp).op, &store.get(exp).payload), (Op::Integer, Payload::Int(2)))
        && store.get(base).op == Op::Function
        && matches!(&store.get(base).payload, Payload::Func(name) if name == "sin")
}

fn is_cos_squared(store: &Store, expr: ExprId) -> bool {
    if store.get(expr).op != Op::Pow {
        return false;
    }
    let children = &store.get(expr).children;
    if children.len() != 2 {
        return false;
    }
    let base = children[0];
    let exp = children[1];

    matches!((&store.get(exp).op, &store.get(exp).payload), (Op::Integer, Payload::Int(2)))
        && store.get(base).op == Op::Function
        && matches!(&store.get(base).payload, Payload::Func(name) if name == "cos")
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
            // Domain-aware: (x^2)^(1/2) -> x if x>=0, |x| if real, sqrt(x^2) otherwise
            if let (Op::Rational, Payload::Rat(n, d)) = (&store.get(e).op, &store.get(e).payload) {
                if *n == 1 && *d == 2 {
                    if let Op::Pow = store.get(b).op {
                        let bb = store.get(b).children[0];
                        let ee = store.get(b).children[1];
                        if matches!(
                            (&store.get(ee).op, &store.get(ee).payload),
                            (Op::Integer, Payload::Int(2))
                        ) {
                            // If nonnegative (includes positive), sqrt(x^2) = x
                            if is_nonnegative_symbol(_ctx, store, bb) {
                                return bb;
                            } else if is_real_symbol(_ctx, store, bb) {
                                // If real but sign unknown, sqrt(x^2) = |x|
                                return store.func("abs", vec![bb]);
                            }
                            // Complex or unknown domain: leave as sqrt(x^2)
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
        Op::Piecewise => {
            let child_ids = {
                let n = store.get(id);
                n.children.clone()
            };
            // Simplify all conditions and values
            let simplified: Vec<ExprId> =
                child_ids.into_iter().map(|c| simplify_rec(store, c, _ctx)).collect();

            // Try to collapse: if a condition is known to be true, return its value
            for chunk in simplified.chunks(2) {
                if chunk.len() == 2 {
                    let cond = chunk[0];
                    let val = chunk[1];

                    // Check if condition evaluates to true
                    if is_true_condition(store, cond, _ctx) {
                        return val;
                    }
                }
            }

            // Rebuild piecewise with simplified children, filtering out false conditions
            let mut filtered_pairs = Vec::new();
            for chunk in simplified.chunks(2) {
                if chunk.len() == 2 {
                    let cond = chunk[0];
                    let val = chunk[1];
                    // Skip pairs with definitely false conditions
                    if !is_false_condition(store, cond) {
                        filtered_pairs.push((cond, val));
                    }
                }
            }

            if filtered_pairs.is_empty() {
                // No valid branches - undefined
                return store.func("Undefined", vec![]);
            }
            if filtered_pairs.len() == 1 {
                // Only one branch left, check if condition is always-true placeholder
                let (cond, val) = filtered_pairs[0];
                if is_true_condition(store, cond, _ctx) {
                    return val;
                }
            }
            store.piecewise(filtered_pairs)
        }
        _ => id,
    }
}

/// Check if a condition expression evaluates to true
fn is_true_condition(store: &Store, cond: ExprId, _ctx: &Context) -> bool {
    // Check for explicit True function
    if let (Op::Function, Payload::Func(name)) = (&store.get(cond).op, &store.get(cond).payload) {
        if name == "True" {
            return true;
        }
    }
    // Check for literal integer 1
    if matches!((&store.get(cond).op, &store.get(cond).payload), (Op::Integer, Payload::Int(1))) {
        return true;
    }
    false
}

/// Check if a condition expression evaluates to false
fn is_false_condition(store: &Store, cond: ExprId) -> bool {
    // Check for explicit False function
    if let (Op::Function, Payload::Func(name)) = (&store.get(cond).op, &store.get(cond).payload) {
        if name == "False" {
            return true;
        }
    }
    // Check for literal integer 0
    if matches!((&store.get(cond).op, &store.get(cond).payload), (Op::Integer, Payload::Int(0))) {
        return true;
    }
    false
}

fn is_positive_symbol(ctx: &Context, store: &Store, id: ExprId) -> bool {
    if let (Op::Symbol, Payload::Sym(s)) = (&store.get(id).op, &store.get(id).payload) {
        return matches!(ctx.has(s, Prop::Positive), Truth::True);
    }
    false
}

fn is_nonnegative_symbol(ctx: &Context, store: &Store, id: ExprId) -> bool {
    if let (Op::Symbol, Payload::Sym(s)) = (&store.get(id).op, &store.get(id).payload) {
        return matches!(ctx.has(s, Prop::Nonnegative), Truth::True);
    }
    false
}

fn is_real_symbol(ctx: &Context, store: &Store, id: ExprId) -> bool {
    if let (Op::Symbol, Payload::Sym(s)) = (&store.get(id).op, &store.get(id).payload) {
        return matches!(ctx.has(s, Prop::Real), Truth::True);
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

    // Flatten nested Mul nodes before power collection
    // This ensures that Mul[x, Mul[2, x]] becomes [x, 2, x]
    let mut flattened = Vec::new();
    for f in factors {
        if matches!(store.get(f).op, Op::Mul) {
            // Extract children of nested Mul
            let nested_children = store.get(f).children.clone();
            flattened.extend(nested_children);
        } else {
            flattened.push(f);
        }
    }

    // Merge powers with same base: x^a * x^b -> x^(a+b)
    use std::collections::HashMap;
    let mut exp_map: HashMap<ExprId, ExprId> = HashMap::new();
    let mut passthrough: Vec<ExprId> = Vec::new();
    for f in flattened {
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
        // Phase I: Without domain assumptions, sqrt(x^2) stays unchanged
        // (could be complex domain, so unsafe to simplify)
        let s = super::simplify(&mut st, sqrt_x2);
        // Compare structure rather than ExprId (hash-consing may rebuild)
        assert_eq!(st.to_string(s), st.to_string(sqrt_x2));
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

    // ========== Phase I: Domain-Aware Simplification Tests ==========

    #[test]
    fn sqrt_x_sq_to_x_with_nonnegative() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let half = st.rat(1, 2);
        let sqrt_x2 = st.pow(x2, half);

        let mut ctx = assumptions::Context::new();
        ctx.assume("x", Prop::Nonnegative);
        let s = super::simplify_with(&mut st, sqrt_x2, &ctx);

        // Should simplify to x (not |x|) when nonnegative
        assert_eq!(s, x);
    }

    #[test]
    fn sqrt_x_sq_to_abs_with_real() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let half = st.rat(1, 2);
        let sqrt_x2 = st.pow(x2, half);

        let mut ctx = assumptions::Context::new();
        ctx.assume("x", Prop::Real);
        let s = super::simplify_with(&mut st, sqrt_x2, &ctx);

        // Should simplify to |x| when real but sign unknown
        let abs_x = st.func("abs", vec![x]);
        assert_eq!(s, abs_x);
    }

    #[test]
    fn sqrt_x_sq_unchanged_without_assumptions() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let half = st.rat(1, 2);
        let sqrt_x2 = st.pow(x2, half);

        let ctx = assumptions::Context::new();
        let s = super::simplify_with(&mut st, sqrt_x2, &ctx);

        // Should leave as sqrt(x^2) when domain unknown (could be complex)
        // Compare structure rather than ExprId (hash-consing may rebuild)
        assert_eq!(st.to_string(s), st.to_string(sqrt_x2));
    }

    #[test]
    fn negative_implies_real_and_nonzero() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let half = st.rat(1, 2);
        let sqrt_x2 = st.pow(x2, half);

        let mut ctx = assumptions::Context::new();
        ctx.assume("x", Prop::Negative);
        let s = super::simplify_with(&mut st, sqrt_x2, &ctx);

        // Negative implies Real, so should get |x|
        let abs_x = st.func("abs", vec![x]);
        assert_eq!(s, abs_x);
    }

    #[test]
    fn positive_implies_nonnegative() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let half = st.rat(1, 2);
        let sqrt_x2 = st.pow(x2, half);

        let mut ctx = assumptions::Context::new();
        ctx.assume("x", Prop::Positive);
        let s = super::simplify_with(&mut st, sqrt_x2, &ctx);

        // Positive implies Nonnegative, so should simplify to x
        assert_eq!(s, x);
    }

    #[test]
    fn nonnegative_nonzero_implies_positive() {
        let mut ctx = assumptions::Context::new();
        ctx.assume("x", Prop::Nonnegative);
        ctx.assume("x", Prop::Nonzero);

        // Should derive Positive from Nonnegative + Nonzero
        assert!(matches!(ctx.has("x", Prop::Positive), Truth::True));
    }

    #[test]
    fn domain_aware_ln_still_works() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let prod = st.mul(vec![x, y]);
        let ln_expr = st.func("ln", vec![prod]);

        let mut ctx = assumptions::Context::new();
        ctx.assume("x", Prop::Nonnegative);
        ctx.assume("x", Prop::Nonzero); // Nonnegative + Nonzero = Positive
        ctx.assume("y", Prop::Positive);

        let s = super::simplify_with(&mut st, ln_expr, &ctx);
        let ln_x = st.func("ln", vec![x]);
        let ln_y = st.func("ln", vec![y]);
        let expected = st.add(vec![ln_x, ln_y]);
        assert_eq!(st.to_string(s), st.to_string(expected));
    }

    // ========== Phase I: Piecewise Tests ==========

    #[test]
    fn piecewise_simplify_true_branch() {
        let mut st = Store::new();
        let x = st.sym("x");
        let true_cond = st.func("True", vec![]);
        let false_cond = st.func("False", vec![]);
        let zero = st.int(0);

        // piecewise((True, x), (False, 0))
        let pw = st.piecewise(vec![(true_cond, x), (false_cond, zero)]);
        let s = super::simplify(&mut st, pw);

        // Should collapse to x (first true branch)
        assert_eq!(s, x);
    }

    #[test]
    fn piecewise_filter_false_branches() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let false_cond = st.func("False", vec![]);
        let true_cond = st.func("True", vec![]);

        // piecewise((False, x), (True, y))
        let pw = st.piecewise(vec![(false_cond, x), (true_cond, y)]);
        let s = super::simplify(&mut st, pw);

        // Should skip false branch and return y
        assert_eq!(s, y);
    }

    #[test]
    fn piecewise_with_integer_conditions() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let one = st.int(1);
        let zero = st.int(0);

        // piecewise((0, x), (1, y)) - 0 is false, 1 is true
        let pw = st.piecewise(vec![(zero, x), (one, y)]);
        let s = super::simplify(&mut st, pw);

        // Should return y (1 is true)
        assert_eq!(s, y);
    }

    #[test]
    fn piecewise_simplify_values() {
        let mut st = Store::new();
        let x = st.sym("x");
        let true_cond = st.func("True", vec![]);

        // Value that needs simplification: x + x
        let val = st.add(vec![x, x]);
        let pw = st.piecewise(vec![(true_cond, val)]);
        let s = super::simplify(&mut st, pw);

        // Should collapse to a simplified form: x + x → 2 * x
        assert_eq!(st.to_string(s), "2 * x");
    }

    #[test]
    fn piecewise_no_true_branch_remains_piecewise() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let cond = st.func("P", vec![x]); // Unknown condition

        // piecewise((P(x), y))
        let pw = st.piecewise(vec![(cond, y)]);
        let s = super::simplify(&mut st, pw);

        // Should remain as piecewise since condition is unknown
        assert!(matches!(st.get(s).op, Op::Piecewise));
    }

    #[test]
    fn piecewise_all_false_becomes_undefined() {
        let mut st = Store::new();
        let x = st.sym("x");
        let false_cond = st.func("False", vec![]);

        // piecewise((False, x))
        let pw = st.piecewise(vec![(false_cond, x)]);
        let s = super::simplify(&mut st, pw);

        // Should become Undefined
        assert!(matches!(st.get(s).op, Op::Function));
        if let Payload::Func(name) = &st.get(s).payload {
            assert_eq!(name, "Undefined");
        }
    }

    #[test]
    fn piecewise_with_true_catchall_collapses() {
        let mut st = Store::new();
        let x = st.sym("x");
        let zero = st.int(0);
        let neg_one = st.int(-1);
        let neg_x = st.mul(vec![neg_one, x]);

        // piecewise((x >= 0, x), (True, -x))
        let cond = st.func(">=", vec![x, zero]);
        let true_cond = st.func("True", vec![]);
        let pw = st.piecewise(vec![(cond, x), (true_cond, neg_x)]);

        let s = super::simplify(&mut st, pw);

        // With True as catch-all, it collapses to -x (since True is detected as true)
        assert_eq!(s, neg_x);
    }

    #[test]
    fn piecewise_abs_with_unknown_conditions_remains() {
        let mut st = Store::new();
        let x = st.sym("x");
        let zero = st.int(0);
        let neg_one = st.int(-1);
        let neg_x = st.mul(vec![neg_one, x]);

        // abs(x) = piecewise((x >= 0, x), (else, -x)) - using unknown "else" condition
        let cond1 = st.func(">=", vec![x, zero]);
        let cond2 = st.func("else", vec![]); // Unknown condition, not True
        let abs_impl = st.piecewise(vec![(cond1, x), (cond2, neg_x)]);

        let s = super::simplify(&mut st, abs_impl);

        // Should remain as piecewise since conditions are unknown
        assert!(matches!(st.get(s).op, Op::Piecewise));
    }

    #[test]
    fn piecewise_nested_simplification() {
        let mut st = Store::new();
        let x = st.sym("x");
        let true_cond = st.func("True", vec![]);

        // Nested: outer piecewise with inner piecewise value
        let inner = st.piecewise(vec![(true_cond, x)]);
        let outer = st.piecewise(vec![(true_cond, inner)]);

        let s = super::simplify(&mut st, outer);

        // Should fully collapse to x
        assert_eq!(s, x);
    }

    #[test]
    fn piecewise_propagate_assumptions_through_branches() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let half = st.rat(1, 2);
        let sqrt_x2 = st.pow(x2, half);

        let true_cond = st.func("True", vec![]);

        // piecewise((True, sqrt(x^2))) with x positive
        let pw = st.piecewise(vec![(true_cond, sqrt_x2)]);

        let mut ctx = assumptions::Context::new();
        ctx.assume("x", Prop::Positive);
        let s = super::simplify_with(&mut st, pw, &ctx);

        // Value should simplify to x, then piecewise collapses
        assert_eq!(s, x);
    }

    #[test]
    fn piecewise_empty_handled() {
        let mut st = Store::new();
        let pw = st.piecewise(vec![]);
        let s = super::simplify(&mut st, pw);

        // Empty piecewise becomes Undefined
        assert!(matches!(st.get(s).op, Op::Function));
    }
}
