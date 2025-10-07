#![deny(warnings)]
use number_theory::{crt_pair, factor, is_prime_u64, mod_inverse};

#[test]
fn primality_various() {
    let primes = [2u64, 3, 5, 17, 97, 1_000_000_007];
    for &p in &primes {
        assert!(is_prime_u64(p));
    }
    let composites = [1u64, 4, 6, 9, 21, 1_000_000_008];
    for &c in &composites {
        assert!(!is_prime_u64(c));
    }
}

#[test]
fn factor_semiprime_e2e() {
    // Factor a semiprime
    let p: u64 = 1_000_003;
    let q: u64 = 1_000_033;
    let n = p * q;
    let mut fs = factor(n);
    fs.sort_unstable();
    assert_eq!(fs, vec![p, q]);
}

#[test]
fn mod_inverse_e2e() {
    // Basic inverse
    assert_eq!(mod_inverse(3, 10), Some(7));
    // No inverse when gcd != 1
    assert_eq!(mod_inverse(2, 4), None);
}

#[test]
fn crt_pair_e2e() {
    // x ≡ 2 (mod 3), x ≡ 3 (mod 5) => x ≡ 8 (mod 15)
    let (x, m) = crt_pair(2, 3, 3, 5).expect("crt solution");
    assert_eq!(m, 15);
    assert_eq!(x % 3, 2);
    assert_eq!(x % 5, 3);
    assert_eq!(x % 15, 8);
}
