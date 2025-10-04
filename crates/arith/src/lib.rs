//! arith: shared small rational arithmetic utilities over i64.
//! - Tuple-based rationals: (num, den) with helpers q_* and rat_*
//! - Newtype `Q(i64, i64)` for use in polynomial code
//!
//!   All rationals are normalized with den>0 and gcd(|num|, den)=1.

#![deny(warnings)]

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Q(pub i64, pub i64);

impl Q {
    pub fn new(num: i64, den: i64) -> Self {
        let (n, d) = normalize_rat(num, den);
        Q(n, d)
    }
    pub fn zero() -> Self {
        Q(0, 1)
    }
    pub fn one() -> Self {
        Q(1, 1)
    }
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

// ---------- Integer gcd ----------
pub fn gcd_i64(mut a: i64, mut b: i64) -> i64 {
    if a == 0 {
        return b.abs();
    }
    if b == 0 {
        return a.abs();
    }
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a.abs()
}

// ---------- Tuple rational helpers (aliases provided for compatibility) ----------
/// Normalize (num, den) to gcd-reduced with den>0
pub fn normalize_rat(num: i64, den: i64) -> (i64, i64) {
    assert!(den != 0, "zero denominator");
    let mut n = num;
    let mut d = den;
    if d < 0 {
        n = -n;
        d = -d;
    }
    if n == 0 {
        return (0, 1);
    }
    let g = gcd_i64(n.abs(), d);
    (n / g, d / g)
}
/// Add two rationals (num,den)
pub fn rat_add(a: (i64, i64), b: (i64, i64)) -> (i64, i64) {
    normalize_rat(a.0 * b.1 + b.0 * a.1, a.1 * b.1)
}
/// Multiply two rationals (num,den)
pub fn rat_mul(a: (i64, i64), b: (i64, i64)) -> (i64, i64) {
    normalize_rat(a.0 * b.0, a.1 * b.1)
}
/// Subtract two rationals (num,den)
pub fn rat_sub(a: (i64, i64), b: (i64, i64)) -> (i64, i64) {
    rat_add(a, (-b.0, b.1))
}

// Prefer q_* naming in calculus; keep both for ergonomic use.
pub fn q_norm(n: i64, d: i64) -> (i64, i64) {
    normalize_rat(n, d)
}
pub fn q_add(a: (i64, i64), b: (i64, i64)) -> (i64, i64) {
    rat_add(a, b)
}
pub fn q_sub(a: (i64, i64), b: (i64, i64)) -> (i64, i64) {
    rat_sub(a, b)
}
pub fn q_mul(a: (i64, i64), b: (i64, i64)) -> (i64, i64) {
    rat_mul(a, b)
}
pub fn q_div(a: (i64, i64), b: (i64, i64)) -> (i64, i64) {
    normalize_rat(a.0 * b.1, a.1 * b.0)
}

// ---------- Q arithmetic helpers ----------
pub fn add_q(a: Q, b: Q) -> Q {
    let (n, d) = rat_add((a.0, a.1), (b.0, b.1));
    Q(n, d)
}
pub fn sub_q(a: Q, b: Q) -> Q {
    let (n, d) = rat_sub((a.0, a.1), (b.0, b.1));
    Q(n, d)
}
pub fn mul_q(a: Q, b: Q) -> Q {
    let (n, d) = rat_mul((a.0, a.1), (b.0, b.1));
    Q(n, d)
}
pub fn div_q(a: Q, b: Q) -> Q {
    let (n, d) = q_div((a.0, a.1), (b.0, b.1));
    Q(n, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gcd_zero_cases() {
        assert_eq!(gcd_i64(0, 5), 5);
        assert_eq!(gcd_i64(5, 0), 5);
        assert_eq!(gcd_i64(0, 0), 0);
    }

    #[test]
    fn gcd_negative() {
        assert_eq!(gcd_i64(-12, 8), 4);
        assert_eq!(gcd_i64(12, -8), 4);
    }

    #[test]
    fn normalize_negative_den() {
        let (n, d) = normalize_rat(3, -4);
        assert_eq!(n, -3);
        assert_eq!(d, 4);
    }

    #[test]
    fn q_operations() {
        assert_eq!(q_norm(4, 6), (2, 3));
        assert_eq!(q_add((1, 3), (1, 6)), (1, 2));
        assert_eq!(q_sub((1, 2), (1, 3)), (1, 6));
        assert_eq!(q_mul((2, 3), (3, 4)), (1, 2));
        assert_eq!(q_div((1, 2), (1, 4)), (2, 1));
    }

    #[test]
    fn q_struct_methods() {
        let q = Q::new(6, 9);
        assert_eq!(q, Q(2, 3));
        assert!(!q.is_zero());
        assert!(Q::zero().is_zero());
        assert_eq!(Q::one(), Q(1, 1));
    }
}
