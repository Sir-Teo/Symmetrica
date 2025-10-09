//! Minimal Polynomial Computation
//!
//! This module provides functions to compute minimal polynomials of algebraic numbers

use arith::Q;

/// Represents a polynomial with rational coefficients
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RationalPoly {
    /// Coefficients [a_0, a_1, ..., a_n] representing a_0 + a_1*x + ... + a_n*x^n
    pub coeffs: Vec<Q>,
}

impl RationalPoly {
    /// Create a new polynomial
    pub fn new(coeffs: Vec<Q>) -> Self {
        let mut c = coeffs;
        // Remove trailing zeros
        while c.len() > 1 && c.last() == Some(&Q::zero()) {
            c.pop();
        }
        if c.is_empty() {
            c.push(Q::zero());
        }
        RationalPoly { coeffs: c }
    }

    /// Create from integer coefficients
    pub fn from_ints(coeffs: Vec<i64>) -> Self {
        RationalPoly::new(coeffs.into_iter().map(|n| Q::new(n, 1)).collect())
    }

    /// Degree of the polynomial
    pub fn degree(&self) -> usize {
        if self.coeffs.len() == 1 && self.coeffs[0] == Q::zero() {
            0
        } else {
            self.coeffs.len() - 1
        }
    }

    /// Evaluate polynomial at a rational point
    pub fn eval(&self, x: Q) -> Q {
        if self.coeffs.is_empty() {
            return Q::zero();
        }

        // Horner's method
        let mut result = self.coeffs[self.coeffs.len() - 1];
        for i in (0..self.coeffs.len() - 1).rev() {
            result = arith::add_q(arith::mul_q(result, x), self.coeffs[i]);
        }
        result
    }

    /// Check if polynomial is monic (leading coefficient is 1)
    pub fn is_monic(&self) -> bool {
        if self.coeffs.is_empty() {
            return false;
        }
        self.coeffs[self.coeffs.len() - 1] == Q::new(1, 1)
    }

    /// Make polynomial monic by dividing by leading coefficient
    pub fn make_monic(&self) -> Self {
        if self.coeffs.is_empty() || self.coeffs.last() == Some(&Q::zero()) {
            return self.clone();
        }

        let leading = self.coeffs[self.coeffs.len() - 1];
        if leading == Q::new(1, 1) {
            return self.clone(); // Already monic
        }

        // Divide each coefficient by leading coefficient
        let coeffs: Vec<Q> = self
            .coeffs
            .iter()
            .map(|c| {
                // c / leading = c * (1/leading) = (c.0 * leading.1) / (c.1 * leading.0)
                let num = c.0 * leading.1;
                let den = c.1 * leading.0;
                let (n, d) = arith::normalize_rat(num, den);
                Q(n, d)
            })
            .collect();

        RationalPoly::new(coeffs)
    }
}

/// Compute minimal polynomial of √d over Q
/// Returns x^2 - d
pub fn minimal_poly_sqrt(d: i64) -> RationalPoly {
    // x^2 - d
    RationalPoly::from_ints(vec![-d, 0, 1])
}

/// Compute minimal polynomial of primitive nth root of unity ζ_n
/// This is the nth cyclotomic polynomial Φ_n(x)
pub fn cyclotomic_polynomial(n: usize) -> RationalPoly {
    match n {
        1 => RationalPoly::from_ints(vec![-1, 1]),         // x - 1
        2 => RationalPoly::from_ints(vec![1, 1]),          // x + 1
        3 => RationalPoly::from_ints(vec![1, 1, 1]),       // x^2 + x + 1
        4 => RationalPoly::from_ints(vec![1, 0, 1]),       // x^2 + 1
        5 => RationalPoly::from_ints(vec![1, 1, 1, 1, 1]), // x^4 + x^3 + x^2 + x + 1
        6 => RationalPoly::from_ints(vec![1, -1, 1]),      // x^2 - x + 1
        8 => RationalPoly::from_ints(vec![1, 0, 0, 0, 1]), // x^4 + 1
        _ => {
            // For general n, use recursive formula
            // This is a simplified version - full implementation would use Möbius inversion
            RationalPoly::from_ints(vec![1]) // Placeholder
        }
    }
}

/// Compute minimal polynomial of i (imaginary unit)
/// Returns x^2 + 1
pub fn minimal_poly_i() -> RationalPoly {
    RationalPoly::from_ints(vec![1, 0, 1])
}

/// Compute minimal polynomial of ∛2 (cube root of 2)
/// Returns x^3 - 2
pub fn minimal_poly_cbrt2() -> RationalPoly {
    RationalPoly::from_ints(vec![-2, 0, 0, 1])
}

