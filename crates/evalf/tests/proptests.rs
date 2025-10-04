//! Property-based tests for evalf

use evalf::{eval, EvalContext};
use expr_core::Store;
use proptest::prelude::*;

fn small_int() -> impl Strategy<Value = i64> {
    -10i64..=10
}

fn small_positive_int() -> impl Strategy<Value = i64> {
    1i64..=10
}

proptest! {
    #[test]
    fn prop_eval_integer(n in small_int()) {
        let mut st = Store::new();
        let expr = st.int(n);
        let ctx = EvalContext::new();

        let result = eval(&st, expr, &ctx).expect("eval");
        prop_assert!((result - n as f64).abs() < 1e-10);
    }

    #[test]
    fn prop_eval_addition_associative(a in small_int(), b in small_int(), c in small_int()) {
        let mut st = Store::new();
        let ea = st.int(a);
        let eb = st.int(b);
        let ec = st.int(c);

        // (a + b) + c
        let sum_ab = st.add(vec![ea, eb]);
        let left = st.add(vec![sum_ab, ec]);

        // a + (b + c)
        let sum_bc = st.add(vec![eb, ec]);
        let right = st.add(vec![ea, sum_bc]);

        let ctx = EvalContext::new();
        let left_val = eval(&st, left, &ctx).expect("eval");
        let right_val = eval(&st, right, &ctx).expect("eval");

        prop_assert!((left_val - right_val).abs() < 1e-10);
    }

    #[test]
    fn prop_eval_multiplication_distributive(a in small_int(), b in small_int(), c in small_int()) {
        let mut st = Store::new();
        let ea = st.int(a);
        let eb = st.int(b);
        let ec = st.int(c);

        // a * (b + c)
        let sum = st.add(vec![eb, ec]);
        let left = st.mul(vec![ea, sum]);

        // a * b + a * c
        let prod1 = st.mul(vec![ea, eb]);
        let prod2 = st.mul(vec![ea, ec]);
        let right = st.add(vec![prod1, prod2]);

        let ctx = EvalContext::new();
        let left_val = eval(&st, left, &ctx).expect("eval");
        let right_val = eval(&st, right, &ctx).expect("eval");

        prop_assert!((left_val - right_val).abs() < 1e-10);
    }

    #[test]
    fn prop_eval_power_identity(n in small_positive_int()) {
        let mut st = Store::new();
        let x = st.sym("x");
        let one = st.int(1);
        let expr = st.pow(x, one);

        let mut ctx = EvalContext::new();
        ctx.bind("x", n as f64);

        let result = eval(&st, expr, &ctx).expect("eval");
        prop_assert!((result - n as f64).abs() < 1e-10);
    }

    #[test]
    fn prop_eval_with_binding(n in small_int()) {
        let mut st = Store::new();
        let x = st.sym("x");

        let mut ctx = EvalContext::new();
        ctx.bind("x", n as f64);

        let result = eval(&st, x, &ctx).expect("eval");
        prop_assert!((result - n as f64).abs() < 1e-10);
    }

    #[test]
    fn prop_eval_rational(num in small_int(), den in small_positive_int()) {
        let mut st = Store::new();
        let expr = st.rat(num, den);
        let ctx = EvalContext::new();

        let result = eval(&st, expr, &ctx).expect("eval");
        let expected = num as f64 / den as f64;
        prop_assert!((result - expected).abs() < 1e-10);
    }
}
