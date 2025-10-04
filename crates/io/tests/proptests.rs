//! Property-based tests for io (JSON and S-expression)

use expr_core::Store;
use io::to_latex;
use io::{from_json, from_sexpr, to_json, to_sexpr};
use proptest::prelude::*;

fn small_int() -> impl Strategy<Value = i64> {
    -5i64..=5
}

fn small_nonzero_int() -> impl Strategy<Value = i64> {
    prop_oneof![(-5i64..=-1), (1i64..=5)]
}

fn quadratic_expr(
    st: &mut Store,
    a: i64,
    b: i64,
    c_num: i64,
    c_den: i64,
    n: i64,
) -> expr_core::ExprId {
    let x = st.sym("x");
    let two = st.int(2);
    let x2 = st.pow(x, two);
    let a_int = st.int(a);
    let ax = st.mul(vec![a_int, x]);
    // Build: x^2 + a*x + b + (c_num/c_den) * x^n
    // Avoid creating separate integer and rational constants that will merge after roundtrip
    let b_int = st.int(b);
    let nn = st.int(n.max(0));
    let pow_term = st.pow(x, nn);
    let rat = st.rat(c_num, c_den);
    let scaled_pow = st.mul(vec![rat, pow_term]);
    st.add(vec![x2, ax, b_int, scaled_pow])
}

proptest! {
    #[test]
    fn prop_json_roundtrip_quadratic(a in small_int(), b in small_int(), c_num in small_int(), c_den in small_nonzero_int(), n in 0i64..=5) {
        let mut st = Store::new();
        let expr = quadratic_expr(&mut st, a, b, c_num, c_den, n);
        let s = to_json(&st, expr);
        let mut st2 = Store::new();
        let parsed = from_json(&mut st2, &s).expect("parse json");
        prop_assert_eq!(st.to_string(expr), st2.to_string(parsed));
    }

    #[test]
    fn prop_sexpr_roundtrip_quadratic(a in small_int(), b in small_int(), c_num in small_int(), c_den in small_nonzero_int(), n in 0i64..=5) {
        let mut st = Store::new();
        let expr = quadratic_expr(&mut st, a, b, c_num, c_den, n);
        let s = to_sexpr(&st, expr);
        let mut st2 = Store::new();
        let parsed = from_sexpr(&mut st2, &s).expect("parse sexpr");
        prop_assert_eq!(st.to_string(expr), st2.to_string(parsed));
    }

    #[test]
    fn prop_latex_non_empty(a in small_int(), b in small_int(), c_num in small_int(), c_den in small_nonzero_int()) {
        let mut st = Store::new();
        let expr = quadratic_expr(&mut st, a, b, c_num, c_den, 2);
        let latex = to_latex(&st, expr);
        prop_assert!(!latex.is_empty());
    }
}
