//! Additional tests to improve pattern module coverage

use expr_core::Store;
use pattern::subst_symbol;

#[test]
fn test_subst_integer() {
    let mut st = Store::new();
    let five = st.int(5);
    let x = st.sym("x");
    let result = subst_symbol(&mut st, five, "x", x);
    assert_eq!(result, five); // Integer unchanged
}

#[test]
fn test_subst_rational() {
    let mut st = Store::new();
    let half = st.rat(1, 2);
    let x = st.sym("x");
    let result = subst_symbol(&mut st, half, "x", x);
    assert_eq!(result, half); // Rational unchanged
}

#[test]
fn test_subst_different_symbol() {
    let mut st = Store::new();
    let y = st.sym("y");
    let x = st.sym("x");
    let result = subst_symbol(&mut st, y, "x", x);
    assert_eq!(result, y); // Different symbol unchanged
}

#[test]
fn test_subst_in_mul() {
    let mut st = Store::new();
    let x = st.sym("x");
    let two = st.int(2);
    let expr = st.mul(vec![two, x]); // 2*x

    let three = st.int(3);
    let result = subst_symbol(&mut st, expr, "x", three);
    // Should be 2*3
    let expected = st.mul(vec![two, three]);
    assert_eq!(result, expected);
}

#[test]
fn test_subst_in_function() {
    let mut st = Store::new();
    let x = st.sym("x");
    let sin_x = st.func("sin", vec![x]);

    let y = st.sym("y");
    let result = subst_symbol(&mut st, sin_x, "x", y);
    let expected = st.func("sin", vec![y]);
    assert_eq!(result, expected);
}

#[test]
fn test_subst_in_piecewise() {
    let mut st = Store::new();
    let x = st.sym("x");
    let zero = st.int(0);
    let one = st.int(1);
    let two = st.int(2);

    // Piecewise: if x > 0 then 1 else 2
    let cond = st.add(vec![x, zero]); // Simplified condition
    let pw = st.piecewise(vec![(one, cond), (two, zero)]);

    let y = st.sym("y");
    let result = subst_symbol(&mut st, pw, "x", y);

    // Should substitute x with y in condition
    let expected_cond = st.add(vec![y, zero]);
    let expected = st.piecewise(vec![(one, expected_cond), (two, zero)]);
    assert_eq!(result, expected);
}

#[test]
fn test_subst_nested_expr() {
    let mut st = Store::new();
    let x = st.sym("x");
    let one = st.int(1);
    let two = st.int(2);

    // (x + 1)^2
    let xp1 = st.add(vec![x, one]);
    let expr = st.pow(xp1, two);

    // Substitute x with x+1
    let x_plus_1 = st.add(vec![x, one]);
    let result = subst_symbol(&mut st, expr, "x", x_plus_1);

    // Should be ((x+1)+1)^2 = (x+2)^2
    let inner = st.add(vec![x_plus_1, one]);
    let expected = st.pow(inner, two);
    assert_eq!(result, expected);
}

#[test]
fn test_subst_multiple_occurrences() {
    let mut st = Store::new();
    let x = st.sym("x");

    // x + x + x
    let expr = st.add(vec![x, x, x]);

    let five = st.int(5);
    let result = subst_symbol(&mut st, expr, "x", five);

    // Should be 5 + 5 + 5
    let expected = st.add(vec![five, five, five]);
    assert_eq!(result, expected);
}

#[test]
fn test_subst_in_complex_mul() {
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");
    let two = st.int(2);

    // 2 * x * y
    let expr = st.mul(vec![two, x, y]);

    let three = st.int(3);
    let result = subst_symbol(&mut st, expr, "x", three);

    // Should be 2 * 3 * y
    let expected = st.mul(vec![two, three, y]);
    assert_eq!(result, expected);
}

#[test]
fn test_subst_preserves_structure() {
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");

    // x^y
    let expr = st.pow(x, y);

    let z = st.sym("z");
    let result = subst_symbol(&mut st, expr, "x", z);

    // Should be z^y
    let expected = st.pow(z, y);
    assert_eq!(result, expected);
}
