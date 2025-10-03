//! Polynomial types/algorithms (minimal v1).
//! - Univariate dense polynomials over Q (i64 rationals)
//! - Division with remainder, Euclidean GCD
//! - Conversions: Expr ⟷ Poly (for sums of monomials in a single symbol)

use arith::{add_q, div_q, mul_q, sub_q, Q};
use expr_core::{ExprId, Op, Payload, Store};

// ---------- Univariate dense polynomial over Q ----------

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UniPoly {
    pub var: String,
    // coeffs[k] is coefficient of x^k; no trailing zeros
    pub coeffs: Vec<Q>,
}

impl UniPoly {
    pub fn new<S: Into<String>>(var: S, mut coeffs: Vec<Q>) -> Self {
        trim_trailing_zeros(&mut coeffs);
        Self { var: var.into(), coeffs }
    }
    pub fn zero<S: Into<String>>(var: S) -> Self {
        Self { var: var.into(), coeffs: vec![] }
    }
    pub fn is_zero(&self) -> bool {
        self.coeffs.is_empty()
    }
    pub fn degree(&self) -> Option<usize> {
        if self.is_zero() {
            None
        } else {
            Some(self.coeffs.len() - 1)
        }
    }
    pub fn leading_coeff(&self) -> Q {
        if let Some(d) = self.degree() {
            self.coeffs[d]
        } else {
            Q::zero()
        }
    }

    pub fn add(&self, rhs: &Self) -> Self {
        assert_eq!(self.var, rhs.var);
        let mut coeffs = Vec::with_capacity(self.coeffs.len().max(rhs.coeffs.len()));
        for i in 0..self.coeffs.len().max(rhs.coeffs.len()) {
            let a = self.coeffs.get(i).copied().unwrap_or(Q::zero());
            let b = rhs.coeffs.get(i).copied().unwrap_or(Q::zero());
            coeffs.push(add_q(a, b));
        }
        Self::new(self.var.clone(), coeffs)
    }
    pub fn sub(&self, rhs: &Self) -> Self {
        assert_eq!(self.var, rhs.var);
        let mut coeffs = Vec::with_capacity(self.coeffs.len().max(rhs.coeffs.len()));
        for i in 0..self.coeffs.len().max(rhs.coeffs.len()) {
            let a = self.coeffs.get(i).copied().unwrap_or(Q::zero());
            let b = rhs.coeffs.get(i).copied().unwrap_or(Q::zero());
            coeffs.push(sub_q(a, b));
        }
        Self::new(self.var.clone(), coeffs)
    }
    pub fn mul(&self, rhs: &Self) -> Self {
        assert_eq!(self.var, rhs.var);
        if self.is_zero() || rhs.is_zero() {
            return Self::zero(&self.var);
        }
        let mut coeffs = vec![Q::zero(); self.coeffs.len() + rhs.coeffs.len() - 1];
        for (i, &a) in self.coeffs.iter().enumerate() {
            if a.is_zero() {
                continue;
            }
            for (j, &b) in rhs.coeffs.iter().enumerate() {
                if b.is_zero() {
                    continue;
                }
                coeffs[i + j] = add_q(coeffs[i + j], mul_q(a, b));
            }
        }
        Self::new(self.var.clone(), coeffs)
    }
    pub fn monic(&self) -> Self {
        if self.is_zero() {
            return self.clone();
        }
        let lc = self.leading_coeff();
        let inv = div_q(Q::one(), lc);
        let coeffs = self.coeffs.iter().map(|&c| mul_q(c, inv)).collect();
        Self::new(self.var.clone(), coeffs)
    }

    // Division with remainder: self = q*div + r, deg r < deg div
    pub fn div_rem(&self, div: &Self) -> Result<(Self, Self), &'static str> {
        assert_eq!(self.var, div.var);
        if div.is_zero() {
            return Err("division by zero polynomial");
        }
        let mut r = self.clone();
        let mut q = UniPoly::zero(&self.var);
        if r.is_zero() {
            return Ok((q, r));
        }
        let ddeg = div.degree().unwrap();
        let dlc = div.leading_coeff();
        while let Some(rdeg) = r.degree() {
            if rdeg < ddeg {
                break;
            }
            let shift = rdeg - ddeg;
            let coeff = div_q(r.leading_coeff(), dlc);
            // q += coeff * x^shift
            if q.coeffs.len() <= shift {
                q.coeffs.resize(shift + 1, Q::zero());
            }
            q.coeffs[shift] = add_q(q.coeffs[shift], coeff);
            // r -= (coeff * x^shift) * div
            let mut to_sub = vec![Q::zero(); shift + div.coeffs.len()];
            for (i, &c) in div.coeffs.iter().enumerate() {
                to_sub[shift + i] = mul_q(coeff, c);
            }
            r = r.sub(&UniPoly::new(self.var.clone(), to_sub));
            if r.is_zero() {
                break;
            }
        }
        Ok((q, r))
    }

    pub fn gcd(mut a: Self, mut b: Self) -> Self {
        assert_eq!(a.var, b.var);
        // Euclidean algorithm
        while !b.is_zero() {
            let r = a.div_rem(&b).expect("non-zero divisor").1;
            a = b;
            b = r;
        }
        a.monic()
    }
}

fn trim_trailing_zeros(v: &mut Vec<Q>) {
    while v.last().is_some_and(|c| c.is_zero()) {
        v.pop();
    }
}

// ---------- Expr ⟷ Poly conversions ----------

