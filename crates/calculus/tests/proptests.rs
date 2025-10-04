//! Property-based tests for calculus operations (Phase L)

use calculus::diff;
use expr_core::Store;
use proptest::prelude::*;
use simplify::simplify;

proptest! {
    #[test]
    fn prop_diff_constant(c in -20i64..=20) {
        let mut st = Store::new();
        let ec = st.int(c);
        let deriv = diff(&mut st, ec, "x");
        let zero = st.int(0);
        prop_assert_eq!(deriv, zero);
    }

    #[test]
    fn prop_diff_linear(a in -20i64..=20) {
        let mut st = Store::new();
        let ea = st.int(a);
        let x = st.sym("x");
        let expr = st.mul(vec![ea, x]);
        let deriv = diff(&mut st, expr, "x");
        let simplified = simplify(&mut st, deriv);
        prop_assert_eq!(simplified, ea);
    }

    #[test]
    fn prop_power_rule(n in 1i64..=10) {
        let mut st = Store::new();
        let x = st.sym("x");
        let en = st.int(n);
        let expr = st.pow(x, en);
        let deriv = diff(&mut st, expr, "x");
        let s = st.to_string(deriv);
        prop_assert!(!s.is_empty());
        prop_assert!(s.contains(&n.to_string()) || n == 1);
    }

    #[test]
    fn prop_diff_linear_sum(a in -20i64..=20, b in -20i64..=20) {
        let mut st = Store::new();
        let x = st.sym("x");
        let ea = st.int(a);
        let eb = st.int(b);
        let f = st.mul(vec![ea, x]);
        let g = st.mul(vec![eb, x]);
        let sum = st.add(vec![f, g]);
        let deriv_sum = diff(&mut st, sum, "x");
        let deriv_f = diff(&mut st, f, "x");
        let deriv_g = diff(&mut st, g, "x");
        let sum_deriv = st.add(vec![deriv_f, deriv_g]);
        let s1 = simplify(&mut st, deriv_sum);
        let s2 = simplify(&mut st, sum_deriv);
        prop_assert_eq!(st.get(s1).digest, st.get(s2).digest);
    }
}
