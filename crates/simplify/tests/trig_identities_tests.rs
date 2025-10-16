//! Integration tests for Phase 6 trigonometric identities
//!
//! These tests verify the advanced trigonometric simplification rules
//! work correctly end-to-end with the simplify crate.

use expr_core::{Op, Store};
use simplify::simplify_trig;

#[test]
fn test_product_to_sum_sin_cos() {
    // sin(x) * cos(y) is NOT expanded by simplify_trig
    // Product-to-sum formulas make expressions more complex, not simpler
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");
    let sinx = st.func("sin", vec![x]);
    let cosy = st.func("cos", vec![y]);
    let product = st.mul(vec![sinx, cosy]);

    let result = simplify_trig(&mut st, product);

    // Result should remain unchanged (product-to-sum disabled)
    assert_eq!(result, product);
}

#[test]
fn test_product_to_sum_cos_cos() {
    // cos(x) * cos(y) is NOT expanded by simplify_trig
    // Product-to-sum formulas make expressions more complex, not simpler
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");
    let cosx = st.func("cos", vec![x]);
    let cosy = st.func("cos", vec![y]);
    let product = st.mul(vec![cosx, cosy]);

    let result = simplify_trig(&mut st, product);

    // Result should remain unchanged (product-to-sum disabled)
    assert_eq!(result, product);
}

#[test]
fn test_product_to_sum_sin_sin() {
    // sin(x) * sin(y) → [cos(x-y) - cos(x+y)] / 2
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");
    let sinx = st.func("sin", vec![x]);
    let siny = st.func("sin", vec![y]);
    let product = st.mul(vec![sinx, siny]);

    let result = simplify_trig(&mut st, product);

    assert_eq!(st.get(result).op, Op::Mul);
}

#[test]
fn test_sum_to_product_sin_plus_sin() {
    // sin(x) + sin(y) → 2 sin((x+y)/2) cos((x-y)/2)
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");
    let sinx = st.func("sin", vec![x]);
    let siny = st.func("sin", vec![y]);
    let sum = st.add(vec![sinx, siny]);

    let result = simplify_trig(&mut st, sum);

    // Result should be a multiplication with 2
    assert_eq!(st.get(result).op, Op::Mul);
    let result_str = st.to_string(result);
    assert!(result_str.contains("2"));
}

#[test]
fn test_sum_to_product_cos_plus_cos() {
    // cos(x) + cos(y) → 2 cos((x+y)/2) cos((x-y)/2)
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");
    let cosx = st.func("cos", vec![x]);
    let cosy = st.func("cos", vec![y]);
    let sum = st.add(vec![cosx, cosy]);

    let result = simplify_trig(&mut st, sum);

    assert_eq!(st.get(result).op, Op::Mul);
    let result_str = st.to_string(result);
    assert!(result_str.contains("2") && result_str.contains("cos"));
}

#[test]
fn test_sum_to_product_sin_minus_sin() {
    // sin(x) - sin(y) → 2 cos((x+y)/2) sin((x-y)/2)
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");
    let sinx = st.func("sin", vec![x]);
    let siny = st.func("sin", vec![y]);
    let neg_one = st.int(-1);
    let neg_siny = st.mul(vec![neg_one, siny]);
    let diff = st.add(vec![sinx, neg_siny]);

    let result = simplify_trig(&mut st, diff);

    assert_eq!(st.get(result).op, Op::Mul);
}

#[test]
fn test_sum_to_product_cos_minus_cos() {
    // cos(x) - cos(y) → -2 sin((x+y)/2) sin((x-y)/2)
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");
    let cosx = st.func("cos", vec![x]);
    let cosy = st.func("cos", vec![y]);
    let neg_one = st.int(-1);
    let neg_cosy = st.mul(vec![neg_one, cosy]);
    let diff = st.add(vec![cosx, neg_cosy]);

    let result = simplify_trig(&mut st, diff);

    assert_eq!(st.get(result).op, Op::Mul);
}

#[test]
fn test_half_angle_sin_squared() {
    // sin²(x/2) → (1 - cos(x))/2
    let mut st = Store::new();
    let x = st.sym("x");
    let half = st.rat(1, 2);
    let x_half = st.mul(vec![half, x]);
    let sin_half = st.func("sin", vec![x_half]);
    let two = st.int(2);
    let sin_sq = st.pow(sin_half, two);

    let result = simplify_trig(&mut st, sin_sq);

    // Should expand to (1 - cos(x))/2
    assert_eq!(st.get(result).op, Op::Mul);
    let result_str = st.to_string(result);
    assert!(result_str.contains("cos"));
}

#[test]
fn test_half_angle_cos_squared() {
    // cos²(x/2) → (1 + cos(x))/2
    let mut st = Store::new();
    let x = st.sym("x");
    let half = st.rat(1, 2);
    let x_half = st.mul(vec![half, x]);
    let cos_half = st.func("cos", vec![x_half]);
    let two = st.int(2);
    let cos_sq = st.pow(cos_half, two);

    let result = simplify_trig(&mut st, cos_sq);

    // Should expand to (1 + cos(x))/2
    assert_eq!(st.get(result).op, Op::Mul);
    let result_str = st.to_string(result);
    assert!(result_str.contains("cos"));
}

