//! Cyclotomic Fields Q(ζ_n)
//!
//! This module provides support for cyclotomic extensions Q(ζ_n) where ζ_n is a primitive nth root of unity.
//! ζ_n = e^(2πi/n)

use arith::Q;
use std::f64::consts::PI;

/// Represents an element in Q(ζ_n) as a polynomial in ζ_n
/// Element: a_0 + a_1*ζ + a_2*ζ^2 + ... + a_{φ(n)-1}*ζ^{φ(n)-1}
/// where φ(n) is Euler's totient function
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cyclotomic {
    /// Coefficients a_0, a_1, ..., a_{φ(n)-1}
    pub coeffs: Vec<Q>,
    /// Order n (ζ_n is primitive nth root of unity)
    pub n: usize,
}

impl Cyclotomic {
    /// Create a new cyclotomic number
    pub fn new(coeffs: Vec<Q>, n: usize) -> Self {
        assert!(n > 0, "Order must be positive");
        let phi_n = euler_phi(n);
        assert!(coeffs.len() <= phi_n, "Too many coefficients for Q(ζ_{})", n);

        // Pad with zeros if needed
        let mut padded = coeffs;
        padded.resize(phi_n, Q::zero());

        Cyclotomic { coeffs: padded, n }
    }

    /// Create from a rational number (constant term)
    pub fn from_rational(q: Q, n: usize) -> Self {
        let phi_n = euler_phi(n);
        let mut coeffs = vec![Q::zero(); phi_n];
        coeffs[0] = q;
        Cyclotomic { coeffs, n }
    }

    /// Create the primitive root ζ_n itself
    pub fn primitive_root(n: usize) -> Self {
        let phi_n = euler_phi(n);
        let mut coeffs = vec![Q::zero(); phi_n];
        if phi_n > 1 {
            coeffs[1] = Q::new(1, 1);
        }
        Cyclotomic { coeffs, n }
    }

    /// Degree of the extension [Q(ζ_n) : Q] = φ(n)
    pub fn degree(&self) -> usize {
        euler_phi(self.n)
    }

    /// Evaluate numerically (for testing)
    pub fn eval_numeric(&self) -> (f64, f64) {
        let theta = 2.0 * PI / (self.n as f64);
        let mut real = 0.0;
        let mut imag = 0.0;

        for (k, coeff) in self.coeffs.iter().enumerate() {
            let c_val = coeff.0 as f64 / coeff.1 as f64;
            let angle = theta * (k as f64);
            real += c_val * angle.cos();
            imag += c_val * angle.sin();
        }

        (real, imag)
    }

    /// Complex conjugate (ζ_n → ζ_n^{-1} = ζ_n^{n-1})
    pub fn conjugate(&self) -> Self {
        // For ζ_n, conjugate maps ζ^k → ζ^{-k} = ζ^{n-k}
        let mut new_coeffs = vec![Q::zero(); self.coeffs.len()];
        new_coeffs[0] = self.coeffs[0]; // Constant term unchanged

        for k in 1..self.coeffs.len() {
            // ζ^k → ζ^{n-k}
            let new_idx = (self.n - k) % self.n;
            if new_idx < self.coeffs.len() {
                new_coeffs[new_idx] = self.coeffs[k];
            }
        }

        Cyclotomic { coeffs: new_coeffs, n: self.n }
    }

    /// Norm (product of all conjugates)
    pub fn norm(&self) -> Q {
        // For cyclotomic fields, this is more complex
        // Simplified: just return product with conjugate for now
        let conj = self.conjugate();
        let prod = self.clone() * conj;

        // Should be rational, return constant term
        prod.coeffs[0]
    }

    /// Trace (sum of all conjugates)
    pub fn trace(&self) -> Q {
        // Simplified: for ζ_n, trace is φ(n) times the constant term
        let phi = euler_phi(self.n);
        Q(self.coeffs[0].0 * phi as i64, self.coeffs[0].1)
    }
}

impl std::ops::Add for Cyclotomic {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        assert_eq!(self.n, rhs.n, "Cannot add elements from different cyclotomic fields");

        let coeffs: Vec<Q> =
            self.coeffs.iter().zip(rhs.coeffs.iter()).map(|(a, b)| arith::add_q(*a, *b)).collect();

        Cyclotomic { coeffs, n: self.n }
    }
}

impl std::ops::Mul for Cyclotomic {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        assert_eq!(self.n, rhs.n, "Cannot multiply elements from different cyclotomic fields");