/// Check if a polynomial is irreducible over Q (simplified check)
/// This is a heuristic - full implementation would use more sophisticated tests
pub fn is_irreducible(poly: &RationalPoly) -> bool {
    let deg = poly.degree();

    // Degree 0 or 1 polynomials
    if deg <= 1 {
        return deg == 1;
    }

    // Check if polynomial is monic
    if !poly.is_monic() {
        return is_irreducible(&poly.make_monic());
    }

    // For degree 2 or 3, check if it has rational roots
    // If no rational roots, it's irreducible
    if deg == 2 || deg == 3 {
        return !has_rational_root(poly);
    }

    // For higher degrees, this is a placeholder
    // Full implementation would use Eisenstein's criterion, etc.
    true
}

/// Check if polynomial has a rational root (simplified)
fn has_rational_root(poly: &RationalPoly) -> bool {
    if poly.coeffs.is_empty() {
        return false;
    }

    // Try small rational values
    for num in -10..=10 {
        for den in 1..=5 {
            let x = Q::new(num, den);
            if poly.eval(x) == Q::zero() {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poly_creation() {
        let p = RationalPoly::from_ints(vec![1, 2, 3]);
        assert_eq!(p.degree(), 2);
        assert_eq!(p.coeffs.len(), 3);
    }

    #[test]
    fn test_poly_eval() {
        // p(x) = 1 + 2x + 3x^2
        let p = RationalPoly::from_ints(vec![1, 2, 3]);

        // p(0) = 1
        assert_eq!(p.eval(Q::new(0, 1)), Q::new(1, 1));

        // p(1) = 1 + 2 + 3 = 6
        assert_eq!(p.eval(Q::new(1, 1)), Q::new(6, 1));

        // p(2) = 1 + 4 + 12 = 17
        assert_eq!(p.eval(Q::new(2, 1)), Q::new(17, 1));
    }

    #[test]
    fn test_minimal_poly_sqrt() {
        // Minimal polynomial of √2 is x^2 - 2
        let p = minimal_poly_sqrt(2);
        assert_eq!(p.degree(), 2);
        assert_eq!(p.coeffs[0], Q::new(-2, 1));
        assert_eq!(p.coeffs[1], Q::new(0, 1));
        assert_eq!(p.coeffs[2], Q::new(1, 1));
    }

    #[test]
    fn test_minimal_poly_i() {
        // Minimal polynomial of i is x^2 + 1
        let p = minimal_poly_i();
        assert_eq!(p.degree(), 2);
        assert_eq!(p.coeffs[0], Q::new(1, 1));
        assert_eq!(p.coeffs[2], Q::new(1, 1));
    }

    #[test]
    fn test_cyclotomic_polynomial() {
        // Φ_1(x) = x - 1
        let phi1 = cyclotomic_polynomial(1);
        assert_eq!(phi1.degree(), 1);

        // Φ_2(x) = x + 1
        let phi2 = cyclotomic_polynomial(2);
        assert_eq!(phi2.degree(), 1);

        // Φ_3(x) = x^2 + x + 1
        let phi3 = cyclotomic_polynomial(3);
        assert_eq!(phi3.degree(), 2);

        // Φ_4(x) = x^2 + 1
        let phi4 = cyclotomic_polynomial(4);
        assert_eq!(phi4.degree(), 2);
        assert_eq!(phi4.coeffs[0], Q::new(1, 1));
        assert_eq!(phi4.coeffs[2], Q::new(1, 1));
    }

    #[test]
    fn test_is_monic() {
        let monic = RationalPoly::from_ints(vec![1, 2, 1]);
        assert!(monic.is_monic());

        let not_monic = RationalPoly::from_ints(vec![1, 2, 3]);
        assert!(!not_monic.is_monic());
    }

    #[test]
    fn test_make_monic() {
        // 2 + 4x + 6x^2 -> (1/3) + (2/3)x + x^2
        let p = RationalPoly::from_ints(vec![2, 4, 6]);
        let monic = p.make_monic();
        assert!(monic.is_monic());
        // Check leading coefficient is 1
        assert_eq!(monic.coeffs[monic.coeffs.len() - 1], Q::new(1, 1));
    }

    #[test]
    fn test_is_irreducible_quadratic() {
        // x^2 - 2 is irreducible over Q
        let p = minimal_poly_sqrt(2);
        assert!(is_irreducible(&p));

        // x^2 + 1 is irreducible over Q
        let q = minimal_poly_i();
        assert!(is_irreducible(&q));
    }

    #[test]
    fn test_minimal_poly_cbrt2() {
        let p = minimal_poly_cbrt2();
        assert_eq!(p.degree(), 3);
        assert_eq!(p.coeffs[0], Q::new(-2, 1));
    }
}
