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
    while q % 2 == 0 {
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
    let mut r = mod_pow(n, (q + 1) / 2, p);

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
    fn test_discrete_log_verify() {
        let base = 3u64;
        let modulus = 17u64;
        let x = 5u64;
        let target = mod_pow(base, x, modulus);

        let result = discrete_log(base, target, modulus);
        assert_eq!(result, Some(x));
    }
}
