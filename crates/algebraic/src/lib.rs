#![deny(warnings)]

//! algebraic: Algebraic number fields and extensions
//!
//! Phase 9: Algebraic Extensions
//! - Quadratic extensions Q(√d)
//! - Cyclotomic fields Q(ζ_n)
//! - Minimal polynomial computation
//! - Galois group computation
//! - Field extension towers

pub mod cyclotomic;
pub mod denesting;
pub mod galois;
pub mod minimal_poly;

use arith::{add_q, mul_q, sub_q, Q};
use std::ops::{Add, Mul, Neg, Sub};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Quad {
    pub a: Q,
    pub b: Q,
    pub d: i64,
}

impl Quad {
    pub fn new(a: Q, b: Q, d: i64) -> Self {
        Quad { a, b, d }
    }
    pub fn from_int(n: i64, d: i64) -> Self {
        Quad::new(Q::new(n, 1), Q::zero(), d)
    }
    pub fn conj(&self) -> Self {
        Quad { a: self.a, b: Q(-self.b.0, self.b.1), d: self.d }
    }
    pub fn norm(&self) -> Q {
        // N(a + b√d) = a^2 - b^2 d
        let a2 = mul_q(self.a, self.a);
        let b2 = mul_q(self.b, self.b);
        // b2 * d
        let bd = Q(b2.0 * self.d, b2.1);
        sub_q(a2, bd)
    }

    /// Multiplicative inverse: 1/(a + b√d) = (a - b√d)/(a² - b²d)
    pub fn inv(&self) -> Option<Self> {
        let n = self.norm();
        if n.0 == 0 {
            return None; // Division by zero
        }
        let conj = self.conj();
        // Divide conjugate by norm
        let a_inv = Q(conj.a.0 * n.1, conj.a.1 * n.0);
        let b_inv = Q(conj.b.0 * n.1, conj.b.1 * n.0);
        Some(Quad::new(a_inv, b_inv, self.d))
    }

    /// Division: self / rhs
    pub fn div(&self, rhs: &Quad) -> Option<Self> {
        let rhs_inv = rhs.inv()?;
        Some(*self * rhs_inv)
    }

    /// Trace: Tr(a + b√d) = 2a
    pub fn trace(&self) -> Q {
        let (num, den) = arith::normalize_rat(self.a.0 * 2, self.a.1);
        Q(num, den)
    }

    /// Check if element is in base field Q (i.e., b = 0)
    pub fn is_rational(&self) -> bool {
        self.b.0 == 0
    }

    /// Get rational part if element is in Q
    pub fn as_rational(&self) -> Option<Q> {
        if self.is_rational() {
            Some(self.a)
        } else {
            None
        }
    }
}

impl Add for Quad {
    type Output = Quad;
    fn add(self, rhs: Quad) -> Quad {
        assert_eq!(self.d, rhs.d, "incompatible extensions: different d");
        Quad { a: add_q(self.a, rhs.a), b: add_q(self.b, rhs.b), d: self.d }
    }
}

impl Sub for Quad {
    type Output = Quad;
    fn sub(self, rhs: Quad) -> Quad {
        assert_eq!(self.d, rhs.d, "incompatible extensions: different d");
        Quad { a: sub_q(self.a, rhs.a), b: sub_q(self.b, rhs.b), d: self.d }
    }
}

impl Neg for Quad {
    type Output = Quad;
    fn neg(self) -> Quad {
        Quad { a: Q(-self.a.0, self.a.1), b: Q(-self.b.0, self.b.1), d: self.d }
    }
}

