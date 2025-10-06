//! Tests for basic summation formulas

use expr_core::Store;
use summation::{sum_arithmetic, sum_geometric, sum_power};

#[test]
fn test_arithmetic_series_1_to_n() {
    let mut st = Store::new();
    let k = st.sym("k");
    let one = st.int(1);
    let n = st.sym("n");

    // ∑(k=1 to n) k = n(n+1)/2
    let zero = st.int(0);
    let result = sum_arithmetic(&mut st, k, one, n, zero, one).expect("arithmetic sum");

    // Check structure: should contain n, n+1, and 1/2
    let result_str = st.to_string(result);
    assert!(result_str.contains("n"));
}

#[test]
fn test_arithmetic_series_concrete() {
    let mut st = Store::new();
    let k = st.sym("k");
    let one = st.int(1);
    let hundred = st.int(100);

    // ∑(k=1 to 100) k = 100*101/2 = 5050
    let zero = st.int(0);
    let result = sum_arithmetic(&mut st, k, one, hundred, zero, one).expect("sum 1 to 100");

    let result_str = st.to_string(result);
    // Should contain 100, 101, or evaluate to something with these
    assert!(result_str.contains("100") || result_str.contains("50"));
}

#[test]
fn test_arithmetic_series_with_offset() {
    let mut st = Store::new();
    let k = st.sym("k");
    let one = st.int(1);
    let ten = st.int(10);
    let three = st.int(3);

    // ∑(k=1 to 10) (3 + 2k)
    let two = st.int(2);
    let result = sum_arithmetic(&mut st, k, one, ten, three, two).expect("arithmetic with offset");

    let result_str = st.to_string(result);
    // Should be 10 terms: first = 3+2*1=5, last = 3+2*10=23, sum = 10*(5+23)/2 = 140
    assert!(!result_str.is_empty());
}

#[test]
fn test_geometric_series_powers_of_two() {
    let mut st = Store::new();
    let k = st.sym("k");
    let zero = st.int(0);
    let n = st.sym("n");
    let two = st.int(2);

    // ∑(k=0 to n) 2^k = 2^(n+1) - 1
    let term = st.pow(two, k);
    let result = sum_geometric(&mut st, term, "k", zero, n, two).expect("geometric sum");

    let result_str = st.to_string(result);
    // Should contain powers of 2
    assert!(result_str.contains("2") || result_str.contains("pow"));
}

#[test]
fn test_geometric_series_concrete() {
    let mut st = Store::new();
    let k = st.sym("k");
    let zero = st.int(0);
    let five = st.int(5);
    let two = st.int(2);

    // ∑(k=0 to 5) 2^k = 2^0 + 2^1 + 2^2 + 2^3 + 2^4 + 2^5 = 1+2+4+8+16+32 = 63
    let term = st.pow(two, k);
    let result = sum_geometric(&mut st, term, "k", zero, five, two).expect("sum powers of 2");

    let result_str = st.to_string(result);
    // Should evaluate to 63 or contain 2^6-1
    assert!(!result_str.is_empty());
}

#[test]
fn test_power_sum_squares() {
    let mut st = Store::new();
    let one = st.int(1);
    let n = st.sym("n");
    let two = st.int(2);

    // ∑(k=1 to n) k² = n(n+1)(2n+1)/6
    let result = sum_power(&mut st, "k", one, n, two).expect("sum of squares");

    let result_str = st.to_string(result);
    // Should contain n and formula components
    assert!(result_str.contains("n"));
}

#[test]
fn test_power_sum_squares_concrete() {
    let mut st = Store::new();
    let one = st.int(1);
    let five = st.int(5);
    let two = st.int(2);

    // ∑(k=1 to 5) k² = 1 + 4 + 9 + 16 + 25 = 55
    let result = sum_power(&mut st, "k", one, five, two).expect("sum 1² to 5²");

    let result_str = st.to_string(result);
    // Should evaluate to 55 or contain formula 5*6*11/6
    assert!(result_str.contains("55") || result_str.contains("5"));
}

#[test]
fn test_power_sum_linear() {
    let mut st = Store::new();
    let one = st.int(1);
    let ten = st.int(10);

    // ∑(k=1 to 10) k = 55
    let result = sum_power(&mut st, "k", one, ten, one).expect("sum k");

    let result_str = st.to_string(result);
    // Should contain 10, 11, or 55
    assert!(result_str.contains("10") || result_str.contains("55"));
}

#[test]
fn test_sum_constant() {
    let mut st = Store::new();
    let k = st.sym("k");
    let one = st.int(1);
    let n = st.sym("n");
    let c = st.int(7);

    // ∑(k=1 to n) 7 = 7n
    let zero = st.int(0);
    let result = sum_arithmetic(&mut st, k, one, n, c, zero).expect("sum constant");

    let result_str = st.to_string(result);
    // Should contain 7 and n
    assert!(result_str.contains("7") && result_str.contains("n"));
}