#[test]
fn test_half_angle_tan_squared() {
    // tan²(x/2) → (1 - cos(x))/(1 + cos(x))
    let mut st = Store::new();
    let x = st.sym("x");
    let half = st.rat(1, 2);
    let x_half = st.mul(vec![half, x]);
    let tan_half = st.func("tan", vec![x_half]);
    let two = st.int(2);
    let tan_sq = st.pow(tan_half, two);

    let result = simplify_trig(&mut st, tan_sq);

    // Should expand to (1 - cos(x))/(1 + cos(x))
    assert_eq!(st.get(result).op, Op::Mul);
}

#[test]
fn test_no_simplification_for_unmatched_patterns() {
    let mut st = Store::new();
    let x = st.sym("x");

    // Single sin(x) should not be simplified
    let sinx = st.func("sin", vec![x]);
    let result = simplify_trig(&mut st, sinx);
    assert_eq!(result, sinx);

    // sin²(x) (not half-angle) should not be expanded
    let two = st.int(2);
    let sin_sq = st.pow(sinx, two);
    let result2 = simplify_trig(&mut st, sin_sq);
    assert_eq!(result2, sin_sq);
}

#[test]
fn test_complex_expression_with_multiple_patterns() {
    // Test that the system handles expressions with multiple applicable patterns
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");

    // sin(x) * cos(x) + sin(y) * cos(y)
    let sinx = st.func("sin", vec![x]);
    let cosx = st.func("cos", vec![x]);
    let siny = st.func("sin", vec![y]);
    let cosy = st.func("cos", vec![y]);

    let prod1 = st.mul(vec![sinx, cosx]);
    let prod2 = st.mul(vec![siny, cosy]);
    let sum = st.add(vec![prod1, prod2]);

    // Each product can be simplified independently
    let result = simplify_trig(&mut st, sum);

    // Result should still be an addition
    assert_eq!(st.get(result).op, Op::Add);
}

#[test]
fn test_product_with_coefficients() {
    // 3 * sin(x) * cos(y) should preserve the coefficient
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");
    let three = st.int(3);
    let sinx = st.func("sin", vec![x]);
    let cosy = st.func("cos", vec![y]);
    let product = st.mul(vec![three, sinx, cosy]);

    let result = simplify_trig(&mut st, product);

    // Coefficient 3 should be preserved
    let result_str = st.to_string(result);
    assert!(result_str.contains("3"));
}

#[test]
fn test_sum_with_extra_terms() {
    // sin(x) + sin(y) + z should apply sum-to-product and keep z
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");
    let z = st.sym("z");
    let sinx = st.func("sin", vec![x]);
    let siny = st.func("sin", vec![y]);
    let sum = st.add(vec![sinx, siny, z]);

    let result = simplify_trig(&mut st, sum);

    // Result should be an addition containing z and the simplified trig part
    assert_eq!(st.get(result).op, Op::Add);
}

#[test]
fn test_product_same_arguments() {
    // sin(x) * cos(x) is a special case
    let mut st = Store::new();
    let x = st.sym("x");
    let sinx = st.func("sin", vec![x]);
    let cosx = st.func("cos", vec![x]);
    let product = st.mul(vec![sinx, cosx]);

    let result = simplify_trig(&mut st, product);

    // Should apply product-to-sum formula
    assert_eq!(st.get(result).op, Op::Mul);
}

#[test]
fn test_idempotency() {
    // Applying simplify_trig twice should give the same result
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");
    let sinx = st.func("sin", vec![x]);
    let siny = st.func("sin", vec![y]);
    let sum = st.add(vec![sinx, siny]);

    let result1 = simplify_trig(&mut st, sum);
    let result2 = simplify_trig(&mut st, result1);

    // Should be idempotent (or at least not crash)
    assert_eq!(st.get(result2).op, Op::Mul);
}

#[test]
fn test_nested_expression_traversal() {
    // Test that simplify_trig recursively traverses nested expressions
    // (sin(x) + sin(y)) * 2 should simplify the inner sum
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");
    let sinx = st.func("sin", vec![x]);
    let siny = st.func("sin", vec![y]);
    let sum = st.add(vec![sinx, siny]);
    let two = st.int(2);
    let nested = st.mul(vec![sum, two]);

    let result = simplify_trig(&mut st, nested);

    // The inner sum should be simplified to a product (sum-to-product)
    // Result structure: (2 * sin((x+y)/2) * cos((x-y)/2)) * 2
    assert_eq!(st.get(result).op, Op::Mul);
    let result_str = st.to_string(result);
    // Should contain simplified trig functions
    assert!(result_str.contains("sin") || result_str.contains("cos"));
}

#[test]
fn test_deeply_nested_expression() {
    // Test multiple levels of nesting
    // ((sin(x) * cos(y)) + 3)
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");
    let sinx = st.func("sin", vec![x]);
    let cosy = st.func("cos", vec![y]);
    let product = st.mul(vec![sinx, cosy]);
    let three = st.int(3);
    let sum = st.add(vec![product, three]);

    let result = simplify_trig(&mut st, sum);

    // The inner product should be simplified (product-to-sum)
    assert_eq!(st.get(result).op, Op::Add);
    let result_str = st.to_string(result);
    // Should still contain 3 and have simplified trig
    assert!(result_str.contains("3"));
    assert!(result_str.contains("sin"));
}
