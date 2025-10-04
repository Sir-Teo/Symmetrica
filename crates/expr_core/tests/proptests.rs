//! Property-based tests for expr_core (Phase L)

use expr_core::Store;
use proptest::prelude::*;
use simplify::simplify;

proptest! {
    #[test]
    fn prop_int_stable(n in -100i64..=100) {
        let mut st = Store::new();
        let id1 = st.int(n);
        let id2 = st.int(n);
        prop_assert_eq!(id1, id2);
    }

    #[test]
    fn prop_add_commutative(a in -100i64..=100, b in -100i64..=100) {
        let mut st = Store::new();
        let ea = st.int(a);
        let eb = st.int(b);
        let sum1 = st.add(vec![ea, eb]);
        let sum2 = st.add(vec![eb, ea]);
        prop_assert_eq!(st.get(sum1).digest, st.get(sum2).digest);
    }

    #[test]
    fn prop_distributive(a in -50i64..=50, b in -50i64..=50, c in -50i64..=50) {
        let mut st = Store::new();
        let ea = st.int(a);
        let eb = st.int(b);
        let ec = st.int(c);
        let sum = st.add(vec![eb, ec]);
        let left = st.mul(vec![ea, sum]);
        let term1 = st.mul(vec![ea, eb]);
        let term2 = st.mul(vec![ea, ec]);
        let right = st.add(vec![term1, term2]);
        let left_s = simplify(&mut st, left);
        let right_s = simplify(&mut st, right);
        prop_assert_eq!(st.get(left_s).digest, st.get(right_s).digest);
    }

    #[test]
    fn prop_simplify_idempotent(a in -100i64..=100, b in -100i64..=100) {
        let mut st = Store::new();
        let ea = st.int(a);
        let eb = st.int(b);
        let expr = st.add(vec![ea, eb, ea]);
        let s1 = simplify(&mut st, expr);
        let s2 = simplify(&mut st, s1);
        prop_assert_eq!(st.get(s1).digest, st.get(s2).digest);
    }
}
