//! AC-aware (Add/Mul) pattern matching v1.
//! - Minimal, deterministic matching with wildcards and literals.
//! - Supports Any-variables that bind to whole subexpressions.
//! - For Add/Mul, children are matched as multisets (order-insensitive) with equal arity.
//!
//! This is a first step toward Roadmap Phase H: Pattern Matching v2.

use expr_core::{ExprId, Op, Payload, Store};
use std::collections::HashMap;

/// Wildcard/literal pattern
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Pat {
    /// Bind any subexpression to the given variable name
    Any(String),
    /// Literal symbol with exact name
    Symbol(String),
    /// Literal integer
    Integer(i64),
    /// Literal rational (num, den)
    Rational(i64, i64),
    /// Function with name and ordered argument patterns
    Function(String, Vec<Pat>),
    /// Addition with n children (order-insensitive, equal arity)
    Add(Vec<Pat>),
    /// Multiplication with n children (order-insensitive, equal arity)
    Mul(Vec<Pat>),
    /// Power pattern: base^exp
    Pow(Box<Pat>, Box<Pat>),
}

/// Variable bindings for wildcard variables
pub type Bindings = HashMap<String, ExprId>;

/// Try to match `pat` against expression `expr` under store `st`.
/// Returns a map of wildcard bindings if match succeeds.
pub fn match_expr(st: &Store, pat: &Pat, expr: ExprId) -> Option<Bindings> {
    let mut b = Bindings::new();
    if match_rec(st, pat, expr, &mut b) {
        Some(b)
    } else {
        None
    }
}

fn match_rec(st: &Store, pat: &Pat, expr: ExprId, b: &mut Bindings) -> bool {
    match pat {
        Pat::Any(name) => {
            if let Some(bound) = b.get(name) {
                *bound == expr
            } else {
                b.insert(name.clone(), expr);
                true
            }
        }
        Pat::Symbol(name) => matches_symbol(st, expr, name),
        Pat::Integer(k) => matches_integer(st, expr, *k),
        Pat::Rational(n, d) => matches_rational(st, expr, (*n, *d)),
        Pat::Function(fname, args) => match_function(st, expr, fname, args, b),
        Pat::Pow(pbase, pexp) => match_pow(st, expr, pbase, pexp, b),
        Pat::Add(children) => match_ac(st, expr, Op::Add, children, b),
        Pat::Mul(children) => match_ac(st, expr, Op::Mul, children, b),
    }
}

fn matches_symbol(st: &Store, id: ExprId, name: &str) -> bool {
    let n = st.get(id);
    matches!((&n.op, &n.payload), (Op::Symbol, Payload::Sym(s)) if s == name)
}

fn matches_integer(st: &Store, id: ExprId, k: i64) -> bool {
    let n = st.get(id);
    matches!((&n.op, &n.payload), (Op::Integer, Payload::Int(v)) if *v == k)
}

fn matches_rational(st: &Store, id: ExprId, q: (i64, i64)) -> bool {
    let n = st.get(id);
    matches!((&n.op, &n.payload), (Op::Rational, Payload::Rat(nu, de)) if (*nu, *de) == q)
}

fn match_function(st: &Store, id: ExprId, name: &str, args: &[Pat], b: &mut Bindings) -> bool {
    let n = st.get(id);
    if let (Op::Function, Payload::Func(fname)) = (&n.op, &n.payload) {
        if fname != name || n.children.len() != args.len() {
            return false;
        }
        for (i, ap) in args.iter().enumerate() {
            if !match_rec(st, ap, n.children[i], b) {
                return false;
            }
        }
        true
    } else {
        false
    }
}

fn match_pow(st: &Store, id: ExprId, base: &Pat, exp: &Pat, b: &mut Bindings) -> bool {
    let n = st.get(id);
    if !matches!(n.op, Op::Pow) || n.children.len() != 2 {
        return false;
    }
    let b_ok = match_rec(st, base, n.children[0], b);
    if !b_ok {
        return false;
    }
    match_rec(st, exp, n.children[1], b)
}

