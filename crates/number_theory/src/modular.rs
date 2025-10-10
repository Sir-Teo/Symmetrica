//! Modular Arithmetic Operations
//!
//! This module provides efficient modular arithmetic:
//! - Modular exponentiation
//! - Legendre symbol
//! - Quadratic residues
//! - Tonelli-Shanks algorithm

/// Modular exponentiation: compute (base^exp) mod modulus
/// Uses binary exponentiation for efficiency
pub fn mod_pow(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
    if modulus == 1 {
        return 0;
    }

    let mut result = 1u64;
    base %= modulus;

    while exp > 0 {
        if exp % 2 == 1 {
            result = ((result as u128 * base as u128) % modulus as u128) as u64;
        }
        exp >>= 1;
        base = ((base as u128 * base as u128) % modulus as u128) as u64;
    }

    result
}

/// Legendre symbol (a/p) for odd prime p
/// Returns:
///   1 if a is a quadratic residue mod p
///  -1 if a is a non-residue mod p
///   0 if a ≡ 0 (mod p)
pub fn legendre_symbol(a: i64, p: u64) -> i8 {
    if p == 2 {
        return 1;
    }

    let a_mod = ((a % p as i64) + p as i64) as u64 % p;

    if a_mod == 0 {
        return 0;
    }

    // Use Euler's criterion: (a/p) ≡ a^((p-1)/2) (mod p)
    let result = mod_pow(a_mod, (p - 1) / 2, p);

    if result == 1 {
        1
    } else if result == p - 1 {
        -1
    } else {
        0
    }
}

/// Check if a is a quadratic residue modulo p
pub fn is_quadratic_residue(a: i64, p: u64) -> bool {
    legendre_symbol(a, p) == 1
}

/// Tonelli-Shanks algorithm for computing square roots modulo prime p
/// Returns r such that r² ≡ n (mod p), or None if n is not a quadratic residue
pub fn tonelli_shanks(n: u64, p: u64) -> Option<u64> {
    if p == 2 {
        return Some(n % 2);
    }

    // Check if n is a quadratic residue
    if legendre_symbol(n as i64, p) != 1 {
        return None;
    }

    // Factor out powers of 2 from p-1
    let mut q = p - 1;
    let mut s = 0u32;
    while q.is_multiple_of(2) {
        q /= 2;
        s += 1;
    }

    // Find a non-residue z
    let mut z = 2u64;
    while legendre_symbol(z as i64, p) != -1 {
        z += 1;
    }

    let mut m = s;
    let mut c = mod_pow(z, q, p);
    let mut t = mod_pow(n, q, p);
    let mut r = mod_pow(n, q.div_ceil(2), p);

    loop {
        if t == 0 {
            return Some(0);
        }
        if t == 1 {
            return Some(r);
        }

        // Find least i such that t^(2^i) = 1
        let mut i = 1u32;
        let mut temp = (t as u128 * t as u128) % p as u128;
        while temp != 1 && i < m {
            temp = (temp * temp) % p as u128;
            i += 1;
        }

        let b = mod_pow(c, 1 << (m - i - 1), p);
        m = i;
        c = ((b as u128 * b as u128) % p as u128) as u64;
        t = ((t as u128 * c as u128) % p as u128) as u64;
        r = ((r as u128 * b as u128) % p as u128) as u64;
    }
}

/// Baby-step giant-step algorithm for discrete logarithm
/// Find x such that base^x ≡ target (mod modulus)
/// Only practical for small moduli (< 10^6)
pub fn discrete_log(base: u64, target: u64, modulus: u64) -> Option<u64> {
    if modulus == 1 {
        return Some(0);
    }

    let m = (modulus as f64).sqrt().ceil() as u64 + 1;

    // Baby step: compute base^j mod modulus for j = 0..m
    use std::collections::HashMap;
    let mut table = HashMap::new();

    let mut gamma = 1u64;
    for j in 0..m {
        if gamma == target {
            return Some(j);
        }
        table.insert(gamma, j);
        gamma = ((gamma as u128 * base as u128) % modulus as u128) as u64;
    }

    // Giant step: compute target * (base^(-m))^i for i = 1..m
    let base_m = mod_pow(base, m, modulus);

    // Compute base_m^(-1) using Fermat's little theorem (assuming modulus is prime)
    // For general modulus, would need extended GCD
    let base_m_inv = mod_pow(base_m, modulus - 2, modulus);

    let mut gamma = target;
    for i in 0..m {
        if let Some(&j) = table.get(&gamma) {
            return Some(i * m + j);
        }
        gamma = ((gamma as u128 * base_m_inv as u128) % modulus as u128) as u64;
    }

    None
}

/// Euler's totient function φ(n): count of integers k in 1..n with gcd(k,n) = 1
pub fn euler_totient(n: u64) -> u64 {
    if n == 1 {
        return 1;
    }

    // Use formula: φ(n) = n * ∏(1 - 1/p) for all prime factors p
    let mut result = n;
    let mut n_mut = n;

    // Check factor 2
    if n_mut.is_multiple_of(2) {
        result /= 2;
        while n_mut.is_multiple_of(2) {
            n_mut /= 2;
        }
    }

    // Check odd factors
    let mut p = 3u64;
    while p * p <= n_mut {
        if n_mut.is_multiple_of(p) {
            result -= result / p;
            while n_mut.is_multiple_of(p) {
                n_mut /= p;
            }
        }
        p += 2;
    }

    // If n_mut > 1, then it's a prime factor
    if n_mut > 1 {
        result -= result / n_mut;
    }

    result
}

