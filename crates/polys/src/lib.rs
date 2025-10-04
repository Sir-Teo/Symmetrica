//! Polynomial types/algorithms (minimal v1).
//! - Univariate dense polynomials over Q (i64 rationals)
//! - Division with remainder, Euclidean GCD
//! - Conversions: Expr ⟷ Poly (for sums of monomials in a single symbol)

use arith::{add_q, div_q, gcd_i64, mul_q, sub_q, Q};
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

    pub fn deriv(&self) -> Self {
        if self.coeffs.len() <= 1 {
            return Self::zero(self.var.clone());
        }
        let mut out: Vec<Q> = Vec::with_capacity(self.coeffs.len() - 1);
        for (k, &c) in self.coeffs.iter().enumerate().skip(1) {
            // d/dx c_k x^k = (k) * c_k x^{k-1}
            let factor = Q(k as i64, 1);
            out.push(mul_q(c, factor));
        }
        Self::new(self.var.clone(), out)
    }

    pub fn eval_q(&self, x: Q) -> Q {
        // Horner's method
        let mut acc = Q::zero();
        for &c in self.coeffs.iter().rev() {
            acc = add_q(mul_q(acc, x), c);
        }
        acc
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

/// Partial fractions for denominators that factor into distinct linear factors over Q.
/// Returns (quotient, terms), where terms are (A_i, r_i) representing A_i/(x - r_i).
/// Only handles the simple case (no repeated factors). Returns None if factoring fails.
pub fn partial_fractions_simple(num: &UniPoly, den: &UniPoly) -> Option<(UniPoly, Vec<(Q, Q)>)> {
    if num.var != den.var {
        return None;
    }

    // Long division to extract polynomial part.
    let (q, r) = num.div_rem(den).ok()?;

    // Factor denominator into distinct rational linear factors using Rational Root Theorem.
    fn lcm_i64(a: i64, b: i64) -> i64 {
        if a == 0 || b == 0 {
            return 0;
        }
        (a / gcd_i64(a.abs(), b.abs())) * b
    }
    fn clear_denominators(p: &UniPoly) -> (Vec<i64>, i64) {
        let mut l = 1i64;
        for &Q(_, d) in &p.coeffs {
            let dd = d.abs().max(1);
            l = if l == 0 { dd } else { lcm_i64(l, dd) };
        }
        let mut ints = Vec::with_capacity(p.coeffs.len());
        for &Q(n, d) in &p.coeffs {
            ints.push(n * (if d == 0 { 0 } else { l / d }));
        }
        (ints, l)
    }
    fn divisors(mut n: i64) -> Vec<i64> {
        if n < 0 {
            n = -n;
        }
        if n == 0 {
            // convention: only 0; callers handle specially
            return vec![0];
        }
        let mut ds = Vec::new();
        let mut i = 1;
        while (i as i128) * (i as i128) <= (n as i128) {
            if n % i == 0 {
                ds.push(i);
                if i != n / i {
                    ds.push(n / i);
                }
            }
            i += 1;
        }
        ds
    }
    fn deflate_by_root(p: &UniPoly, r: Q) -> Option<UniPoly> {
        let var = p.var.clone();
        let mut new_coeffs: Vec<Q> = Vec::with_capacity(p.coeffs.len().saturating_sub(1));
        let mut acc = Q::zero();
        for &c in p.coeffs.iter().rev() {
            acc = add_q(mul_q(acc, r), c);
            new_coeffs.push(acc);
        }
        if !acc.is_zero() {
            return None;
        }
        new_coeffs.pop();
        new_coeffs.reverse();
        Some(UniPoly::new(var, new_coeffs))
    }

    // Collect distinct rational roots (with multiplicity 1) by repeated deflation.
    let mut den_work = den.clone();
    let mut roots: Vec<Q> = Vec::new();
    loop {
        match den_work.degree() {
            None | Some(0) => break,
            Some(1) => {
                // ax + b => root = -b/a
                let a = den_work.coeffs.get(1).copied().unwrap_or(Q::zero());
                let b = den_work.coeffs.first().copied().unwrap_or(Q::zero());
                if a.is_zero() {
                    return None;
                }
                let root = div_q(Q(-b.0, b.1), a);
                roots.push(root);
                break;
            }
            Some(_) => {
                let (ints, _) = clear_denominators(&den_work);
                let lc = *ints.last().unwrap_or(&0);
                let ct = *ints.first().unwrap_or(&0);
                let mut found = None;
                'outer: for qd in divisors(lc).into_iter().flat_map(|q| vec![q, -q]) {
                    if qd == 0 {
                        continue;
                    }
                    for pn in divisors(ct).into_iter().flat_map(|pn| vec![pn, -pn]) {
                        let r = Q(pn, qd);
                        if den_work.eval_q(r).is_zero() {
                            found = Some(r);
                            break 'outer;
                        }
                    }
                }
                let r = found?;
                roots.push(r);
                den_work = deflate_by_root(&den_work, r)?;
            }
        }
    }

    // Ensure distinct (no repeated roots): derivative at each root must be non-zero.
    let dprime = den.deriv();
    for &rv in &roots {
        if dprime.eval_q(rv).is_zero() {
            return None;
        }
    }

    // Compute residues A_i = r(root_i) / den'(root_i)
    let mut terms: Vec<(Q, Q)> = Vec::with_capacity(roots.len());
    for &rv in &roots {
        let numv = r.eval_q(rv);
        let denv = dprime.eval_q(rv);
        if denv.is_zero() {
            return None;
        }
        let a = div_q(numv, denv);
        terms.push((a, rv));
    }

    Some((q, terms))
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

    #[test]
    fn partial_fractions_simple_linear_denominator() {
        // (2x+3)/(x^2+3x+2) = 1/(x+1) + 1/(x+2)
        let var = "x";
        let num = UniPoly::new(var, vec![Q(3, 1), Q(2, 1)]); // 3 + 2x
        let den = UniPoly::new(var, vec![Q(2, 1), Q(3, 1), Q(1, 1)]); // 2 + 3x + x^2
        let (q, terms) = partial_fractions_simple(&num, &den).expect("pf");
        assert!(q.is_zero());
        assert_eq!(terms.len(), 2);
        let mut ok1 = false;
        let mut ok2 = false;
        for (a, r) in terms {
            // (A, root)
            if r == Q(-1, 1) {
                assert_eq!(a, Q(1, 1));
                ok1 = true;
            } else if r == Q(-2, 1) {
                assert_eq!(a, Q(1, 1));
                ok2 = true;
            }
        }
        assert!(ok1 && ok2);
    }

    #[test]
    fn partial_fractions_improper_fraction() {
        // x^3 / (x+1) has quotient x^2 - x + 1 and remainder -1
        let var = "x";
        let num = UniPoly::new(var, vec![Q(0, 1), Q(0, 1), Q(0, 1), Q(1, 1)]);
        let den = UniPoly::new(var, vec![Q(1, 1), Q(1, 1)]);
        let (q, terms) = partial_fractions_simple(&num, &den).expect("pf");
        assert_eq!(q.degree(), Some(2));
        assert_eq!(terms.len(), 1);
        assert_eq!(terms[0].1, Q(-1, 1)); // root at -1
    }

    #[test]
    fn partial_fractions_mismatched_vars() {
        let num = UniPoly::new("x", vec![Q(1, 1)]);
        let den = UniPoly::new("y", vec![Q(1, 1), Q(1, 1)]);
        assert!(partial_fractions_simple(&num, &den).is_none());
    }

    #[test]
    fn partial_fractions_repeated_root_returns_none() {
        // (x+1) / (x+1)^2 has a repeated root, not supported
        let var = "x";
        let num = UniPoly::new(var, vec![Q(1, 1), Q(1, 1)]);
        let den = UniPoly::new(var, vec![Q(1, 1), Q(2, 1), Q(1, 1)]); // (x+1)^2 = x^2 + 2x + 1
                                                                      // This should detect that the same root appears twice
        let result = partial_fractions_simple(&num, &den);
        // The function will try to deflate and fail to find distinct roots
        // Since (x+1)^2 will yield root -1 once, then deflating again gives (x+1) again => same root
        // We test that it returns None (cannot factor into distinct linear terms)
        assert!(result.is_none());
    }

    #[test]
    fn partial_fractions_no_rational_roots() {
        // x^2 + 1 has no rational roots
        let var = "x";
        let num = UniPoly::new(var, vec![Q(1, 1)]);
        let den = UniPoly::new(var, vec![Q(1, 1), Q(0, 1), Q(1, 1)]); // 1 + x^2
        assert!(partial_fractions_simple(&num, &den).is_none());
    }

    #[test]
    fn unipoly_zero_and_degree() {
        let p = UniPoly::zero("x");
        assert!(p.is_zero());
        assert_eq!(p.degree(), None);
        assert_eq!(p.leading_coeff(), Q::zero());
    }

    #[test]
    fn unipoly_deriv() {
        let p = UniPoly::new("x", vec![Q(2, 1), Q(3, 1), Q(1, 1)]);
        let dp = p.deriv();
        assert_eq!(dp.coeffs, vec![Q(3, 1), Q(2, 1)]);
    }

    #[test]
    fn unipoly_eval() {
        let p = UniPoly::new("x", vec![Q(1, 1), Q(2, 1), Q(1, 1)]);
        let v = p.eval_q(Q(2, 1));
        assert_eq!(v, Q(9, 1));
    }

    #[test]
    fn unipoly_add_different_lengths() {
        let p1 = UniPoly::new("x", vec![Q(1, 1)]);
        let p2 = UniPoly::new("x", vec![Q(1, 1), Q(1, 1), Q(1, 1)]);
        let sum = p1.add(&p2);
        assert_eq!(sum.coeffs.len(), 3);
    }

    #[test]
    fn unipoly_sub() {
        let p1 = UniPoly::new("x", vec![Q(5, 1), Q(3, 1)]);
        let p2 = UniPoly::new("x", vec![Q(2, 1), Q(1, 1)]);
        let diff = p1.sub(&p2);
        assert_eq!(diff.coeffs, vec![Q(3, 1), Q(2, 1)]);
    }

    #[test]
    fn unipoly_mul_with_zero() {
        let p1 = UniPoly::new("x", vec![Q(1, 1), Q(2, 1)]);
        let p2 = UniPoly::zero("x");
        let prod = p1.mul(&p2);
        assert!(prod.is_zero());
    }

    #[test]
    fn unipoly_div_rem_by_zero() {
        let p = UniPoly::new("x", vec![Q(1, 1)]);
        let z = UniPoly::zero("x");
        let res = p.div_rem(&z);
        assert!(res.is_err());
    }

    #[test]
    fn expr_to_unipoly_rational_coeff() {
        let mut st = Store::new();
        let x = st.sym("x");
        let half = st.rat(1, 2);
        let expr = st.mul(vec![half, x]);
        let p = expr_to_unipoly(&st, expr, "x").expect("poly");
        assert_eq!(p.coeffs[1], Q(1, 2));
    }

    #[test]
    fn expr_to_unipoly_pow_negative_fails() {
        let mut st = Store::new();
        let x = st.sym("x");
        let m1 = st.int(-1);
        let expr = st.pow(x, m1);
        let p = expr_to_unipoly(&st, expr, "x");
        assert!(p.is_none());
    }

    #[test]
    fn expr_to_unipoly_wrong_var() {
        let mut st = Store::new();
        let y = st.sym("y");
        let p = expr_to_unipoly(&st, y, "x");
        assert!(p.is_none());
    }

    #[test]
    fn expr_to_unipoly_function_fails() {
        let mut st = Store::new();
        let x = st.sym("x");
        let sinx = st.func("sin", vec![x]);
        let p = expr_to_unipoly(&st, sinx, "x");
        assert!(p.is_none());
    }

    #[test]
    fn unipoly_to_expr_zero() {
        let mut st = Store::new();
        let p = UniPoly::zero("x");
        let e = unipoly_to_expr(&mut st, &p);
        assert_eq!(e, st.int(0));
    }

    #[test]
    fn unipoly_monic() {
        let p = UniPoly::new("x", vec![Q(2, 1), Q(4, 1)]);
        let m = p.monic();
        assert_eq!(m.leading_coeff(), Q(1, 1));
    }

    #[test]
    fn unipoly_monic_zero() {
        let p = UniPoly::zero("x");
        let m = p.monic();
        assert!(m.is_zero());
    }
}