pub fn expr_to_unipoly(store: &Store, id: ExprId, var: &str) -> Option<UniPoly> {
    fn as_int(store: &Store, id: ExprId) -> Option<i64> {
        if let (Op::Integer, Payload::Int(k)) = (&store.get(id).op, &store.get(id).payload) {
            Some(*k)
        } else {
            None
        }
    }
    fn as_rat(store: &Store, id: ExprId) -> Option<Q> {
        match (&store.get(id).op, &store.get(id).payload) {
            (Op::Integer, Payload::Int(k)) => Some(Q(*k, 1)),
            (Op::Rational, Payload::Rat(n, d)) => Some(Q(*n, *d)),
            _ => None,
        }
    }
    fn as_symbol(store: &Store, id: ExprId, var: &str) -> bool {
        matches!((&store.get(id).op, &store.get(id).payload), (Op::Symbol, Payload::Sym(ref s)) if s==var)
    }

    // Decompose an expression into coeff * x^k if possible
    fn term_to_monomial(store: &Store, id: ExprId, var: &str) -> Option<(Q, usize)> {
        match store.get(id).op {
            Op::Integer | Op::Rational => as_rat(store, id).map(|q| (q, 0)),
            Op::Symbol => {
                if as_symbol(store, id, var) {
                    Some((Q(1, 1), 1))
                } else {
                    None
                }
            }
            Op::Pow => {
                let n = store.get(id);
                let base = n.children[0];
                let exp = n.children[1];
                if !as_symbol(store, base, var) {
                    return None;
                }
                let k = as_int(store, exp)?;
                if k < 0 {
                    return None;
                }
                Some((Q(1, 1), k as usize))
            }
            Op::Mul => {
                let mut coeff = Q::one();
                let mut k: usize = 0;
                for &f in &store.get(id).children {
                    if let Some(q) = as_rat(store, f) {
                        coeff = mul_q(coeff, q);
                        continue;
                    }
                    if as_symbol(store, f, var) {
                        k += 1;
                        continue;
                    }
                    if store.get(f).op == Op::Pow {
                        let b = store.get(f).children[0];
                        let e = store.get(f).children[1];
                        if !as_symbol(store, b, var) {
                            return None;
                        }
                        let kk = as_int(store, e)?;
                        if kk < 0 {
                            return None;
                        }
                        k += kk as usize;
                        continue;
                    }
                    return None;
                }
                Some((coeff, k))
            }
            _ => None,
        }
    }

    match store.get(id).op {
        Op::Integer | Op::Rational | Op::Symbol | Op::Pow | Op::Mul => {
            if let Some((q, k)) = term_to_monomial(store, id, var) {
                let mut coeffs = vec![Q::zero(); k + 1];
                coeffs[k] = q;
                return Some(UniPoly::new(var.to_string(), coeffs));
            }
            None
        }
        Op::Add => {
            let mut acc = UniPoly::zero(var.to_string());
            for &t in &store.get(id).children {
                let mono = expr_to_unipoly(store, t, var)?;
                acc = acc.add(&mono);
            }
            Some(acc)
        }
        _ => None,
    }
}

pub fn unipoly_to_expr(store: &mut Store, p: &UniPoly) -> ExprId {
    if p.is_zero() {
        return store.int(0);
    }
    let mut terms: Vec<ExprId> = Vec::new();
    let x = store.sym(&p.var);
    for (k, &q) in p.coeffs.iter().enumerate() {
        if q.is_zero() {
            continue;
        }
        let coeff = if q.1 == 1 { store.int(q.0) } else { store.rat(q.0, q.1) };
        let term = if k == 0 {
            coeff
        } else {
            let kint = store.int(k as i64);
            let pow = store.pow(x, kint);
            store.mul(vec![coeff, pow])
        };
        terms.push(term);
    }
    store.add(terms)
}

// ---------- Tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unipoly_division_and_gcd() {
        // (x^2 + 3x + 2) / (x + 1) = x + 2, r = 0; gcd(x^2-1, x^2-x) = x-1
        let var = "x";
        let p = UniPoly::new(var, vec![Q(2, 1), Q(3, 1), Q(1, 1)]);
        let d = UniPoly::new(var, vec![Q(1, 1), Q(1, 1)]);
        let (q, r) = p.div_rem(&d).unwrap();
        assert!(r.is_zero());
        assert_eq!(q, UniPoly::new(var, vec![Q(2, 1), Q(1, 1)]));

        let p1 = UniPoly::new(var, vec![Q(-1, 1), Q(0, 1), Q(1, 1)]); // x^2 - 1
        let p2 = UniPoly::new(var, vec![Q(0, 1), Q(-1, 1), Q(1, 1)]); // x^2 - x
        let g = UniPoly::gcd(p1, p2);
        assert_eq!(g, UniPoly::new(var, vec![Q(-1, 1), Q(1, 1)]).monic()); // x - 1
    }

    #[test]
    fn expr_poly_roundtrip() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let p2 = st.pow(x, two);
        let three = st.int(3);
        let three_x = st.mul(vec![three, x]);
        let two2 = st.int(2);
        let expr = st.add(vec![p2, three_x, two2]);
        let p = expr_to_unipoly(&st, expr, "x").expect("convertible");
        assert_eq!(p, UniPoly::new("x", vec![Q(2, 1), Q(3, 1), Q(1, 1)]));
        let back = unipoly_to_expr(&mut st, &p);
        assert_eq!(back, expr);
    }
}
