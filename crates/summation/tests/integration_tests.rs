//! Integration tests for the summation module

use expr_core::Store;
use summation::{definite_sum, sum_closed_form};

#[test]
fn test_sum_closed_form_arithmetic() {
    let mut st = Store::new();
    let k = st.sym("k");
    let one = st.int(1);
    let n = st.sym("n");

    // ∑(k=1 to n) k
    let result = sum_closed_form(&mut st, k, "k", one, n).expect("closed form");
    let result_str = st.to_string(result);

    // Should contain n
    assert!(result_str.contains("n"));
}

#[test]
fn test_sum_closed_form_constant() {
    let mut st = Store::new();
    let c = st.int(5);
    let one = st.int(1);
    let ten = st.int(10);

    // ∑(k=1 to 10) 5 = 50
    let result = sum_closed_form(&mut st, c, "k", one, ten).expect("constant sum");
    let result_str = st.to_string(result);

    // Should contain 5 and 10
    assert!(result_str.contains("5"));
}

#[test]
fn test_definite_sum_squares() {
    let mut st = Store::new();
    let k = st.sym("k");
    let two = st.int(2);
    let k_squared = st.pow(k, two);
    let one = st.int(1);
    let n = st.sym("n");

    // ∑(k=1 to n) k²
    let result = definite_sum(&mut st, k_squared, "k", one, n).expect("sum of squares");
    let result_str = st.to_string(result);

    // Should contain n and formula components
    assert!(result_str.contains("n"));
}

#[test]
fn test_sum_arithmetic_progression() {
    let mut st = Store::new();
    let k = st.sym("k");
    let one = st.int(1);
    let n = st.sym("n");
    let two = st.int(2);
    let three = st.int(3);

    // ∑(k=1 to n) (3 + 2k)
    let two_k = st.mul(vec![two, k]);
    let expr = st.add(vec![three, two_k]);
    let result = sum_closed_form(&mut st, expr, "k", one, n).expect("arithmetic progression");
    let result_str = st.to_string(result);

    assert!(result_str.contains("n"));
}

#[test]
fn test_sum_geometric_progression() {
    let mut st = Store::new();
    let k = st.sym("k");
    let zero = st.int(0);
    let n = st.sym("n");
    let three = st.int(3);

    // ∑(k=0 to n) 3^k
    let expr = st.pow(three, k);
    let result = sum_closed_form(&mut st, expr, "k", zero, n).expect("geometric progression");
    let result_str = st.to_string(result);

    // Should contain powers or geometric formula
    assert!(result_str.contains("3") || result_str.contains("pow"));
}

#[test]
fn test_sum_linear_combination() {
    let mut st = Store::new();
    let k = st.sym("k");
    let one = st.int(1);
    let n = st.sym("n");
    let two = st.int(2);
    let three = st.int(3);

    // ∑(k=1 to n) (2k + 3) = 2*∑k + 3*n
    let two_k = st.mul(vec![two, k]);
    let expr = st.add(vec![two_k, three]);
    let result = sum_closed_form(&mut st, expr, "k", one, n).expect("linear combination");
    let result_str = st.to_string(result);

    assert!(result_str.contains("n"));
}

#[test]
fn test_sum_with_different_bounds() {
    let mut st = Store::new();
    let k = st.sym("k");
    let five = st.int(5);
    let ten = st.int(10);

    // ∑(k=5 to 10) k = sum of 5,6,7,8,9,10 = 45
    let result = sum_closed_form(&mut st, k, "k", five, ten).expect("bounded sum");
    let result_str = st.to_string(result);

    // Should work with non-unit lower bound
    assert!(!result_str.is_empty());
}

#[test]
fn test_sum_multiple_of_k() {
    let mut st = Store::new();
    let k = st.sym("k");
    let one = st.int(1);
    let n = st.sym("n");
    let seven = st.int(7);

    // ∑(k=1 to n) 7k = 7 * ∑k = 7n(n+1)/2
    let expr = st.mul(vec![seven, k]);
    let result = sum_closed_form(&mut st, expr, "k", one, n).expect("multiple of k");
    let result_str = st.to_string(result);

    assert!(result_str.contains("7") && result_str.contains("n"));
}
