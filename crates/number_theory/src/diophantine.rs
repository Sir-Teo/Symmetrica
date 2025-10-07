//! Diophantine Equation Solvers
//!
//! This module provides solvers for various Diophantine equations:
//! - Linear: ax + by = c
//! - Pell's equation: x² - dy² = 1
//! - Pythagorean triples

/// Extended Euclidean Algorithm
/// Returns (gcd, x, y) such that ax + by = gcd(a, b)
pub fn extended_gcd(a: i64, b: i64) -> (i64, i64, i64) {
    if b == 0 {
        return (a, 1, 0);
    }

    let (gcd, x1, y1) = extended_gcd(b, a % b);
    let x = y1;
    let y = x1 - (a / b) * y1;

    (gcd, x, y)
}

/// Solve linear Diophantine equation ax + by = c
/// Returns (x0, y0) as a particular solution, or None if no solution exists
/// General solution: x = x0 + (b/gcd)*t, y = y0 - (a/gcd)*t for any integer t
pub fn solve_linear_diophantine(a: i64, b: i64, c: i64) -> Option<(i64, i64)> {
    if a == 0 && b == 0 {
        return if c == 0 { Some((0, 0)) } else { None };
    }

    let (gcd, x0, y0) = extended_gcd(a, b);

    // Check if solution exists
    if c % gcd != 0 {
        return None;
    }

    // Scale the solution
    let scale = c / gcd;
    Some((x0 * scale, y0 * scale))
}

/// Generate Pythagorean triples (a, b, c) where a² + b² = c²
/// Returns primitive triples up to the given limit
pub fn pythagorean_triples(limit: u64) -> Vec<(u64, u64, u64)> {
    let mut triples = Vec::new();

    // Use Euclid's formula: a = m²-n², b = 2mn, c = m²+n²
    // where m > n > 0, gcd(m,n) = 1, and m-n is odd
    let m_limit = (limit as f64).sqrt() as u64 + 1;

    for m in 2..=m_limit {
        for n in 1..m {
            // Check conditions for primitive triple
            if gcd_u64(m, n) != 1 {
                continue;
            }
            if (m - n) % 2 == 0 {
                continue;
            }

            let a = m * m - n * n;
            let b = 2 * m * n;
            let c = m * m + n * n;

            if c > limit {
                break;
            }

            // Ensure a ≤ b
            if a <= b {
                triples.push((a, b, c));
            } else {
                triples.push((b, a, c));
            }
        }
    }

    triples.sort();
    triples
}

/// Solve Pell's equation x² - dy² = 1
/// Returns the fundamental solution (x, y), or None if d is a perfect square
pub fn solve_pell(d: u64) -> Option<(u64, u64)> {
    // Check if d is a perfect square
    let sqrt_d = (d as f64).sqrt() as u64;
    if sqrt_d * sqrt_d == d {
        return None; // No non-trivial solution
    }

    // Use continued fraction expansion of √d
    // This is a simplified implementation for small d
    let mut m = 0i64;
    let mut d_val = 1i64;
    let mut a = sqrt_d as i64;

    let mut p_prev = 1i64;
    let mut p_curr = a;
    let mut q_prev = 0i64;
    let mut q_curr = 1i64;

    // Iterate until we find a solution
    for _ in 0..1000 {
        m = d_val * a - m;
        d_val = (d as i64 - m * m) / d_val;
        a = (sqrt_d as i64 + m) / d_val;

        let p_next = a * p_curr + p_prev;
        let q_next = a * q_curr + q_prev;

        // Check if (p_next, q_next) is a solution
        let x_sq = (p_next as i128) * (p_next as i128);
        let dy_sq = (d as i128) * (q_next as i128) * (q_next as i128);

        if x_sq - dy_sq == 1 {
            return Some((p_next as u64, q_next as u64));
        }

        p_prev = p_curr;
        p_curr = p_next;
        q_prev = q_curr;
        q_curr = q_next;
    }

    None // Failed to find solution
}

fn gcd_u64(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extended_gcd() {
        let (gcd, x, y) = extended_gcd(48, 18);
        assert_eq!(gcd, 6);
        assert_eq!(48 * x + 18 * y, 6);
    }

    #[test]
    fn test_extended_gcd_coprime() {
        let (gcd, x, y) = extended_gcd(17, 19);
        assert_eq!(gcd, 1);
        assert_eq!(17 * x + 19 * y, 1);
    }

    #[test]
    fn test_solve_linear_diophantine_solvable() {
        // 3x + 5y = 1
        let result = solve_linear_diophantine(3, 5, 1);
        assert!(result.is_some());
        let (x, y) = result.unwrap();
        assert_eq!(3 * x + 5 * y, 1);
    }

    #[test]
    fn test_solve_linear_diophantine_unsolvable() {
        // 6x + 9y = 5 (gcd(6,9) = 3, doesn't divide 5)
        let result = solve_linear_diophantine(6, 9, 5);
        assert!(result.is_none());
    }

    #[test]
    fn test_solve_linear_diophantine_zero() {
        let result = solve_linear_diophantine(0, 0, 0);
        assert_eq!(result, Some((0, 0)));
    }

    #[test]
    fn test_pythagorean_triples_small() {
        let triples = pythagorean_triples(15);
        assert!(triples.contains(&(3, 4, 5)));
        assert!(triples.contains(&(5, 12, 13)));
    }

    #[test]
    fn test_pythagorean_triples_verify() {
        let triples = pythagorean_triples(50);
        for (a, b, c) in triples {
            assert_eq!(a * a + b * b, c * c);
        }
    }

    #[test]
    fn test_solve_pell_small() {
        // x² - 2y² = 1, solution: (3, 2)
        let result = solve_pell(2);
        assert!(result.is_some());
        let (x, y) = result.unwrap();
        assert_eq!(x * x - 2 * y * y, 1);
    }

    #[test]
    fn test_solve_pell_perfect_square() {
        // d = 4 is a perfect square
        let result = solve_pell(4);
        assert!(result.is_none());
    }

    #[test]
    fn test_solve_pell_various() {
        for d in [2, 3, 5, 6, 7, 8, 10] {
            if let Some((x, y)) = solve_pell(d) {
                let x_sq = (x as i128) * (x as i128);
                let dy_sq = (d as i128) * (y as i128) * (y as i128);
                assert_eq!(x_sq - dy_sq, 1);
            }
        }
    }
}
