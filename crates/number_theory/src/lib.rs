#![deny(warnings)]
//! number_theory: Phase 7 scaffold
//! - Deterministic Miller–Rabin for u64 (common bases)
//! - Modular arithmetic helpers
//!
//! Note: The base set used here (2, 3, 5, 7, 11, 13, 17) is commonly
//! used for 64-bit integers and is deterministic for large ranges.
//! For truly full 64-bit determinism, larger specialized bases exist; we
//! can add them when needed. For now, this offers a strong probable-prime
//! test suitable for initial Phase 7 work.

/// Compute (base^exp) mod m using repeated squaring.
fn mod_pow(mut base: u128, mut exp: u128, m: u128) -> u128 {
    let mut result: u128 = 1 % m;
    base %= m;
    while exp > 0 {
        if (exp & 1) == 1 {
            result = (result * base) % m;
        }
        base = (base * base) % m;
        exp >>= 1;
    }
    result
}

#[cfg(any(test, feature = "number_theory_experimental"))]
/// Chinese Remainder Theorem for multiple congruences.
/// Input: slice of (a_i, m_i). Returns (x, M) where M=lcm of moduli, if consistent.
pub fn crt(congruences: &[(u128, u128)]) -> Option<(u128, u128)> {
    if congruences.is_empty() {
        return None;
    }
    let mut acc = congruences[0];
    for &(a, m) in &congruences[1..] {
        acc = crt_pair(acc.0, acc.1, a, m)?;
    }
    Some(acc)
}

/// Extended Euclidean algorithm: returns (g, x, y) such that a*x + b*y = g = gcd(a,b)
fn extended_gcd(a: i128, b: i128) -> (i128, i128, i128) {
    let (mut old_r, mut r) = (a, b);
    let (mut old_s, mut s) = (1i128, 0i128);
    let (mut old_t, mut t) = (0i128, 1i128);

    while r != 0 {
        let q = old_r / r;
        (old_r, r) = (r, old_r - q * r);
        (old_s, s) = (s, old_s - q * s);
        (old_t, t) = (t, old_t - q * t);
    }
    (old_r.abs(), old_s, old_t)
}

#[cfg(any(test, feature = "number_theory_experimental"))]
/// Compute (base^exp) mod m for u64 inputs.
pub fn mod_pow_u64(base: u64, exp: u64, m: u64) -> u64 {
    if m == 0 {
        return 0;
    }
    mod_pow(base as u128, exp as u128, m as u128) as u64
}

#[cfg(any(test, feature = "number_theory_experimental"))]
/// Pollard's rho factorization: returns a non-trivial factor of n, if found.
pub fn pollards_rho(n: u64) -> Option<u64> {
    if n < 2 {
        return None;
    }
    if n.is_multiple_of(2) {
        return Some(2);
    }
    if is_prime_u64(n) {
        return None;
    }

    let nn = n as u128;
    for &c in &[1u128, 3u128, 5u128, 7u128, 11u128] {
        let f = |x: u128| ((x * x) + c) % nn;
        let mut x: u128 = 2;
        let mut y: u128 = 2;
        let mut d: u128 = 1;
        let mut iter: usize = 0;
        while d == 1 && iter < 10_000 {
            x = f(x);
            y = f(f(y));
            let diff = x.abs_diff(y);
            d = gcd_u128(diff, nn);
            iter += 1;
        }
        if d != 1 && d != nn {
            return Some(d as u64);
        }
    }
    None
}

#[cfg(any(test, feature = "number_theory_experimental"))]
/// Factor n into prime factors using Pollard's rho and primality checks.
pub fn factor(n: u64) -> Vec<u64> {
    let mut result = Vec::new();
    if n < 2 {
        return result;
    }
    let mut stack = vec![n];
    while let Some(m) = stack.pop() {
        if m < 2 {
            continue;
        }
        if is_prime_u64(m) {
            result.push(m);
            continue;
        }
        if m.is_multiple_of(2) {
            result.push(2);
            stack.push(m / 2);
            continue;
        }
        if let Some(f) = pollards_rho(m) {
            stack.push(f);
            stack.push(m / f);
        } else {
            let mut d = 3u64;
            let mut found = false;
            while (d as u128 * d as u128) <= m as u128 {
                if m.is_multiple_of(d) {
                    result.push(d);
                    stack.push(m / d);
                    found = true;
                    break;
                }
                d += 2;
            }
            if !found {
                result.push(m);
            }
        }
    }
    result
}

#[cfg(any(test, feature = "number_theory_experimental"))]
/// Chinese Remainder Theorem for two congruences.
pub fn crt_pair(a1: u128, m1: u128, a2: u128, m2: u128) -> Option<(u128, u128)> {
    if m1 == 0 || m2 == 0 {
        return None;
    }
    let a1i = a1 as i128;
    let m1i = m1 as i128;
    let a2i = a2 as i128;
    let m2i = m2 as i128;
    let (g, x, _y) = extended_gcd(m1i, m2i);
    let diff = a2i - a1i;
    if diff % g != 0 {
        return None;
    }
    let m2_red = (m2i / g).abs();
    let k = ((diff / g) * x).rem_euclid(m2_red);
    let x_sol = a1i + k * m1i;
    let m_lcm = (m1i / g) * m2i;
    let m_norm: u128 = m_lcm.unsigned_abs();
    let x_norm: u128 = (x_sol.rem_euclid(m_lcm)).unsigned_abs();
    Some((x_norm, m_norm))
}

#[cfg(any(test, feature = "number_theory_experimental"))]
/// Compute the greatest common divisor of two u128 values.
pub fn gcd_u128(a: u128, b: u128) -> u128 {
    let mut a = a;
    let mut b = b;
    while b != 0 {
        let c = a % b;
        a = b;
        b = c;
    }
    a
}

 

