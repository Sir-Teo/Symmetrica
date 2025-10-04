//! Power series utilities and Maclaurin expansions.

use arith::{q_add, q_div, q_mul, q_norm, q_sub};
use expr_core::{ExprId, Op, Payload, Store};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Series {
    // coeffs[k] = coefficient of x^k, each as reduced rational (num, den), den>0
    pub coeffs: Vec<(i64, i64)>,
}

impl Series {
    pub fn zero(order: usize) -> Self {
        Self { coeffs: vec![(0, 1); order + 1] }
    }

    pub fn one(order: usize) -> Self {
        let mut c = vec![(0, 1); order + 1];
        c[0] = (1, 1);
        Self { coeffs: c }
    }

    pub fn const_q(num: i64, den: i64, order: usize) -> Self {
        let mut c = vec![(0, 1); order + 1];
        c[0] = q_norm(num, den);
        Self { coeffs: c }
    }

    pub fn x(order: usize) -> Self {
        let mut c = vec![(0, 1); order + 1];
        if order >= 1 {
            c[1] = (1, 1);
        }
        Self { coeffs: c }
    }

    pub fn truncate(mut self, order: usize) -> Self {
        self.coeffs.truncate(order + 1);
        while self.coeffs.last().is_some_and(|c| c.0 == 0) && self.coeffs.len() > 1 {
            self.coeffs.pop();
        }
        self
    }

    pub fn add(&self, rhs: &Self, order: usize) -> Self {
        let n = order + 1;
        let mut out = vec![(0, 1); n];
        for (k, out_k) in out.iter_mut().enumerate().take(n) {
            let a = self.coeffs.get(k).copied().unwrap_or((0, 1));
            let b = rhs.coeffs.get(k).copied().unwrap_or((0, 1));
            *out_k = q_add(a, b);
        }
        Self { coeffs: out }
    }

    pub fn sub(&self, rhs: &Self, order: usize) -> Self {
        let n = order + 1;
        let mut out = vec![(0, 1); n];
        for (k, out_k) in out.iter_mut().enumerate().take(n) {
            let a = self.coeffs.get(k).copied().unwrap_or((0, 1));
            let b = rhs.coeffs.get(k).copied().unwrap_or((0, 1));
            *out_k = q_sub(a, b);
        }
        Self { coeffs: out }
    }

    pub fn mul(&self, rhs: &Self, order: usize) -> Self {
        let n = order + 1;
        let mut out = vec![(0, 1); n];
        for (i, out_i) in out.iter_mut().enumerate().take(n) {
            for j in 0..=i {
                let a = self.coeffs.get(j).copied().unwrap_or((0, 1));
                let b = rhs.coeffs.get(i - j).copied().unwrap_or((0, 1));
                *out_i = q_add(*out_i, q_mul(a, b));
            }
        }
        Self { coeffs: out }
    }

    // Compose s(inner): requires inner.c0 == 0
    pub fn compose(&self, inner: &Self, order: usize) -> Option<Self> {
        if inner.coeffs.first().copied().unwrap_or((0, 1)) != (0, 1) {
            return None;
        }
        let n = order + 1;
        let mut out = Series::zero(order);
        // p = inner^k
        let mut p = Series::one(order);
        for k in 0..n {
            let a_k = self.coeffs.get(k).copied().unwrap_or((0, 1));
            if a_k.0 != 0 {
                let term = p.scale(a_k, order);
                out = out.add(&term, order);
            }
            p = p.mul(inner, order);
        }
        Some(out)
    }

