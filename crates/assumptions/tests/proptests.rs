//! Property-based tests for assumptions

use assumptions::{Context, Prop, Truth};
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_assume_and_has_positive(n in 1usize..=10) {
        let mut ctx = Context::new();
        let var = format!("x{}", n);
        ctx.assume(&var, Prop::Positive);

        let result = ctx.has(&var, Prop::Positive);
        prop_assert_eq!(result, Truth::True);
    }

    #[test]
    fn prop_assume_and_has_real(n in 1usize..=10) {
        let mut ctx = Context::new();
        let var = format!("y{}", n);
        ctx.assume(&var, Prop::Real);

        let result = ctx.has(&var, Prop::Real);
        prop_assert_eq!(result, Truth::True);
    }

    #[test]
    fn prop_unknown_var_returns_unknown(n in 1usize..=10) {
        let ctx = Context::new();
        let var = format!("unknown{}", n);

        let result = ctx.has(&var, Prop::Positive);
        prop_assert_eq!(result, Truth::Unknown);
    }

    #[test]
    fn prop_assume_positive_implies_real(n in 1usize..=10) {
        let mut ctx = Context::new();
        let var = format!("z{}", n);
        ctx.assume(&var, Prop::Positive);

        // Positive numbers are real
        let real_result = ctx.has(&var, Prop::Real);
        prop_assert_eq!(real_result, Truth::True);
    }

    #[test]
    fn prop_context_cloning_preserves_assumptions(n in 1usize..=5) {
        let mut ctx1 = Context::new();
        let var = format!("a{}", n);
        ctx1.assume(&var, Prop::Positive);

        let ctx2 = ctx1.clone();
        let result = ctx2.has(&var, Prop::Positive);
        prop_assert_eq!(result, Truth::True);
    }

    #[test]
    fn prop_positive_implies_nonzero(n in 1usize..=10) {
        let mut ctx = Context::new();
        let var = format!("b{}", n);
        ctx.assume(&var, Prop::Positive);

        // Positive implies nonzero
        let nonzero_result = ctx.has(&var, Prop::Nonzero);
        prop_assert_eq!(nonzero_result, Truth::True);
    }
}
