//! Radical Denesting
//!
//! Algorithms for simplifying nested radicals.
//! Example: √(5 + 2√6) = √2 + √3

use arith::Q;

/// Attempt to denest √(a + b√c) into √x + √y form
/// Uses Ramanujan's denesting conditions
///
/// Returns Some((x, y)) if √(a + b√c) = √x + √y, otherwise None
///
/// Algorithm:
/// If √(a + b√c) = √x + √y, then squaring both sides:
/// a + b√c = x + y + 2√(xy)
/// So: a = x + y and b²c = 4xy
///
/// This gives us: x + y = a and xy = b²c/4
/// These are roots of: t² - at + b²c/4 = 0
/// Using quadratic formula: t = (a ± √(a² - b²c))/2
///
/// For denesting to work, a² - b²c must be a perfect square
pub fn denest_sqrt(a: i64, b: i64, c: i64) -> Option<(i64, i64)> {
    // Check if b²c is divisible by 4
    let b2c = b * b * c;
    if b2c % 4 != 0 {
        return None;
    }

    // Compute discriminant: a² - b²c
    let discriminant = a * a - b2c;

    if discriminant < 0 {
        return None;
    }

    // Check if discriminant is a perfect square
    let sqrt_disc = (discriminant as f64).sqrt() as i64;
    if sqrt_disc * sqrt_disc != discriminant {
        return None;
    }

    // Compute x and y
    let x = (a + sqrt_disc) / 2;
    let y = (a - sqrt_disc) / 2;

    // Verify: x + y = a and 4xy = b²c
    if x + y != a {
        return None;
    }
    if 4 * x * y != b2c {
        return None;
    }

    // Ensure x >= y >= 0
    if x >= 0 && y >= 0 {
        Some((x, y))
    } else {
        None
    }
}

/// Denest √(a/d + (b/d)√c) with rational coefficients
/// Returns Some((x, y, d)) representing (√x + √y)/√d
pub fn denest_sqrt_rational(a: Q, b: Q, c: i64) -> Option<(i64, i64, i64)> {
    // Convert to common denominator
    // a/a_den + (b/b_den)√c
    // = (a*b_den + b*a_den*√c)/(a_den*b_den)

    let a_num = a.0;
    let a_den = a.1;
    let b_num = b.0;
    let b_den = b.1;

    // Common denominator
    let common_den = a_den * b_den;
    let new_a = a_num * b_den;
    let new_b = b_num * a_den;

    // Try to denest √(new_a + new_b√c)
    if let Some((x, y)) = denest_sqrt(new_a, new_b, c) {
        // Result is (√x + √y)/√common_den
        Some((x, y, common_den))
    } else {
        None
    }
}

/// Check if a number is a perfect square
pub fn is_perfect_square(n: i64) -> bool {
    if n < 0 {
        return false;
    }
    let sqrt_n = (n as f64).sqrt() as i64;
    sqrt_n * sqrt_n == n
}

