#![deny(warnings)]
//! pattern v1: basic substitution utilities.
//! - Substitute a symbol with an expression throughout a tree.

pub mod ac;
pub mod domain;
pub mod pipeline;
pub mod registry;
pub mod rewrite;
pub mod scheduler;

use expr_core::{ExprId, Op, Payload, Store};

/// Substitute all occurrences of symbol `sym` with `with_expr` inside `id`.
/// Does not recurse into `with_expr` (it is inserted as-is).
/// Results are memoized in the store to avoid redundant computation.
pub fn subst_symbol(store: &mut Store, id: ExprId, sym: &str, with_expr: ExprId) -> ExprId {
    // Check memoization cache first
    if let Some(cached) = store.get_subst_cached(id, sym, with_expr) {
        return cached;
    }

    // Compute the substitution
    let result = subst_symbol_impl(store, id, sym, with_expr);

    // Cache the result before returning
    store.cache_subst(id, sym.to_string(), with_expr, result);
    result
}

/// Internal substitution implementation (without memoization)
fn subst_symbol_impl(store: &mut Store, id: ExprId, sym: &str, with_expr: ExprId) -> ExprId {
    match store.get(id).op {
        Op::Integer | Op::Rational => id,
        Op::Symbol => match &store.get(id).payload {
            Payload::Sym(s) if s == sym => with_expr,
            _ => id,
        },
        Op::Add => {
            let children = store.get(id).children.clone();
            let mapped = children
                .into_iter()
                .map(|c| subst_symbol(store, c, sym, with_expr))
                .collect::<Vec<_>>();
            store.add(mapped)
        }
        Op::Mul => {
            let children = store.get(id).children.clone();
            let mapped = children
                .into_iter()
                .map(|c| subst_symbol(store, c, sym, with_expr))
                .collect::<Vec<_>>();
            store.mul(mapped)
        }
        Op::Pow => {
            let (b_id, e_id) = {
                let n = store.get(id);
                (n.children[0], n.children[1])
            };
            let b = subst_symbol(store, b_id, sym, with_expr);
            let e = subst_symbol(store, e_id, sym, with_expr);
            store.pow(b, e)
        }
        Op::Function => {
            let name = match &store.get(id).payload {
                Payload::Func(s) => s.clone(),
                _ => "<f>".into(),
            };
            let children = store.get(id).children.clone();
            let mapped = children
                .into_iter()
                .map(|c| subst_symbol(store, c, sym, with_expr))
                .collect::<Vec<_>>();
            store.func(name, mapped)
        }
        Op::Piecewise => {
            let children = store.get(id).children.clone();
            let mapped = children
                .into_iter()
                .map(|c| subst_symbol(store, c, sym, with_expr))
                .collect::<Vec<_>>();
            // Rebuild as pairs
            let mut pairs = Vec::new();
            for chunk in mapped.chunks(2) {
                if chunk.len() == 2 {
                    pairs.push((chunk[0], chunk[1]));
                }
            }
            store.piecewise(pairs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subst_in_pow_and_add() {
        let mut st = Store::new();
        let x = st.sym("x");
        let one = st.int(1);
        let xp1 = st.add(vec![x, one]);
        let two = st.int(2);
        let expr = st.pow(xp1, two); // (x+1)^2

        let y = st.sym("y");
        let two2 = st.int(2);
        let y_plus_2 = st.add(vec![y, two2]);
        let out = subst_symbol(&mut st, expr, "x", y_plus_2);
        let out_s = simplify::simplify(&mut st, out);

        // Expected: ((y+2)+1)^2 => (y+3)^2
        let three = st.int(3);
        let y3 = st.add(vec![y, three]);
        let two3 = st.int(2);
        let expected = st.pow(y3, two3);
        assert_eq!(st.get(out_s).digest, st.get(expected).digest);
        assert_eq!(st.to_string(out_s), st.to_string(expected));
    }

    #[test]
    fn subst_noop_when_symbol_absent() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let f = st.mul(vec![two, x]);
        let z = st.sym("z");
        let out = subst_symbol(&mut st, f, "y", z);
        assert_eq!(st.get(out).digest, st.get(f).digest);
        assert_eq!(st.to_string(out), st.to_string(f));
    }

    #[test]
    fn subst_in_function() {
        let mut st = Store::new();
        let x = st.sym("x");
        let sinx = st.func("sin", vec![x]);
        let two = st.int(2);
        let out = subst_symbol(&mut st, sinx, "x", two);
        assert!(st.to_string(out).contains("sin"));
        assert!(st.to_string(out).contains("2"));
    }

    #[test]
    fn subst_memoization() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let two = st.int(2);
        let x2 = st.pow(x, two);

        // First substitution - computes and caches
        let result1 = subst_symbol(&mut st, x2, "x", y);

        // Second substitution - should use cache
        let result2 = subst_symbol(&mut st, x2, "x", y);
        assert_eq!(result1, result2);

        // Different replacement - not cached
        let z = st.sym("z");
        let result3 = subst_symbol(&mut st, x2, "x", z);
        assert_ne!(result1, result3);

        // Clear cache and verify recomputation
        st.clear_caches();
        let result4 = subst_symbol(&mut st, x2, "x", y);
        assert_eq!(result1, result4); // Same result, but recomputed
    }

    #[test]
    fn subst_integer_unchanged() {
        let mut st = Store::new();
        let five = st.int(5);
        let x = st.sym("x");
        let out = subst_symbol(&mut st, five, "y", x);
        assert_eq!(out, five);
    }

    #[test]
    fn subst_rational_unchanged() {
        let mut st = Store::new();
        let half = st.rat(1, 2);
        let x = st.sym("x");
        let out = subst_symbol(&mut st, half, "y", x);
        assert_eq!(out, half);
    }
}
