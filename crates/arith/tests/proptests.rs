//! Property-based tests for arith

use arith::{add_q, div_q, gcd_i64, mul_q, normalize_rat, sub_q, Q};
use proptest::prelude::*;

fn small_int() -> impl Strategy<Value = i64> {
    -10i64..=10
}

fn small_nonzero_int() -> impl Strategy<Value = i64> {
    prop_oneof![(-10i64..=-1), (1i64..=10)]
}

proptest! {
    #[test]
    fn prop_gcd_commutative(a in small_int(), b in small_int()) {
        prop_assert_eq!(gcd_i64(a, b), gcd_i64(b, a));
    }

    #[test]
    fn prop_gcd_divides_both(a in small_nonzero_int(), b in small_nonzero_int()) {
        let g = gcd_i64(a, b);
        if g != 0 {
            prop_assert_eq!(a % g, 0);
            prop_assert_eq!(b % g, 0);
        }
    }

    #[test]
    fn prop_normalize_positive_denominator(num in small_int(), den in small_nonzero_int()) {
        let (_, d) = normalize_rat(num, den);
        prop_assert!(d > 0);
    }

    #[test]
    fn prop_normalize_reduces_gcd(num in small_nonzero_int(), den in small_nonzero_int()) {
        let (n, d) = normalize_rat(num, den);
        if n != 0 {
            prop_assert_eq!(gcd_i64(n.abs(), d), 1);
        }
    }

    #[test]
    fn prop_q_addition_commutative(a in small_int(), b in small_int(), c in small_nonzero_int(), d in small_nonzero_int()) {
        let qa = Q::new(a, c);
        let qb = Q::new(b, d);
        let sum1 = add_q(qa, qb);
        let sum2 = add_q(qb, qa);
        prop_assert_eq!(sum1, sum2);
    }

    #[test]
    fn prop_q_multiplication_commutative(a in small_int(), b in small_int(), c in small_nonzero_int(), d in small_nonzero_int()) {
        let qa = Q::new(a, c);
        let qb = Q::new(b, d);
        let prod1 = mul_q(qa, qb);
        let prod2 = mul_q(qb, qa);
        prop_assert_eq!(prod1, prod2);
    }

    #[test]
    fn prop_q_add_zero_identity(a in small_int(), b in small_nonzero_int()) {
        let q = Q::new(a, b);
        let zero = Q::zero();
        let result = add_q(q, zero);
        prop_assert_eq!(result, q);
    }

    #[test]
    fn prop_q_mul_one_identity(a in small_int(), b in small_nonzero_int()) {
        let q = Q::new(a, b);
        let one = Q::one();
        let result = mul_q(q, one);
        prop_assert_eq!(result, q);
    }

    #[test]
    fn prop_q_mul_zero(a in small_int(), b in small_nonzero_int()) {
        let q = Q::new(a, b);
        let zero = Q::zero();
        let result = mul_q(q, zero);
        prop_assert!(result.is_zero());
    }

    #[test]
    fn prop_q_subtraction_inverse_of_addition(a in small_int(), b in small_int(), c in small_nonzero_int(), d in small_nonzero_int()) {
        let qa = Q::new(a, c);
        let qb = Q::new(b, d);
        let sum = add_q(qa, qb);
        let back = sub_q(sum, qb);
        prop_assert_eq!(back, qa);
    }

    #[test]
    fn prop_q_division_inverse_of_multiplication(a in small_nonzero_int(), b in small_nonzero_int(), c in small_nonzero_int(), d in small_nonzero_int()) {
        let qa = Q::new(a, c);
        let qb = Q::new(b, d);
        let prod = mul_q(qa, qb);
        let back = div_q(prod, qb);
        prop_assert_eq!(back, qa);
    }

    #[test]
    fn prop_q_new_normalizes(num in small_int(), den in small_nonzero_int()) {
        let q = Q::new(num, den);
        // Denominator should always be positive
        prop_assert!(q.1 > 0);
        // If numerator is non-zero, gcd should be 1
        if q.0 != 0 {
            prop_assert_eq!(gcd_i64(q.0.abs(), q.1), 1);
        }
    }
}