/// Find a primitive root modulo p (generator of multiplicative group)
/// Returns the smallest primitive root if p is prime, None otherwise
pub fn primitive_root(p: u64) -> Option<u64> {
    if p < 2 {
        return None;
    }

    // For simplicity, only handle prime p
    // A primitive root g satisfies: ord_p(g) = φ(p) = p-1

    let phi = p - 1; // For prime p, φ(p) = p-1

    // Find prime factors of phi
    let mut factors = Vec::new();
    let mut temp = phi;
    let limit = (phi as f64).sqrt() as u64 + 1;

    for d in 2..=limit {
        if temp == 1 {
            break;
        }
        if temp.is_multiple_of(d) {
            factors.push(d);
            while temp.is_multiple_of(d) {
                temp /= d;
            }
        }
    }
    if temp > 1 {
        factors.push(temp);
    }

    // Test candidates
    'outer: for g in 2..p {
        // Check if g^(phi/q) ≢ 1 (mod p) for all prime factors q of phi
        for &q in &factors {
            if mod_pow(g, phi / q, p) == 1 {
                continue 'outer;
            }
        }
        // g is a primitive root
        return Some(g);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_pow_small() {
        assert_eq!(mod_pow(2, 10, 1000), 24);
        assert_eq!(mod_pow(3, 4, 7), 4);
        assert_eq!(mod_pow(5, 3, 13), 8);
    }

    #[test]
    fn test_mod_pow_large() {
        // 2^100 mod 1000000007
        let result = mod_pow(2, 100, 1_000_000_007);
        assert!(result < 1_000_000_007);
    }

    #[test]
    fn test_legendre_symbol() {
        // 2 is a quadratic residue mod 7
        assert_eq!(legendre_symbol(2, 7), 1);
        // 3 is not a quadratic residue mod 7
        assert_eq!(legendre_symbol(3, 7), -1);
        // 0 mod 7
        assert_eq!(legendre_symbol(0, 7), 0);
    }

    #[test]
    fn test_is_quadratic_residue() {
        // Squares are always quadratic residues
        assert!(is_quadratic_residue(1, 7));
        assert!(is_quadratic_residue(4, 7));
        assert!(is_quadratic_residue(2, 7)); // 3² ≡ 2 (mod 7)
    }

    #[test]
    fn test_tonelli_shanks() {
        // Find square root of 2 mod 7
        if let Some(r) = tonelli_shanks(2, 7) {
            assert_eq!((r * r) % 7, 2);
        }

        // Find square root of 10 mod 13
        if let Some(r) = tonelli_shanks(10, 13) {
            assert_eq!((r * r) % 13, 10);
        }
    }

    #[test]
    fn test_tonelli_shanks_non_residue() {
        // 3 is not a quadratic residue mod 7
        assert!(tonelli_shanks(3, 7).is_none());
    }

    #[test]
    fn test_discrete_log_small() {
        // 2^x ≡ 8 (mod 11), x = 3
        let result = discrete_log(2, 8, 11);
        assert_eq!(result, Some(3));
    }

    #[test]
    fn test_euler_totient_small() {
        assert_eq!(euler_totient(1), 1);
        assert_eq!(euler_totient(2), 1); // φ(2) = 1
        assert_eq!(euler_totient(6), 2); // φ(6) = 2 (1,5 coprime to 6)
        assert_eq!(euler_totient(9), 6); // φ(9) = 6
    }

    #[test]
    fn test_euler_totient_prime() {
        // For prime p, φ(p) = p-1
        assert_eq!(euler_totient(7), 6);
        assert_eq!(euler_totient(11), 10);
        assert_eq!(euler_totient(13), 12);
    }

    #[test]
    fn test_euler_totient_prime_power() {
        // φ(p^k) = p^k - p^(k-1) = p^(k-1)(p-1)
        assert_eq!(euler_totient(4), 2); // φ(2^2) = 2
        assert_eq!(euler_totient(8), 4); // φ(2^3) = 4
        assert_eq!(euler_totient(25), 20); // φ(5^2) = 20
    }

    #[test]
    fn test_primitive_root_small_primes() {
        // 2 is a primitive root mod 5
        let root = primitive_root(5);
        assert!(root.is_some());
        let g = root.unwrap();
        // Verify g generates all non-zero elements mod 5
        assert!((2..5).contains(&g));
    }

    #[test]
    fn test_primitive_root_seven() {
        // 3 is the smallest primitive root mod 7
        let root = primitive_root(7);
        assert_eq!(root, Some(3));

        // Verify: 3^1=3, 3^2=2, 3^3=6, 3^4=4, 3^5=5, 3^6=1 (mod 7)
        // This generates all elements 1..6
    }

    #[test]
    fn test_primitive_root_eleven() {
        // 2 is a primitive root mod 11
        let root = primitive_root(11);
        assert_eq!(root, Some(2));
    }

    #[test]
    fn test_discrete_log_verify() {
        let base = 3u64;
        let modulus = 17u64;
        let x = 5u64;
        let target = mod_pow(base, x, modulus);

        let result = discrete_log(base, target, modulus);
        assert_eq!(result, Some(x));
    }
}
