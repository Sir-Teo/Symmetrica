#![deny(warnings)]

//! algebraic: minimal quadratic extension Q(√d)
//! Representation: a + b√d with a,b in Q and squarefree integer d (not enforced).

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

    fn q(n: i64, d: i64) -> Q { Q::new(n, d) }

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
}
