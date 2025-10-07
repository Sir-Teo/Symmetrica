#![deny(warnings)]
use number_theory::is_prime_u64;

#[test]
fn primality_small_cases() {
    let primes = [2u64, 3, 5, 17, 19, 97];
    for &p in &primes {
        assert!(is_prime_u64(p), "{} should be prime", p);
    }
    let composites = [1u64, 4, 6, 9, 21, 91, 221];
    for &c in &composites {
        assert!(!is_prime_u64(c), "{} should be composite", c);
    }
}

#[test]
fn primality_larger_known() {
    // Well-known 10-digit prime
    let p: u64 = 1_000_000_007;
    assert!(is_prime_u64(p));
    // Neighbor composite
    assert!(!is_prime_u64(1_000_000_008));
}
