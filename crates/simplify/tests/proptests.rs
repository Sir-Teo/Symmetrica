//! Property-based tests for simplifier (Phase L)

use expr_core::Store;
use proptest::prelude::*;
use simplify::simplify;

proptest! {
    #[test]
    fn prop_simplify_add_zero(a in -50i64..=50) {
        let mut st = Store::new();
        let ea = st.int(a);
        let zero = st.int(0);
        let expr = st.add(vec![ea, zero]);
        let simplified = simplify(&mut st, expr);
        prop_assert_eq!(simplified, ea);
    }

    #[test]
    fn prop_simplify_mul_one(a in -50i64..=50) {
        let mut st = Store::new();
        let ea = st.int(a);
        let one = st.int(1);
        let expr = st.mul(vec![ea, one]);
        let simplified = simplify(&mut st, expr);
        prop_assert_eq!(simplified, ea);
    }

    #[test]
    fn prop_simplify_mul_zero(a in -50i64..=50) {
        let mut st = Store::new();
        let ea = st.int(a);
        let zero = st.int(0);
        let expr = st.mul(vec![ea, zero]);
        let simplified = simplify(&mut st, expr);
        prop_assert_eq!(simplified, zero);
    }

    #[test]
    fn prop_double_negation(a in -50i64..=50) {
        let mut st = Store::new();
        let ea = st.int(a);
        let neg_one = st.int(-1);
        let neg_a = st.mul(vec![neg_one, ea]);
        let neg_neg_a = st.mul(vec![neg_one, neg_a]);
        let simplified = simplify(&mut st, neg_neg_a);
        prop_assert_eq!(simplified, ea);
    }
}
