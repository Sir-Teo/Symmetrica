#![deny(warnings)]
//! pattern v1: basic substitution utilities.
//! - Substitute a symbol with an expression throughout a tree.

use expr_core::{ExprId, Op, Payload, Store};

/// Substitute all occurrences of symbol `sym` with `with_expr` inside `id`.
/// Does not recurse into `with_expr` (it is inserted as-is).
pub fn subst_symbol(store: &mut Store, id: ExprId, sym: &str, with_expr: ExprId) -> ExprId {
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
}