/// AC matching for Add/Mul with equal arity.
/// Greedy: tries to match each pattern child to some distinct expression child.
fn match_ac(st: &Store, id: ExprId, op: Op, pats: &[Pat], b: &mut Bindings) -> bool {
    let n = st.get(id);
    if n.op != op || n.children.len() != pats.len() {
        return false;
    }
    let mut used = vec![false; n.children.len()];

    // We clone and try bindings; on failure, revert to snapshot to avoid partial bindings leaking.
    fn try_assign(
        st: &Store,
        pats: &[Pat],
        children: &[ExprId],
        used: &mut [bool],
        b: &mut Bindings,
    ) -> bool {
        if pats.is_empty() {
            return true;
        }
        // Take first pattern and try to match with any unused child
        let (first, rest) = pats.split_first().unwrap();
        let snapshot = b.clone();
        for (i, &child) in children.iter().enumerate() {
            if used[i] {
                continue;
            }
            let mut local_b = snapshot.clone();
            if match_rec(st, first, child, &mut local_b) {
                used[i] = true;
                if try_assign(st, rest, children, used, &mut local_b) {
                    // Commit successful bindings back
                    b.clear();
                    b.extend(local_b);
                    return true;
                }
                used[i] = false;
            }
        }
        false
    }

    try_assign(st, pats, &n.children, &mut used, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_add_commutative_two_terms() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let expr = st.add(vec![y, x]); // out of order on purpose

        let pat = Pat::Add(vec![Pat::Any("a".into()), Pat::Any("b".into())]);
        let b = match_expr(&st, &pat, expr).expect("should match");
        // Check both bindings are present and distinct
        assert_eq!(b.len(), 2);
        let a = b.get("a").unwrap();
        let bb = b.get("b").unwrap();
        assert_ne!(a, bb);
        // one of them is x, the other is y
        let set = [*a, *bb];
        assert!(set.contains(&x) && set.contains(&y));
    }

    #[test]
    fn match_mul_with_pow_any_order() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let three = st.int(3);
        let expr = st.mul(vec![three, x2]); // order swapped

        let pat = Pat::Mul(vec![
            Pat::Pow(Box::new(Pat::Symbol("x".into())), Box::new(Pat::Integer(2))),
            Pat::Any("c".into()),
        ]);
        let b = match_expr(&st, &pat, expr).expect("should match");
        let c = b.get("c").unwrap();
        assert_eq!(*c, three);
    }

    #[test]
    fn match_function_composition() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let sin_x2 = st.func("sin", vec![x2]);

        let pat = Pat::Function("sin".into(), vec![Pat::Any("u".into())]);
        let b = match_expr(&st, &pat, sin_x2).expect("should match");
        assert_eq!(*b.get("u").unwrap(), x2);
    }

    #[test]
    fn match_any_binds_same_variable_consistently() {
        let mut st = Store::new();
        let x = st.sym("x");
        // x + x should match with both "a"s binding to same x
        let expr = st.add(vec![x, x]);
        let pat = Pat::Add(vec![Pat::Any("a".into()), Pat::Any("a".into())]);
        let b = match_expr(&st, &pat, expr).expect("should match");
        assert_eq!(b.len(), 1);
        assert_eq!(*b.get("a").unwrap(), x);
    }

    #[test]
    fn match_any_fails_when_variable_binds_different_values() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let expr = st.add(vec![x, y]);
        // Try to match x + y with a + a (same variable twice)
        let pat = Pat::Add(vec![Pat::Any("a".into()), Pat::Any("a".into())]);
        assert!(match_expr(&st, &pat, expr).is_none());
    }

    #[test]
    fn match_integer_literal() {
        let mut st = Store::new();
        let five = st.int(5);
        let pat = Pat::Integer(5);
        assert!(match_expr(&st, &pat, five).is_some());

        let pat_wrong = Pat::Integer(3);
        assert!(match_expr(&st, &pat_wrong, five).is_none());
    }

    #[test]
    fn match_rational_literal() {
        let mut st = Store::new();
        let half = st.rat(1, 2);
        let pat = Pat::Rational(1, 2);
        assert!(match_expr(&st, &pat, half).is_some());

        let pat_wrong = Pat::Rational(1, 3);
        assert!(match_expr(&st, &pat_wrong, half).is_none());
    }

    #[test]
    fn match_symbol_literal() {
        let mut st = Store::new();
        let x = st.sym("x");
        let pat = Pat::Symbol("x".into());
        assert!(match_expr(&st, &pat, x).is_some());

        let pat_wrong = Pat::Symbol("y".into());
        assert!(match_expr(&st, &pat_wrong, x).is_none());
    }

    #[test]
    fn match_nested_mul_add() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let two = st.int(2);
        // (x + y) * 2
        let sum = st.add(vec![x, y]);
        let expr = st.mul(vec![sum, two]);

        let pat = Pat::Mul(vec![
            Pat::Add(vec![Pat::Any("a".into()), Pat::Any("b".into())]),
            Pat::Integer(2),
        ]);
        let b = match_expr(&st, &pat, expr).expect("should match");
        assert!(b.contains_key("a"));
        assert!(b.contains_key("b"));
    }

    #[test]
    fn match_function_wrong_name() {
        let mut st = Store::new();
        let x = st.sym("x");
        let sinx = st.func("sin", vec![x]);
        let pat = Pat::Function("cos".into(), vec![Pat::Any("u".into())]);
        assert!(match_expr(&st, &pat, sinx).is_none());
    }

    #[test]
    fn match_function_wrong_arity() {
        let mut st = Store::new();
        let x = st.sym("x");
        let f = st.func("f", vec![x]);
        let pat = Pat::Function("f".into(), vec![Pat::Any("a".into()), Pat::Any("b".into())]);
        assert!(match_expr(&st, &pat, f).is_none());
    }

    #[test]
    fn match_pow_pattern() {
        let mut st = Store::new();
        let x = st.sym("x");
        let n = st.int(3);
        let pow_expr = st.pow(x, n);

        let pat = Pat::Pow(Box::new(Pat::Symbol("x".into())), Box::new(Pat::Any("exp".into())));
        let b = match_expr(&st, &pat, pow_expr).expect("should match");
        assert_eq!(*b.get("exp").unwrap(), n);
    }

    #[test]
    fn match_ac_empty_fails() {
        let mut st = Store::new();
        let x = st.sym("x");
        let expr = st.add(vec![x]);
        let pat = Pat::Add(vec![]);
        assert!(match_expr(&st, &pat, expr).is_none());
    }

    #[test]
    fn match_ac_different_arity_fails() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let z = st.sym("z");
        let expr = st.add(vec![x, y, z]);
        let pat = Pat::Add(vec![Pat::Any("a".into()), Pat::Any("b".into())]);
        assert!(match_expr(&st, &pat, expr).is_none());
    }

    #[test]
    fn match_mul_three_terms_commutative() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let two = st.int(2);
        // 2 * x * y
        let expr = st.mul(vec![two, x, y]);

        // Pattern: x * y * 2 (different order)
        let pat = Pat::Mul(vec![Pat::Symbol("x".into()), Pat::Symbol("y".into()), Pat::Integer(2)]);
        assert!(match_expr(&st, &pat, expr).is_some());
    }
}