    pub fn scale(&self, q: (i64, i64), order: usize) -> Self {
        let n = order + 1;
        let mut out = vec![(0, 1); n];
        for (k, out_k) in out.iter_mut().enumerate().take(n) {
            *out_k = q_mul(self.coeffs.get(k).copied().unwrap_or((0, 1)), q);
        }
        Self { coeffs: out }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LimitPoint {
    Zero,
    PosInf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LimitResult {
    Finite((i64, i64)),
    Infinity,
    Indeterminate,
    Unsupported,
}

/// Try to compute limit for polynomial-like expressions in `var`.
/// Supported:
/// - point = Zero: returns constant term c0 as rational.
/// - point = PosInf: if degree==0 returns constant; if degree>0 returns Infinity.
pub fn limit_poly(store: &Store, id: ExprId, var: &str, point: LimitPoint) -> LimitResult {
    fn const_term(store: &Store, id: ExprId, var: &str) -> Option<(i64, i64)> {
        match (&store.get(id).op, &store.get(id).payload) {
            (Op::Integer, Payload::Int(k)) => Some(((*k), 1)),
            (Op::Rational, Payload::Rat(n, d)) => Some(((*n), (*d))),
            (Op::Symbol, Payload::Sym(s)) => {
                if s == var {
                    Some((0, 1))
                } else {
                    None
                }
            }
            (Op::Add, _) => {
                let mut acc = (0, 1);
                for &c in &store.get(id).children {
                    let ct = const_term(store, c, var)?;
                    acc = q_add(acc, ct);
                }
                Some(acc)
            }
            (Op::Mul, _) => {
                let mut acc = (1, 1);
                for &f in &store.get(id).children {
                    let ct = const_term(store, f, var)?;
                    acc = q_mul(acc, ct);
                }
                Some(acc)
            }
            (Op::Pow, _) => {
                let n = store.get(id);
                let base = n.children[0];
                let exp = n.children[1];
                match (&store.get(exp).op, &store.get(exp).payload) {
                    (Op::Integer, Payload::Int(k)) if *k >= 0 => {
                        let ct = const_term(store, base, var)?;
                        // ct^k
                        let mut acc = (1, 1);
                        for _ in 0..(*k as usize) {
                            acc = q_mul(acc, ct);
                        }
                        Some(acc)
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn degree(store: &Store, id: ExprId, var: &str) -> Option<isize> {
        match (&store.get(id).op, &store.get(id).payload) {
            (Op::Integer, Payload::Int(k)) => Some(if *k == 0 { -1 } else { 0 }),
            (Op::Rational, Payload::Rat(n, _)) => Some(if *n == 0 { -1 } else { 0 }),
            (Op::Symbol, Payload::Sym(s)) => {
                if s == var {
                    Some(1)
                } else {
                    None
                }
            }
            (Op::Add, _) => {
                let mut deg = -1;
                for &c in &store.get(id).children {
                    let cd = degree(store, c, var)?;
                    if cd > deg {
                        deg = cd;
                    }
                }
                Some(deg)
            }
            (Op::Mul, _) => {
                let mut deg = 0isize;
                for &f in &store.get(id).children {
                    let fd = degree(store, f, var)?;
                    if fd < 0 {
                        return Some(-1);
                    }
                    deg += fd;
                }
                Some(deg)
            }
            (Op::Pow, _) => {
                let n = store.get(id);
                let base = n.children[0];
                let exp = n.children[1];
                match (&store.get(exp).op, &store.get(exp).payload) {
                    (Op::Integer, Payload::Int(k)) if *k >= 0 => {
                        let bd = degree(store, base, var)?;
                        if bd < 0 {
                            Some(-1)
                        } else {
                            Some(bd * (*k as isize))
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    match point {
        LimitPoint::Zero => match const_term(store, id, var) {
            Some(ct) => LimitResult::Finite(q_norm(ct.0, ct.1)),
            None => LimitResult::Unsupported,
        },
        LimitPoint::PosInf => match degree(store, id, var) {
            Some(d) if d < 0 => LimitResult::Finite((0, 1)),
            Some(0) => match const_term(store, id, var) {
                Some(ct) => LimitResult::Finite(q_norm(ct.0, ct.1)),
                None => LimitResult::Unsupported,
            },
            Some(_) => LimitResult::Infinity,
            None => LimitResult::Unsupported,
        },
    }
}

/// Maclaurin series up to `order` (inclusive) for a subset of expressions.
/// Restrictions:
/// - Only supports one variable `var`.
/// - For `exp(u)`, `sin(u)`, `cos(u)`: requires u(0) = 0 for composition.
/// - For `ln(u)`: requires u(0) = 1.
pub fn maclaurin(store: &Store, id: ExprId, var: &str, order: usize) -> Option<Series> {
    match store.get(id).op {
        Op::Integer => {
            if let Payload::Int(k) = store.get(id).payload {
                Some(Series::const_q(k, 1, order))
            } else {
                None
            }
        }
        Op::Rational => {
            if let Payload::Rat(n, d) = store.get(id).payload {
                Some(Series::const_q(n, d, order))
            } else {
                None
            }
        }
        Op::Symbol => match &store.get(id).payload {
            Payload::Sym(s) if s == var => Some(Series::x(order)),
            _ => None,
        },
        Op::Add => {
            let mut acc = Series::zero(order);
            for &c in &store.get(id).children {
                let sc = maclaurin(store, c, var, order)?;
                acc = acc.add(&sc, order);
            }
            Some(acc)
        }
        Op::Mul => {
            let mut prod = Series::one(order);
            for &f in &store.get(id).children {
                let sf = maclaurin(store, f, var, order)?;
                prod = prod.mul(&sf, order);
            }
            Some(prod)
        }
        Op::Pow => {
            let base = store.get(id).children[0];
            let exp = store.get(id).children[1];
            let k = match (&store.get(exp).op, &store.get(exp).payload) {
                (Op::Integer, Payload::Int(m)) if *m >= 0 => *m as usize,
                _ => return None,
            };
            let mut s = Series::one(order);
            let b = maclaurin(store, base, var, order)?;
            for _ in 0..k {
                s = s.mul(&b, order);
            }
            Some(s)
        }
        Op::Function => {
            // Single-arg functions
            let n = store.get(id);
            let fname = match &n.payload {
                Payload::Func(s) => s.as_str(),
                _ => return None,
            };
            if n.children.len() != 1 {
                return None;
            }
            let u = n.children[0];
            let su = maclaurin(store, u, var, order)?;
            match fname {
                "exp" => {
                    if su.coeffs.first().copied().unwrap_or((0, 1)) != (0, 1) {
                        return None;
                    }
                    let mut base = Series::zero(order);
                    base.coeffs = (0..=order)
                        .map(|k| (1i64, factorial(k as u32) as i64))
                        .map(|(n, d)| q_div((n, 1), (d, 1)))
                        .collect();
                    base.compose(&su, order)
                }
                "sin" => {
                    if su.coeffs.first().copied().unwrap_or((0, 1)) != (0, 1) {
                        return None;
                    }
                    let mut base = Series::zero(order);
                    base.coeffs = vec![(0, 1); order + 1];
                    for m in 0..=order {
                        let p = 2 * m + 1;
                        if p > order {
                            break;
                        }
                        let sign = if m % 2 == 0 { 1 } else { -1 };
                        base.coeffs[p] = q_div((sign, 1), (factorial(p as u32) as i64, 1));
                    }
                    base.compose(&su, order)
                }
                "cos" => {
                    if su.coeffs.first().copied().unwrap_or((0, 1)) != (0, 1) {
                        return None;
                    }
                    let mut base = Series::zero(order);
                    base.coeffs = vec![(0, 1); order + 1];
                    base.coeffs[0] = (1, 1);
                    for m in 0..=order {
                        let p = 2 * m;
                        if p == 0 {
                            continue;
                        }
                        if p > order {
                            break;
                        }
                        let sign = if m % 2 == 0 { 1 } else { -1 };
                        base.coeffs[p] = q_div((sign, 1), (factorial(p as u32) as i64, 1));
                    }
                    base.compose(&su, order)
                }
                "ln" | "log" => {
                    if su.coeffs.first().copied().unwrap_or((0, 1)) != (1, 1) {
                        return None;
                    }
                    let one = Series::one(order);
                    let v = su.sub(&one, order);
                    let mut out = Series::zero(order);
                    let mut pow = Series::one(order);
                    for k in 1..=order {
                        pow = if k == 1 { v.clone() } else { pow.mul(&v, order) };
                        let sign = if k % 2 == 1 { 1 } else { -1 };
                        let coeff = q_div((sign, 1), (k as i64, 1));
                        out = out.add(&pow.scale(coeff, order), order);
                    }
                    Some(out)
                }
                _ => None,
            }
        }
    }
}

fn factorial(n: u32) -> u128 {
    (1..=n as u128).product::<u128>().max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn series_basic_ops() {
        let s1 = Series::const_q(2, 1, 3);
        let s2 = Series::x(3);
        let sum = s1.add(&s2, 3);
        assert_eq!(sum.coeffs[0], (2, 1));
        assert_eq!(sum.coeffs[1], (1, 1));
        let prod = s1.mul(&s2, 3);
        assert_eq!(prod.coeffs[1], (2, 1));
    }

    #[test]
    fn series_sub() {
        let s1 = Series::const_q(5, 1, 2);
        let s2 = Series::const_q(3, 1, 2);
        let diff = s1.sub(&s2, 2);
        assert_eq!(diff.coeffs[0], (2, 1));
    }

    #[test]
    fn series_scale() {
        let s = Series::x(2);
        let scaled = s.scale((3, 2), 2);
        assert_eq!(scaled.coeffs[0], (0, 1));
        assert_eq!(scaled.coeffs[1], (3, 2));
    }

    #[test]
    fn series_truncate() {
        let mut s = Series::x(5);
        s.coeffs.push((0, 1));
        s.coeffs.push((0, 1));
        let t = s.truncate(2);
        assert!(t.coeffs.len() <= 3);
    }

    #[test]
    fn series_compose_requires_zero_const() {
        let s = Series::const_q(1, 1, 3);
        let inner = Series::const_q(1, 1, 3);
        let res = s.compose(&inner, 3);
        assert!(res.is_none());
    }

    #[test]
    fn maclaurin_mul() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let expr = st.mul(vec![two, x]);
        let s = maclaurin(&st, expr, "x", 3).expect("2x series");
        assert_eq!(s.coeffs[0], (0, 1));
        assert_eq!(s.coeffs[1], (2, 1));
    }

    #[test]
    fn maclaurin_pow() {
        let mut st = Store::new();
        let x = st.sym("x");
        let one = st.int(1);
        let two = st.int(2);
        let xp1 = st.add(vec![x, one]);
        let pow = st.pow(xp1, two);
        let s = maclaurin(&st, pow, "x", 3).expect("(x+1)^2");
        assert_eq!(s.coeffs[0], (1, 1));
    }

    #[test]
    fn maclaurin_negative_exponent_fails() {
        let mut st = Store::new();
        let x = st.sym("x");
        let m1 = st.int(-1);
        let pow = st.pow(x, m1);
        let s = maclaurin(&st, pow, "x", 3);
        assert!(s.is_none());
    }

    #[test]
    fn maclaurin_log_requires_one_at_zero() {
        let mut st = Store::new();
        let x = st.sym("x");
        let lnx = st.func("ln", vec![x]);
        let s = maclaurin(&st, lnx, "x", 3);
        assert!(s.is_none());
    }

    #[test]
    fn limit_poly_constant() {
        let mut st = Store::new();
        let five = st.int(5);
        let l = limit_poly(&st, five, "x", LimitPoint::Zero);
        assert_eq!(l, LimitResult::Finite((5, 1)));
    }

    #[test]
    fn limit_poly_rational_coeff() {
        let mut st = Store::new();
        let x = st.sym("x");
        let half = st.rat(1, 2);
        let expr = st.mul(vec![half, x]);
        let l = limit_poly(&st, expr, "x", LimitPoint::Zero);
        assert_eq!(l, LimitResult::Finite((0, 1)));
    }

    #[test]
    fn limit_poly_unsupported() {
        let mut st = Store::new();
        let x = st.sym("x");
        let sinx = st.func("sin", vec![x]);
        let l = limit_poly(&st, sinx, "x", LimitPoint::Zero);
        assert_eq!(l, LimitResult::Unsupported);
    }
}