impl Mul for Quad {
    type Output = Quad;
    fn mul(self, rhs: Quad) -> Quad {
        assert_eq!(self.d, rhs.d, "incompatible extensions: different d");
        // (a + b√d)(c + e√d) = (ac + be d) + (ae + bc)√d
        let ac = mul_q(self.a, rhs.a);
        let be = mul_q(self.b, rhs.b);
        let be_d = Q(be.0 * self.d, be.1);
        let ae = mul_q(self.a, rhs.b);
        let bc = mul_q(self.b, rhs.a);
        let real = add_q(ac, be_d);
        let imag = add_q(ae, bc);
        Quad { a: real, b: imag, d: self.d }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(n: i64, d: i64) -> Q {
        Q::new(n, d)
    }

    #[test]
    fn add_sub_mul_basic() {
        let d = 5;
        let x = Quad::new(q(1, 2), q(1, 3), d); // 1/2 + (1/3)√5
        let y = Quad::new(q(2, 3), q(1, 6), d); // 2/3 + (1/6)√5
        let s = x + y;
        assert_eq!(s.d, d);
        // crude structural checks
        assert_eq!(s.a, add_q(q(1, 2), q(2, 3)));
        assert_eq!(s.b, add_q(q(1, 3), q(1, 6)));

        let p = x * y;
        // p = (1/2*2/3 + 1/3*1/6*5) + ((1/2*1/6) + (1/3*2/3))√5
        let ac = mul_q(q(1, 2), q(2, 3));
        let be = mul_q(q(1, 3), q(1, 6));
        let be_d = Q(be.0 * d, be.1);
        let ae = mul_q(q(1, 2), q(1, 6));
        let bc = mul_q(q(1, 3), q(2, 3));
        assert_eq!(p.a, add_q(ac, be_d));
        assert_eq!(p.b, add_q(ae, bc));
    }

    #[test]
    fn norm_conjugate_identity() {
        let d = 3;
        let a = Quad::new(q(3, 4), q(2, 5), d);
        let c = a.conj();
        let n = a.norm();
        // a * conj(a) = norm(a) (as element of Q) = n + 0√d
        let prod = a * c;
        assert_eq!(prod.b, Q::zero());
        assert_eq!(prod.a, n);
    }

    #[test]
    fn from_int_and_neg() {
        let d = 2;
        let x = Quad::from_int(7, d);
        assert_eq!(x.a, Q::new(7, 1));
        assert_eq!(x.b, Q::zero());
        let y = -x;
        assert_eq!(y.a, Q::new(-7, 1));
        assert_eq!(y.b, Q::zero());
    }

    #[test]
    fn test_sub() {
        let d = 5;
        let x = Quad::new(q(3, 2), q(1, 3), d);
        let y = Quad::new(q(1, 2), q(1, 6), d);
        let diff = x - y;
        assert_eq!(diff.d, d);
        assert_eq!(diff.a, sub_q(q(3, 2), q(1, 2)));
        assert_eq!(diff.b, sub_q(q(1, 3), q(1, 6)));
    }

    #[test]
    fn test_inverse() {
        let d = 2;
        // (1 + √2)
        let x = Quad::new(q(1, 1), q(1, 1), d);
        let x_inv = x.inv().unwrap();

        // x * x^(-1) should equal 1
        let prod = x * x_inv;
        assert_eq!(prod.a, Q::new(1, 1));
        assert_eq!(prod.b, Q::zero());
    }

    #[test]
    fn test_inverse_zero() {
        let d = 3;
        let zero = Quad::new(Q::zero(), Q::zero(), d);
        assert!(zero.inv().is_none());
    }

    #[test]
    fn test_division() {
        let d = 5;
        let x = Quad::new(q(3, 1), q(2, 1), d); // 3 + 2√5
        let y = Quad::new(q(1, 1), q(1, 1), d); // 1 + √5

        let quotient = x.div(&y).unwrap();
        // Verify: quotient * y = x
        let prod = quotient * y;
        assert_eq!(prod.a, x.a);
        assert_eq!(prod.b, x.b);
    }

    #[test]
    fn test_trace() {
        let d = 7;
        let x = Quad::new(q(5, 2), q(3, 4), d); // 5/2 + (3/4)√7
        let tr = x.trace();
        // Trace = 2 * (5/2) = 5
        assert_eq!(tr, Q::new(5, 1));
    }

    #[test]
    fn test_is_rational() {
        let d = 3;
        let rational = Quad::new(q(7, 3), Q::zero(), d);
        let irrational = Quad::new(q(1, 1), q(1, 1), d);

        assert!(rational.is_rational());
        assert!(!irrational.is_rational());
    }

    #[test]
    fn test_as_rational() {
        let d = 11;
        let x = Quad::new(q(5, 2), Q::zero(), d);
        let y = Quad::new(q(1, 1), q(1, 1), d);

        assert_eq!(x.as_rational(), Some(q(5, 2)));
        assert_eq!(y.as_rational(), None);
    }

    #[test]
    fn test_field_axioms() {
        let d = 2;
        let x = Quad::new(q(1, 1), q(1, 1), d);
        let y = Quad::new(q(2, 1), q(1, 2), d);
        let z = Quad::new(q(1, 3), q(1, 4), d);

        // Associativity of addition
        let lhs = (x + y) + z;
        let rhs = x + (y + z);
        assert_eq!(lhs.a, rhs.a);
        assert_eq!(lhs.b, rhs.b);

        // Commutativity of multiplication
        let lhs = x * y;
        let rhs = y * x;
        assert_eq!(lhs.a, rhs.a);
        assert_eq!(lhs.b, rhs.b);

        // Distributivity
        let lhs = x * (y + z);
        let rhs = (x * y) + (x * z);
        assert_eq!(lhs.a, rhs.a);
        assert_eq!(lhs.b, rhs.b);
    }
}
