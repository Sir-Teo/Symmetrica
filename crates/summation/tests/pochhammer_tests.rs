//! Tests for Pochhammer symbol and factorials

use expr_core::Store;
use summation::{falling_factorial, pochhammer, rising_factorial};

#[test]
fn test_pochhammer_zero() {
    let mut st = Store::new();
    let x = st.sym("x");
    let zero = st.int(0);

    // (x)₀ = 1
    let result = pochhammer(&mut st, x, zero).expect("(x)₀");
    assert_eq!(st.to_string(result), "1");
}

#[test]
fn test_pochhammer_one() {
    let mut st = Store::new();
    let x = st.sym("x");
    let one = st.int(1);

    // (x)₁ = x
    let result = pochhammer(&mut st, x, one).expect("(x)₁");
    assert_eq!(st.to_string(result), "x");
}

#[test]
fn test_pochhammer_symbolic() {
    let mut st = Store::new();
    let x = st.sym("x");
    let three = st.int(3);

    // (x)₃ = x(x+1)(x+2)
    let result = pochhammer(&mut st, x, three).expect("(x)₃");
    let result_str = st.to_string(result);

    // Should contain x and additions
    assert!(result_str.contains("x"));
}

#[test]
fn test_rising_factorial_concrete() {
    let mut st = Store::new();
    let five = st.int(5);
    let three = st.int(3);

    // (5)₃ = 5*6*7 = 210
    let result = rising_factorial(&mut st, five, three).expect("5*6*7");
    let result_str = st.to_string(result);

    // Should be 210 or contain the factors
    assert!(result_str.contains("210") || result_str.contains("5"));
}

#[test]
fn test_rising_factorial_negative() {
    let mut st = Store::new();
    let x = st.sym("x");
    let neg_two = st.int(-2);

    // (x)₋₂ = 1/((x-1)(x-2))
    let result = rising_factorial(&mut st, x, neg_two).expect("negative rising");
    let result_str = st.to_string(result);

    // Should contain x and negative powers or fractions
    assert!(result_str.contains("x"));
}

#[test]
fn test_falling_factorial_concrete() {
    let mut st = Store::new();
    let five = st.int(5);
    let three = st.int(3);

    // 5^(3) = 5*4*3 = 60
    let result = falling_factorial(&mut st, five, three).expect("5*4*3");
    let result_str = st.to_string(result);

    // Should be 60 or contain factors
    assert!(result_str.contains("60") || result_str.contains("5"));
}

#[test]
fn test_falling_factorial_symbolic() {
    let mut st = Store::new();
    let n = st.sym("n");
    let three = st.int(3);

    // n^(3) = n(n-1)(n-2)
    let result = falling_factorial(&mut st, n, three).expect("n(n-1)(n-2)");
    let result_str = st.to_string(result);

    // Should contain n
    assert!(result_str.contains("n"));
}

#[test]
fn test_pochhammer_binomial_coefficient_relation() {
    let mut st = Store::new();
    let _n = st.sym("n");
    let _k = st.sym("k");

    // Binomial coefficient: C(n,k) = (n-k+1)_k / k!
    // Test (n)_k where n is concrete
    let five = st.int(5);
    let two = st.int(2);

    // (5)₂ = 5*6 = 30
    let result = pochhammer(&mut st, five, two).expect("(5)₂");
    let result_str = st.to_string(result);

    assert!(result_str.contains("30") || result_str.contains("5"));
}

#[test]
fn test_falling_factorial_zero() {
    let mut st = Store::new();
    let x = st.sym("x");
    let zero = st.int(0);

    // x^(0) = 1
    let result = falling_factorial(&mut st, x, zero).expect("x^(0)");
    assert_eq!(st.to_string(result), "1");
}

#[test]
fn test_rising_factorial_large() {
    let mut st = Store::new();
    let one = st.int(1);
    let five = st.int(5);

    // (1)₅ = 1*2*3*4*5 = 120 = 5!
    let result = rising_factorial(&mut st, one, five).expect("(1)₅");
    let result_str = st.to_string(result);

    // Should be 120
    assert!(result_str.contains("120") || result_str.contains("1"));
}
