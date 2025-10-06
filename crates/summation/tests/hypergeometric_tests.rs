//! Tests for hypergeometric term recognition

use expr_core::Store;
use summation::is_hypergeometric;

#[test]
fn test_is_hypergeometric_polynomial() {
    let mut st = Store::new();
    let k = st.sym("k");
    let two = st.int(2);
    let k_squared = st.pow(k, two);
    assert!(is_hypergeometric(&mut st, k_squared, "k"));
}

#[test]
fn test_is_hypergeometric_linear() {
    let mut st = Store::new();
    let k = st.sym("k");

    // k is hypergeometric: (k+1)/k is rational
    assert!(is_hypergeometric(&mut st, k, "k"));
}

#[test]
fn test_is_hypergeometric_constant() {
    let mut st = Store::new();
    let five = st.int(5);

    // Constant is hypergeometric (ratio is 1)
    assert!(is_hypergeometric(&mut st, five, "k"));
}

#[test]
fn test_is_hypergeometric_exponential() {
    let mut st = Store::new();
    let k = st.sym("k");
    // 2^k is hypergeometric: 2^(k+1)/2^k = 2 is rational
    let two = st.int(2);
    let two_pow_k = st.pow(two, k);
    assert!(is_hypergeometric(&mut st, two_pow_k, "k"));
}

#[test]
fn test_is_hypergeometric_rational() {
    let mut st = Store::new();
    let k = st.sym("k");
    let one = st.int(1);

    // k/(k+1) is hypergeometric
    let k_plus_1 = st.add(vec![k, one]);
    let minus_one = st.int(-1);
    let inv_k_plus_1 = st.pow(k_plus_1, minus_one);
    let rational = st.mul(vec![k, inv_k_plus_1]);

    assert!(is_hypergeometric(&mut st, rational, "k"));
}

#[test]
fn test_is_hypergeometric_product() {
    let mut st = Store::new();
    let k = st.sym("k");
    // 2^k * k is hypergeometric
    let two = st.int(2);
    let two_pow_k = st.pow(two, k);
    let product = st.mul(vec![two_pow_k, k]);

    assert!(is_hypergeometric(&mut st, product, "k"));
}

#[test]
fn test_is_hypergeometric_factorial_like() {
    let mut st = Store::new();
    let k = st.sym("k");
    let one = st.int(1);

    // k(k-1)(k-2) represented as product - this should be hypergeometric
    // For simplicity, test k(k+1) which is hypergeometric
    let k_plus_1 = st.add(vec![k, one]);
    let product = st.mul(vec![k, k_plus_1]);

    assert!(is_hypergeometric(&mut st, product, "k"));
}
