//! Integer Factorization Algorithms
//!
//! This module provides efficient factorization algorithms:
//! - Trial division with 2,3,5-wheel optimization
//! - Integration with Pollard's rho for large factors
//! - Unified interface that auto-selects best algorithm

/// Trial division with wheel factorization
/// Returns prime factorization as (prime, exponent) pairs
pub fn trial_division(n: u64, limit: Option<u64>) -> Vec<(u64, u32)> {
    if n <= 1 {
        return vec![];
    }

    let mut factors = Vec::new();
    let mut n = n;

    // Handle 2
    let mut count = 0u32;
    while n % 2 == 0 && count < 64 {
        count += 1;
        n /= 2;
    }
    if count > 0 {
        factors.push((2, count));
    }

    // Handle 3
    count = 0;
    while n % 3 == 0 && count < 64 {
        count += 1;
        n /= 3;
    }
    if count > 0 {
        factors.push((3, count));
    }

    // Handle 5
    count = 0;
    while n % 5 == 0 && count < 64 {
        count += 1;
        n /= 5;
    }
    if count > 0 {
        factors.push((5, count));
    }

    // Simple trial division for remaining factors
    let limit = limit.unwrap_or_else(|| (n as f64).sqrt() as u64 + 1);
    let mut d = 7u64;
    
    while d <= limit && n > 1 {
        count = 0;
        while n % d == 0 && count < 64 {
            count += 1;
            n /= d;
        }
        if count > 0 {
            factors.push((d, count));
        }
        
        // Skip even numbers
        d += if d % 6 == 1 { 4 } else { 2 };
        
        if d > limit {
            break;
        }
    }

    // If n > 1, it's a prime factor
    if n > 1 {
        factors.push((n, 1));
    }

    factors
}

/// Complete factorization using best algorithm
/// Auto-selects between trial division and Pollard's rho
pub fn factor(n: u64) -> Vec<(u64, u32)> {
    if n <= 1 {
        return vec![];
    }

    // For small numbers, use trial division
    if n < 1_000_000 {
        return trial_division(n, None);
    }

    // For larger numbers, use trial division up to 10^6, then Pollard's rho
    let mut factors = trial_division(n, Some(1_000_000));

    // Check if there's a large composite remaining
    if let Some(&(last_factor, exp)) = factors.last() {
        if last_factor > 1_000_000 && exp == 1 {
            // This might be composite, try Pollard's rho
            factors.pop();
            if let Some(sub_factors) = try_pollard_rho(last_factor) {
                factors.extend(sub_factors);
            } else {
                // Couldn't factor, assume prime
                factors.push((last_factor, 1));
            }
        }
    }

    // Merge duplicate factors
    merge_factors(factors)
}

/// Try Pollard's rho algorithm (simple implementation)
fn try_pollard_rho(n: u64) -> Option<Vec<(u64, u32)>> {
    if n <= 1 {
        return Some(vec![]);
    }

    // Simple Pollard's rho with f(x) = x^2 + 1
    let mut x = 2u64;
    let mut y = 2u64;

    let f = |val: u64| -> u64 { ((val as u128 * val as u128 + 1) % n as u128) as u64 };

    for _ in 0..100_000 {
        x = f(x);
        y = f(f(y));

        let diff = if x > y { x - y } else { y - x };
        let d = gcd(diff, n);

        if d != 1 && d != n {
            // Found a factor
            let mut result = Vec::new();
            
            // Recursively factor both parts
            if let Some(mut f1) = try_pollard_rho(d) {
                result.append(&mut f1);
            } else {
                result.push((d, 1));
            }
            
            if let Some(mut f2) = try_pollard_rho(n / d) {
                result.append(&mut f2);
            } else {
                result.push((n / d, 1));
            }
            
            return Some(merge_factors(result));
        }

        if d == n {
            break;
        }
    }

    None // Failed to factor
}

/// Greatest common divisor
fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a
}

/// Merge duplicate factors and sort
fn merge_factors(factors: Vec<(u64, u32)>) -> Vec<(u64, u32)> {
    use std::collections::HashMap;
    
    let mut map: HashMap<u64, u32> = HashMap::new();
    for (prime, exp) in factors {
        *map.entry(prime).or_insert(0) += exp;
    }
    
    let mut result: Vec<_> = map.into_iter().collect();
    result.sort_by_key(|&(p, _)| p);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trial_division_small() {
        assert_eq!(trial_division(12, None), vec![(2, 2), (3, 1)]);
        assert_eq!(trial_division(17, None), vec![(17, 1)]);
        assert_eq!(trial_division(100, None), vec![(2, 2), (5, 2)]);
        assert_eq!(trial_division(1, None), vec![]);
    }

    #[test]
    fn test_trial_division_primes() {
        assert_eq!(trial_division(2, None), vec![(2, 1)]);
        assert_eq!(trial_division(3, None), vec![(3, 1)]);
        assert_eq!(trial_division(97, None), vec![(97, 1)]);
    }

    #[test]
    fn test_trial_division_powers() {
        assert_eq!(trial_division(8, None), vec![(2, 3)]);
        assert_eq!(trial_division(27, None), vec![(3, 3)]);
        assert_eq!(trial_division(32, None), vec![(2, 5)]);
    }

    #[test]
    fn test_factor_small() {
        assert_eq!(factor(60), vec![(2, 2), (3, 1), (5, 1)]);
        assert_eq!(factor(210), vec![(2, 1), (3, 1), (5, 1), (7, 1)]);
    }

    #[test]
    fn test_factor_large_prime() {
        // 10007 is prime
        assert_eq!(factor(10007), vec![(10007, 1)]);
    }

    #[test]
    fn test_factor_semiprime() {
        // 10403 = 101 * 103
        let result = factor(10403);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0 * result[1].0, 10403);
    }

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(48, 18), 6);
        assert_eq!(gcd(17, 19), 1);
        assert_eq!(gcd(100, 50), 50);
    }
}
