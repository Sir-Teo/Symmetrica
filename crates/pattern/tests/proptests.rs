//! Property-based tests for pattern matching and substitution

use expr_core::Store;
use pattern::subst_symbol;
use proptest::prelude::*;

fn small_int() -> impl Strategy<Value = i64> {
    -5i64..=5
}

proptest! {
    #[test]
    fn prop_subst_integer_unchanged(n in small_int()) {
        let mut st = Store::new();
        let expr = st.int(n);
        let new_val = st.int(n + 1);

        let result = subst_symbol(&mut st, expr, "x", new_val);
        // Substituting in an integer should not change it
        prop_assert_eq!(result, expr);
    }

    #[test]
    fn prop_subst_symbol_replaces(_old_val in small_int(), new_val in small_int()) {
        let mut st = Store::new();
        let x = st.sym("x");
        let new_expr = st.int(new_val);

        let result = subst_symbol(&mut st, x, "x", new_expr);
        prop_assert_eq!(result, new_expr);
    }

    #[test]
    fn prop_subst_preserves_other_symbols(n in small_int()) {
        let mut st = Store::new();
        let y = st.sym("y");
        let new_expr = st.int(n);

        // Substituting x with n in expression "y" should leave it unchanged
        let result = subst_symbol(&mut st, y, "x", new_expr);
        prop_assert_eq!(result, y);
    }

    #[test]
    fn prop_subst_in_add(a in small_int(), b in small_int()) {
        let mut st = Store::new();
        let x = st.sym("x");
        let ea = st.int(a);

        // x + a
        let expr = st.add(vec![x, ea]);

        // Substitute x with b
        let new_val = st.int(b);
        let result = subst_symbol(&mut st, expr, "x", new_val);

        // Should be b + a
        let expected = st.add(vec![new_val, ea]);
        prop_assert_eq!(st.get(result).digest, st.get(expected).digest);
    }

    #[test]
    fn prop_subst_in_mul(a in small_int(), b in small_int()) {
        let mut st = Store::new();
        let x = st.sym("x");
        let ea = st.int(a);

        // x * a
        let expr = st.mul(vec![x, ea]);

        // Substitute x with b
        let new_val = st.int(b);
        let result = subst_symbol(&mut st, expr, "x", new_val);

        // Should be b * a
        let expected = st.mul(vec![new_val, ea]);
        prop_assert_eq!(st.get(result).digest, st.get(expected).digest);
    }

    #[test]
    fn prop_subst_in_pow(exp in 1i64..=3, new_base in 1i64..=3) {
        let mut st = Store::new();
        let x = st.sym("x");
        let e_exp = st.int(exp);

        // x^exp
        let expr = st.pow(x, e_exp);

        // Substitute x with new_base
        let new_val = st.int(new_base);
        let result = subst_symbol(&mut st, expr, "x", new_val);

        // Should be new_base^exp
        let expected = st.pow(new_val, e_exp);
        prop_assert_eq!(st.get(result).digest, st.get(expected).digest);
    }

    #[test]
    fn prop_subst_twice_composes(a in small_int()) {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");

        // First substitute x with y
        let step1 = subst_symbol(&mut st, x, "x", y);

        // Then substitute y with a
        let new_val = st.int(a);
        let result = subst_symbol(&mut st, step1, "y", new_val);

        // Should equal directly substituting x with a
        prop_assert_eq!(result, new_val);
    }
}