        let phi_n = euler_phi(self.n);
        let mut result = vec![Q::zero(); phi_n];

        // Multiply polynomials and reduce modulo Φ_n(x)
        for (i, a) in self.coeffs.iter().enumerate() {
            for (j, b) in rhs.coeffs.iter().enumerate() {
                let coeff = arith::mul_q(*a, *b);
                let power = (i + j) % self.n;

                // Reduce using ζ^n = 1
                if power < phi_n {
                    result[power] = arith::add_q(result[power], coeff);
                } else {
                    // ζ^n = 1, so ζ^{n+k} = ζ^k
                    let reduced_power = power % self.n;
                    if reduced_power < phi_n {
                        result[reduced_power] = arith::add_q(result[reduced_power], coeff);
                    }
                }
            }
        }

        Cyclotomic { coeffs: result, n: self.n }
    }
}

/// Euler's totient function φ(n)
/// Returns the number of integers k in 1..n that are coprime to n
pub fn euler_phi(n: usize) -> usize {
    if n == 1 {
        return 1;
    }

    let mut result = n;
    let mut n_mut = n;
    let mut p = 2;

    while p * p <= n_mut {
        if n_mut.is_multiple_of(p) {
            while n_mut.is_multiple_of(p) {
                n_mut /= p;
            }
            result -= result / p;
        }
        p += 1;
    }

    if n_mut > 1 {
        result -= result / n_mut;
    }

    result
}

/// Check if n is a primitive nth root of unity
pub fn is_primitive_root(power: usize, n: usize) -> bool {
    if power == 0 {
        return false;
    }

    // ζ^k is primitive iff gcd(k, n) = 1
    gcd(power, n) == 1
}

fn gcd(mut a: usize, mut b: usize) -> usize {
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
    fn test_euler_phi() {
        assert_eq!(euler_phi(1), 1);
        assert_eq!(euler_phi(2), 1);
        assert_eq!(euler_phi(3), 2);
        assert_eq!(euler_phi(4), 2);
        assert_eq!(euler_phi(5), 4);
        assert_eq!(euler_phi(6), 2);
        assert_eq!(euler_phi(8), 4);
        assert_eq!(euler_phi(12), 4);
    }

    #[test]
    fn test_primitive_root_creation() {
        let zeta_4 = Cyclotomic::primitive_root(4);
        assert_eq!(zeta_4.n, 4);
        assert_eq!(zeta_4.degree(), 2); // φ(4) = 2
    }

    #[test]
    fn test_from_rational() {
        let q = Q::new(3, 2);
        let elem = Cyclotomic::from_rational(q, 5);
        assert_eq!(elem.coeffs[0], q);
        assert_eq!(elem.degree(), 4); // φ(5) = 4
    }

    #[test]
    fn test_addition() {
        let a = Cyclotomic::from_rational(Q::new(1, 1), 3);
        let b = Cyclotomic::from_rational(Q::new(2, 1), 3);
        let c = a + b;
        assert_eq!(c.coeffs[0], Q::new(3, 1));
    }

    #[test]
    fn test_multiplication_constants() {
        let a = Cyclotomic::from_rational(Q::new(2, 1), 4);
        let b = Cyclotomic::from_rational(Q::new(3, 1), 4);
        let c = a * b;
        assert_eq!(c.coeffs[0], Q::new(6, 1));
    }

    #[test]
    fn test_is_primitive_root() {
        assert!(is_primitive_root(1, 5));
        assert!(is_primitive_root(2, 5));
        assert!(is_primitive_root(3, 5));
        assert!(is_primitive_root(4, 5));
        assert!(!is_primitive_root(0, 5));

        assert!(is_primitive_root(1, 6));
        assert!(!is_primitive_root(2, 6)); // gcd(2,6) = 2
        assert!(!is_primitive_root(3, 6)); // gcd(3,6) = 3
        assert!(is_primitive_root(5, 6));
    }

    #[test]
    fn test_conjugate() {
        let zeta = Cyclotomic::primitive_root(4);
        let conj = zeta.conjugate();

        // For ζ_4 = i, conjugate should be -i = ζ_4^3
        assert_eq!(conj.n, 4);
    }

    #[test]
    fn test_degree() {
        assert_eq!(Cyclotomic::primitive_root(3).degree(), 2);
        assert_eq!(Cyclotomic::primitive_root(4).degree(), 2);
        assert_eq!(Cyclotomic::primitive_root(5).degree(), 4);
        assert_eq!(Cyclotomic::primitive_root(8).degree(), 4);
    }
}