/// Compute modular inverse of a modulo m, if it exists.
/// Returns Some(inv) such that (a*inv) % m == 1, or None if gcd(a,m) != 1.
pub fn mod_inverse(a: u64, m: u64) -> Option<u64> {
    if m == 0 {
        return None;
    }
    let a_i = (a % m) as i128;
    let m_i = m as i128;
    let (g, x, _) = extended_gcd(a_i, m_i);
    if g != 1 {
        return None;
    }
    // x may be negative; normalize to [0, m)
    let inv = ((x % m_i) + m_i) % m_i;
    Some(inv as u64)
}

/// Return true if `n` passes a single Miller–Rabin round for given base `a`.
/// n - 1 = d * 2^s with d odd.
fn miller_rabin_round(n: u128, d: u128, s: u32, a: u128) -> bool {
    if a.is_multiple_of(n) {
        return true; // a divisible by n -> trivial pass
    }
    let mut x = mod_pow(a, d, n);
    if x == 1 || x == n - 1 {
        return true;
    }
    for _ in 1..s {
        x = (x * x) % n;
        if x == n - 1 {
            return true;
        }
    }
    false
}

/// Strong probable-prime test for u64 using Miller–Rabin with common bases.
/// This is a robust check for primes; suitable as a building block for Phase 7.
pub fn is_prime_u64(n: u64) -> bool {
    // Handle small cases
    if n < 2 {
        return false;
    }
    // Small primes
    const SMALL_PRIMES: [u64; 12] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];
    for &p in &SMALL_PRIMES {
        if n == p {
            return true;
        }
        if n.is_multiple_of(p) && n != p {
            return false;
        }
    }

    // Write n-1 = d * 2^s with d odd
    let mut d: u128 = (n as u128) - 1;
    let mut s: u32 = 0;
    while d.is_multiple_of(2u128) {
        d /= 2;
        s += 1;
    }

    // Common test bases for 64-bit numbers
    // (Good coverage; we can expand if we want deterministic for full 2^64.)
    const BASES: [u64; 7] = [2, 3, 5, 7, 11, 13, 17];

    let nn = n as u128;
    for &a in &BASES {
        if (a as u128).is_multiple_of(nn) {
            continue; // skip if base == multiple of n
        }
        if !miller_rabin_round(nn, d, s, a as u128) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_primes_and_composites() {
        let primes = [2u64, 3, 5, 17, 19, 97, 2_147_483_647u64]; // 2^31-1 (Mersenne prime)
        for &p in &primes {
            assert!(is_prime_u64(p), "{} should be prime", p);
        }
        let comps = [1u64, 4, 6, 9, 21, 91, 221, 341, 561, 2_147_483_648u64]; // includes Carmichael numbers
        for &c in &comps {
            assert!(!is_prime_u64(c), "{} should be composite", c);
        }
    }

    #[test]
    fn larger_numbers() {
        // Well-known prime: 1_000_000_007
        let p: u64 = 1_000_000_007;
        assert!(is_prime_u64(p));

        // Neighbor composite
        assert!(!is_prime_u64(1_000_000_008));
    }

    #[test]
    fn mod_inverse_basic() {
        // 3 * 7 == 21 == 1 (mod 10)
        assert_eq!(mod_inverse(3, 10), Some(7));
        // inverse does not exist when gcd(a,m) != 1
        assert_eq!(mod_inverse(2, 4), None);
        // large modulus
        let m = 1_000_000_007u64;
        let a = 123_456_789u64;
        let inv = mod_inverse(a, m).unwrap();
        assert_eq!(((a as u128 * inv as u128) % m as u128) as u64, 1);
    }

    #[test]
    fn mod_pow_u64_basic() {
        assert_eq!(mod_pow_u64(2, 10, 1000), 24);
        assert_eq!(mod_pow_u64(10, 0, 7), 1);
        assert_eq!(mod_pow_u64(5, 1, 7), 5);
    }

    #[test]
    fn pollards_rho_finds_factor() {
        // 91 = 7 * 13
        let n = 91u64;
        let f = pollards_rho(n).expect("should find factor");
        assert!(f == 7 || f == 13);
        assert!(n.is_multiple_of(f));
    }

    #[test]
    fn factor_semiprime() {
        let p: u64 = 1_000_003;
        let q: u64 = 1_000_033;
        let n = p * q;
        let mut fs = factor(n);
        fs.sort_unstable();
        assert_eq!(fs, vec![p, q]);
    }

    #[test]
    fn crt_pair_basic() {
        // x ≡ 2 (mod 3), x ≡ 3 (mod 5) => x ≡ 8 (mod 15)
        let (x, m) = crt_pair(2, 3, 3, 5).expect("crt solution");
        assert_eq!(m, 15);
        assert_eq!(x % 3, 2);
        assert_eq!(x % 5, 3);
        assert_eq!(x % 15, 8);
    }

    #[test]
    fn crt_pair_inconsistent() {
        // x ≡ 1 (mod 2), x ≡ 2 (mod 4) is inconsistent
        assert!(crt_pair(1, 2, 2, 4).is_none());
    }

    #[test]
    fn crt_three_congruences() {
        // x ≡ 2 (mod 3), x ≡ 3 (mod 5), x ≡ 2 (mod 7) => x ≡ 23 (mod 105)
        let (x, m) = crt(&[(2, 3), (3, 5), (2, 7)]).expect("crt solution");
        assert_eq!(m, 105);
        assert_eq!(x % 3, 2);
        assert_eq!(x % 5, 3);
        assert_eq!(x % 7, 2);
        assert_eq!(x % 105, 23);
    }
}
