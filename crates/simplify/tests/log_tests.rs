//! Integration tests for Phase 6 logarithm simplification

use assumptions::{Context, Prop};
use expr_core::{Op, Store};
use simplify::{contract_logarithms, simplify_logarithms};

#[test]
fn test_expand_log_product_with_positivity() {
    let mut st = Store::new();
    let mut ctx = Context::new();
    let x = st.sym("x");
    let y = st.sym("y");
    ctx.assume("x", Prop::Positive);
    ctx.assume("y", Prop::Positive);

    let product = st.mul(vec![x, y]);
    let ln_xy = st.func("ln", vec![product]);

    let result = simplify_logarithms(&mut st, ln_xy, &ctx);

    // Should expand to ln(x) + ln(y)
    assert_eq!(st.get(result).op, Op::Add);
    let add_children = &st.get(result).children;
    assert_eq!(add_children.len(), 2);
}

#[test]
fn test_expand_log_power_with_positivity() {
    let mut st = Store::new();
    let mut ctx = Context::new();
    let x = st.sym("x");
    ctx.assume("x", Prop::Positive);

    let five = st.int(5);
    let x5 = st.pow(x, five);
    let ln_x5 = st.func("ln", vec![x5]);

    let result = simplify_logarithms(&mut st, ln_x5, &ctx);

    // Should expand to 5*ln(x)
    assert_eq!(st.get(result).op, Op::Mul);
    let mul_children = &st.get(result).children;
    assert_eq!(mul_children.len(), 2);
}

#[test]
fn test_no_expand_without_positivity() {
    let mut st = Store::new();
    let ctx = Context::new(); // No assumptions
    let x = st.sym("x");
    let y = st.sym("y");

    let product = st.mul(vec![x, y]);
    let ln_xy = st.func("ln", vec![product]);

    let result = simplify_logarithms(&mut st, ln_xy, &ctx);

    // Should NOT expand (x, y not known to be positive)
    assert_eq!(result, ln_xy);
}

#[test]
fn test_expand_log_quotient() {
    let mut st = Store::new();
    let mut ctx = Context::new();
    let x = st.sym("x");
    let y = st.sym("y");
    ctx.assume("x", Prop::Positive);
    ctx.assume("y", Prop::Positive);

    // ln(x/y) = ln(x * y^(-1))
    let neg_one = st.int(-1);
    let y_inv = st.pow(y, neg_one);
    let quotient = st.mul(vec![x, y_inv]);
    let ln_quot = st.func("ln", vec![quotient]);

    let result = simplify_logarithms(&mut st, ln_quot, &ctx);

    // Should expand to ln(x) + (-1)*ln(y) = ln(x) - ln(y)
    assert_eq!(st.get(result).op, Op::Add);
}

#[test]
fn test_expand_log_triple_product() {
    let mut st = Store::new();
    let mut ctx = Context::new();
    let x = st.sym("x");
    let y = st.sym("y");
    let z = st.sym("z");
    ctx.assume("x", Prop::Positive);
    ctx.assume("y", Prop::Positive);
    ctx.assume("z", Prop::Positive);

    let product = st.mul(vec![x, y, z]);
    let ln_xyz = st.func("ln", vec![product]);

    let result = simplify_logarithms(&mut st, ln_xyz, &ctx);

    // Should expand to ln(x) + ln(y) + ln(z)
    assert_eq!(st.get(result).op, Op::Add);
    let add_children = &st.get(result).children;
    assert_eq!(add_children.len(), 3);
}

#[test]
fn test_expand_log_rational_power() {
    let mut st = Store::new();
    let mut ctx = Context::new();
    let x = st.sym("x");
    ctx.assume("x", Prop::Positive);

    let half = st.rat(1, 2);
    let sqrt_x = st.pow(x, half);
    let ln_sqrt_x = st.func("ln", vec![sqrt_x]);

    let result = simplify_logarithms(&mut st, ln_sqrt_x, &ctx);

    // Should expand to (1/2)*ln(x)
    assert_eq!(st.get(result).op, Op::Mul);
}

#[test]
fn test_contract_log_sum() {
    let mut st = Store::new();
    let ctx = Context::new();
    let x = st.sym("x");
    let y = st.sym("y");

    let ln_x = st.func("ln", vec![x]);
    let ln_y = st.func("ln", vec![y]);
    let sum = st.add(vec![ln_x, ln_y]);

    let result = contract_logarithms(&mut st, sum, &ctx);

    // Should contract to ln(x*y)
    assert_eq!(st.get(result).op, Op::Function);
}

#[test]
fn test_contract_log_difference() {
    let mut st = Store::new();
    let ctx = Context::new();
    let x = st.sym("x");
    let y = st.sym("y");

    let ln_x = st.func("ln", vec![x]);
    let ln_y = st.func("ln", vec![y]);
    let neg_one = st.int(-1);
    let neg_ln_y = st.mul(vec![neg_one, ln_y]);
    let diff = st.add(vec![ln_x, neg_ln_y]);

    let result = contract_logarithms(&mut st, diff, &ctx);

    // Should contract to ln(x/y) = ln(x * y^(-1))
    assert_eq!(st.get(result).op, Op::Function);
}

#[test]
fn test_contract_scaled_log() {
    let mut st = Store::new();
    let ctx = Context::new();
    let x = st.sym("x");
    let y = st.sym("y");

    let ln_x = st.func("ln", vec![x]);
    let ln_y = st.func("ln", vec![y]);
    let two = st.int(2);
    let two_ln_x = st.mul(vec![two, ln_x]);
    let sum = st.add(vec![two_ln_x, ln_y]); // Two terms: 2*ln(x) + ln(y)

    let result = contract_logarithms(&mut st, sum, &ctx);

    // Should contract to ln(x^2 * y)
    assert_eq!(st.get(result).op, Op::Function);
}

#[test]
fn test_expand_log_with_constant() {
    let mut st = Store::new();
    let ctx = Context::new();

    let two = st.int(2);
    let ln_2 = st.func("ln", vec![two]);

    let result = simplify_logarithms(&mut st, ln_2, &ctx);

    // Constants are positive, but won't expand further
    // (no multiplication inside)
    assert_eq!(result, ln_2);
}

#[test]
fn test_expand_log_mixed_positive_and_unknown() {
    let mut st = Store::new();
    let mut ctx = Context::new();
    let x = st.sym("x");
    let y = st.sym("y");
    ctx.assume("x", Prop::Positive);
    // y has no assumption

    let product = st.mul(vec![x, y]);
    let ln_xy = st.func("ln", vec![product]);

    let result = simplify_logarithms(&mut st, ln_xy, &ctx);

    // Should NOT expand (y not known to be positive)
    assert_eq!(result, ln_xy);
}

#[test]
fn test_no_contract_non_log_sum() {
    let mut st = Store::new();
    let ctx = Context::new();
    let x = st.sym("x");
    let y = st.sym("y");

    let sum = st.add(vec![x, y]);

    let result = contract_logarithms(&mut st, sum, &ctx);

    // Should remain unchanged (not log terms)
    assert_eq!(result, sum);
}

#[test]
fn test_expand_log_power_negative_exponent() {
    let mut st = Store::new();
    let mut ctx = Context::new();
    let x = st.sym("x");
    ctx.assume("x", Prop::Positive);

    let neg_two = st.int(-2);
    let x_neg_2 = st.pow(x, neg_two);
    let ln_expr = st.func("ln", vec![x_neg_2]);

    let result = simplify_logarithms(&mut st, ln_expr, &ctx);

    // Should expand to -2*ln(x)
    assert_eq!(st.get(result).op, Op::Mul);
}