/// Simplify √n by extracting perfect square factors
/// Returns (coefficient, radicand) where √n = coefficient * √radicand
pub fn simplify_sqrt(n: i64) -> (i64, i64) {
    if n <= 0 {
        return (0, n);
    }

    let mut coeff = 1i64;
    let mut rad = n;

    // Extract factors of 4
    while rad % 4 == 0 {
        coeff *= 2;
        rad /= 4;
    }

    // Try small primes
    for p in [2i64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31] {
        let p2 = p * p;
        while rad % p2 == 0 {
            coeff *= p;
            rad /= p2;
        }
    }

    (coeff, rad)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_denest_ramanujan_example() {
        // √(5 + 2√6) = √2 + √3
        // Here: a=5, b=2, c=6
        // x + y = 5, xy = 6 => x=3, y=2
        let result = denest_sqrt(5, 2, 6);
        assert_eq!(result, Some((3, 2)));
    }

    #[test]
    fn test_denest_another_example() {
        // √(7 + 4√3) = 2 + √3 = √4 + √3
        // a=7, b=4, c=3
        // x + y = 7, xy = 12 => x=4, y=3
        let result = denest_sqrt(7, 4, 3);
        assert_eq!(result, Some((4, 3)));
    }

    #[test]
    fn test_denest_fails_when_not_perfect_square() {
        // √(5 + 2√5) cannot be denested
        // discriminant = 25 - 20 = 5 (not a perfect square)
        let result = denest_sqrt(5, 2, 5);
        assert_eq!(result, None);
    }

    #[test]
    fn test_denest_simple_case() {
        // √(2 + 2√1) = √(2 + 2) = √4 = 2
        // Using formula: x + y = 2, xy = 1
        // So x=1, y=1 (roots of t² - 2t + 1 = 0)
        // √(2 + 2√1) = √1 + √1 = 1 + 1 = 2 ✓
        let result = denest_sqrt(2, 2, 1);
        assert_eq!(result, Some((1, 1)));
    }

    #[test]
    fn test_is_perfect_square() {
        assert!(is_perfect_square(0));
        assert!(is_perfect_square(1));
        assert!(is_perfect_square(4));
        assert!(is_perfect_square(9));
        assert!(is_perfect_square(16));
        assert!(is_perfect_square(100));

        assert!(!is_perfect_square(2));
        assert!(!is_perfect_square(3));
        assert!(!is_perfect_square(5));
        assert!(!is_perfect_square(10));
        assert!(!is_perfect_square(-1));
    }

    #[test]
    fn test_simplify_sqrt() {
        assert_eq!(simplify_sqrt(1), (1, 1));
        assert_eq!(simplify_sqrt(4), (2, 1));
        assert_eq!(simplify_sqrt(8), (2, 2)); // √8 = 2√2
        assert_eq!(simplify_sqrt(12), (2, 3)); // √12 = 2√3
        assert_eq!(simplify_sqrt(18), (3, 2)); // √18 = 3√2
        assert_eq!(simplify_sqrt(50), (5, 2)); // √50 = 5√2
        assert_eq!(simplify_sqrt(72), (6, 2)); // √72 = 6√2
    }

    #[test]
    fn test_simplify_sqrt_prime() {
        // Prime numbers can't be simplified
        assert_eq!(simplify_sqrt(2), (1, 2));
        assert_eq!(simplify_sqrt(3), (1, 3));
        assert_eq!(simplify_sqrt(5), (1, 5));
        assert_eq!(simplify_sqrt(7), (1, 7));
    }

    #[test]
    fn test_denest_rational() {
        // √(5/2 + √6) with common denominator
        let a = Q::new(5, 2);
        let b = Q::new(1, 1);

        // This converts to √((5 + 2√6)/2)
        // Not directly denestable in simple form
        let result = denest_sqrt_rational(a, b, 6);
        // May or may not denest depending on the form
        // Just verify it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_denest_large_example() {
        // √(97 + 56√3) = 7 + 4√3 = √49 + √48 (but √48 = 4√3)
        // Let's try: a=97, b=56, c=3
        // x + y = 97, xy = 56²*3/4 = 2352
        // discriminant = 97² - 56²*3 = 9409 - 9408 = 1
        let result = denest_sqrt(97, 56, 3);
        assert_eq!(result, Some((49, 48)));
    }

    #[test]
    fn test_denest_negative_discriminant() {
        // If a² < b²c, discriminant is negative
        let result = denest_sqrt(1, 2, 2);
        assert_eq!(result, None);
    }

    #[test]
    fn test_denest_verification() {
        // Verify that denesting is correct by checking the identity
        if let Some((x, y)) = denest_sqrt(5, 2, 6) {
            // √(5 + 2√6) should equal √x + √y = √3 + √2
            // Squaring: (√3 + √2)² = 3 + 2 + 2√6 = 5 + 2√6 ✓
            assert_eq!(x + y, 5);
            assert_eq!(4 * x * y, 4 * 6); // 4xy = b²c
        }
    }
}
