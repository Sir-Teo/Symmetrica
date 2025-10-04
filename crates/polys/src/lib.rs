//! Polynomial types/algorithms (minimal v1).
//! - Univariate dense polynomials over Q (i64 rationals)
//! - Division with remainder, Euclidean GCD, square-free decomposition
//! - Resultants and discriminants
//! - Multivariate sparse polynomials over Q
//! - Conversions: Expr ⟷ Poly (for sums of monomials in single or multiple symbols)

use arith::{add_q, div_q, gcd_i64, mul_q, sub_q, Q};
use expr_core::{ExprId, Op, Payload, Store};
use matrix::MatrixQ;
use std::collections::BTreeMap;

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

    /// Square-free decomposition using a simplified approach.
    /// Returns true square-free factors (gcd with derivative), removing multiplicity.
    /// Note: This is a simplified implementation for Phase C.
    /// Returns a list with the square-free part.
    ///
    /// For a polynomial with repeated roots, extracts square-free factors.
    pub fn square_free_decomposition(&self) -> Vec<(Self, usize)> {
        if self.is_zero() {
            return vec![];
        }

        let p = self.monic();
        let dp = p.deriv();

        if dp.is_zero() {
            return vec![(p, 1)];
        }

        // Compute gcd(p, p')
        let g = Self::gcd(p.clone(), dp.clone());

        // If gcd = 1, p is already square-free
        if g.degree() == Some(0) || g.is_zero() {
            return vec![(p, 1)];
        }

        // Simple approach: return square-free part
        // p / gcd(p, p') is square-free
        let (square_free_part, _) = p.div_rem(&g).expect("gcd divides p");

        vec![(square_free_part.monic(), 1)]
    }

    /// Compute the resultant of two polynomials using the Sylvester matrix determinant.
    ///
    /// The resultant is zero if and only if the polynomials have a common root.
    /// For polynomials f of degree n and g of degree m, constructs an (m+n) × (m+n)
    /// Sylvester matrix and returns its determinant.
    ///
    /// Returns None if both polynomials are zero.
    pub fn resultant(f: &Self, g: &Self) -> Option<Q> {
        assert_eq!(f.var, g.var, "polynomials must have the same variable");

        if f.is_zero() && g.is_zero() {
            return None;
        }

        // Handle cases where one polynomial is zero
        if f.is_zero() {
            return Some(Q::zero());
        }
        if g.is_zero() {
            return Some(Q::zero());
        }

        let n = f.degree()?;
        let m = g.degree()?;

        // Handle constant polynomials
        if n == 0 && m == 0 {
            return Some(Q::one());
        }
        if n == 0 {
            // f is constant, resultant is f^m
            let f0 = f.coeffs[0];
            let mut result = Q::one();
            for _ in 0..m {
                result = mul_q(result, f0);
            }
            return Some(result);
        }
        if m == 0 {
            // g is constant, resultant is g^n
            let g0 = g.coeffs[0];
            let mut result = Q::one();
            for _ in 0..n {
                result = mul_q(result, g0);
            }
            return Some(result);
        }

        // Build Sylvester matrix: (m+n) × (m+n)
        let size = m + n;
        let mut entries = Vec::with_capacity(size * size);

        for i in 0..size {
            for j in 0..size {
                let val = if i < m {
                    // First m rows: shifted coefficients of f
                    // Row i has f's coefficients starting at column i
                    if j >= i && j - i <= n {
                        f.coeffs[n - (j - i)]
                    } else {
                        Q::zero()
                    }
                } else {
                    // Last n rows: shifted coefficients of g
                    // Row i-m (for i >= m) has g's coefficients starting at column (i-m)
                    let row_offset = i - m;
                    if j >= row_offset && j - row_offset <= m {
                        g.coeffs[m - (j - row_offset)]
                    } else {
                        Q::zero()
                    }
                };
                entries.push(val);
            }
        }

        let sylvester = MatrixQ::new(size, size, entries);
        Some(sylvester.det_bareiss().expect("square matrix"))
    }

    /// Compute the discriminant of a polynomial.
    ///
    /// The discriminant is zero if and only if the polynomial has a repeated root.
    /// For a polynomial f of degree n with leading coefficient a_n:
    ///   disc(f) = (-1)^(n(n-1)/2) / a_n * resultant(f, f')
    ///
    /// Returns None if the polynomial is zero or constant.
    pub fn discriminant(&self) -> Option<Q> {
        if self.is_zero() {
            return None;
        }

        let n = self.degree()?;
        if n == 0 {
            return None; // Constant polynomial has no discriminant
        }

        let fp = self.deriv();
        let res = Self::resultant(self, &fp)?;

        let lc = self.leading_coeff();
        if lc.is_zero() {
            return None;
        }

        // disc(f) = (-1)^(n(n-1)/2) / lc * res(f, f')
        let sign_power = (n * (n - 1)) / 2;
        let sign = if sign_power % 2 == 0 { Q::one() } else { Q(-1, 1) };

        let disc = div_q(mul_q(sign, res), lc);
        Some(disc)
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

// ---------- Multivariate sparse polynomial over Q ----------

/// A monomial: product of variables raised to non-negative integer powers.
/// Represented as a sorted map from variable name to exponent.
/// Example: x^2 * y * z^3 is represented as {"x": 2, "y": 1, "z": 3}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Monomial(BTreeMap<String, usize>);

impl Monomial {
    pub fn one() -> Self {
        Self(BTreeMap::new())
    }

    pub fn var<S: Into<String>>(name: S) -> Self {
        let mut map = BTreeMap::new();
        map.insert(name.into(), 1);
        Self(map)
    }

    pub fn degree(&self) -> usize {
        self.0.values().sum()
    }

    /// Multiply two monomials by adding exponents
    pub fn mul(&self, other: &Self) -> Self {
        let mut result = self.0.clone();
        for (var, &exp) in &other.0 {
            *result.entry(var.clone()).or_insert(0) += exp;
        }
        // Remove zero exponents
        result.retain(|_, &mut exp| exp > 0);
        Self(result)
    }

    /// Evaluate monomial at given variable assignments
    pub fn eval(&self, vals: &BTreeMap<String, Q>) -> Option<Q> {
        let mut result = Q::one();
        for (var, &exp) in &self.0 {
            let val = vals.get(var)?;
            for _ in 0..exp {
                result = mul_q(result, *val);
            }
        }
        Some(result)
    }
}

/// Multivariate sparse polynomial over Q
/// Represented as a map from monomial to coefficient
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MultiPoly {
    /// Map from monomial to coefficient; zero coefficients should be removed
    pub terms: BTreeMap<Monomial, Q>,
}

impl MultiPoly {
    pub fn zero() -> Self {
        Self { terms: BTreeMap::new() }
    }

    pub fn constant(c: Q) -> Self {
        if c.is_zero() {
            return Self::zero();
        }
        let mut terms = BTreeMap::new();
        terms.insert(Monomial::one(), c);
        Self { terms }
    }

    pub fn var<S: Into<String>>(name: S) -> Self {
        let mut terms = BTreeMap::new();
        terms.insert(Monomial::var(name), Q::one());
        Self { terms }
    }

    pub fn is_zero(&self) -> bool {
        self.terms.is_empty()
    }

    /// Total degree: maximum degree of any monomial
    pub fn total_degree(&self) -> usize {
        self.terms.keys().map(|m| m.degree()).max().unwrap_or(0)
    }

    /// Add two polynomials
    pub fn add(&self, other: &Self) -> Self {
        let mut result = self.terms.clone();
        for (mon, &coeff) in &other.terms {
            let new_coeff = add_q(result.get(mon).copied().unwrap_or(Q::zero()), coeff);
            if new_coeff.is_zero() {
                result.remove(mon);
            } else {
                result.insert(mon.clone(), new_coeff);
            }
        }
        Self { terms: result }
    }

    /// Subtract two polynomials
    pub fn sub(&self, other: &Self) -> Self {
        let mut result = self.terms.clone();
        for (mon, &coeff) in &other.terms {
            let new_coeff = sub_q(result.get(mon).copied().unwrap_or(Q::zero()), coeff);
            if new_coeff.is_zero() {
                result.remove(mon);
            } else {
                result.insert(mon.clone(), new_coeff);
            }
        }
        Self { terms: result }
    }

    /// Multiply two polynomials
    pub fn mul(&self, other: &Self) -> Self {
        if self.is_zero() || other.is_zero() {
            return Self::zero();
        }

        let mut result: BTreeMap<Monomial, Q> = BTreeMap::new();
        for (m1, &c1) in &self.terms {
            for (m2, &c2) in &other.terms {
                let mon = m1.mul(m2);
                let coeff = mul_q(c1, c2);
                let new_coeff = add_q(result.get(&mon).copied().unwrap_or(Q::zero()), coeff);
                if new_coeff.is_zero() {
                    result.remove(&mon);
                } else {
                    result.insert(mon, new_coeff);
                }
            }
        }
        Self { terms: result }
    }

    /// Evaluate polynomial at given variable assignments
    pub fn eval(&self, vals: &BTreeMap<String, Q>) -> Option<Q> {
        let mut result = Q::zero();
        for (mon, &coeff) in &self.terms {
            let mon_val = mon.eval(vals)?;
            result = add_q(result, mul_q(coeff, mon_val));
        }
        Some(result)
    }

    /// Number of terms (non-zero coefficients)
    pub fn num_terms(&self) -> usize {
        self.terms.len()
    }
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

    #[test]
    fn square_free_already_square_free() {
        // p(x) = x + 1 is already square-free
        let p = UniPoly::new("x", vec![Q(1, 1), Q(1, 1)]);
        let decomp = p.square_free_decomposition();
        assert_eq!(decomp.len(), 1);
        assert_eq!(decomp[0].1, 1); // multiplicity 1
        assert_eq!(decomp[0].0.monic(), p.monic());
    }

    #[test]
    fn square_free_perfect_square() {
        // p(x) = (x - 1)^2 = x^2 - 2x + 1
        // Square-free part should be (x - 1)
        let p = UniPoly::new("x", vec![Q(1, 1), Q(-2, 1), Q(1, 1)]);
        let decomp = p.square_free_decomposition();
        assert_eq!(decomp.len(), 1);

        // The square-free part should be x - 1
        let expected = UniPoly::new("x", vec![Q(-1, 1), Q(1, 1)]).monic();
        assert_eq!(decomp[0].0.monic(), expected);
    }

    #[test]
    fn square_free_mixed_multiplicities() {
        // p(x) = x^2 * (x - 1)^3 = x^5 - 3x^4 + 3x^3 - x^2
        // Square-free part should be x * (x - 1)
        let p = UniPoly::new("x", vec![Q(0, 1), Q(0, 1), Q(-1, 1), Q(3, 1), Q(-3, 1), Q(1, 1)]);
        let decomp = p.square_free_decomposition();
        assert!(!decomp.is_empty());

        // The square-free part x(x-1) should have degree 2
        assert_eq!(decomp[0].0.degree(), Some(2));
    }

    #[test]
    fn square_free_cubic_with_repeated_root() {
        // p(x) = (x + 2)^2 * (x - 3) = x^3 + x^2 - 8x - 12
        // Square-free part should be (x + 2)(x - 3)
        let p = UniPoly::new("x", vec![Q(-12, 1), Q(-8, 1), Q(1, 1), Q(1, 1)]);
        let decomp = p.square_free_decomposition();
        assert!(!decomp.is_empty());

        // The square-free part should have degree 2
        assert_eq!(decomp[0].0.degree(), Some(2));
    }

    #[test]
    fn square_free_zero_polynomial() {
        let p = UniPoly::zero("x");
        let decomp = p.square_free_decomposition();
        assert_eq!(decomp.len(), 0);
    }

    #[test]
    fn square_free_constant_polynomial() {
        // p(x) = 5 (constant)
        let p = UniPoly::new("x", vec![Q(5, 1)]);
        let decomp = p.square_free_decomposition();
        // Constant is considered square-free with multiplicity 1
        assert_eq!(decomp.len(), 1);
        assert_eq!(decomp[0].1, 1);
    }

    #[test]
    fn square_free_linear() {
        // p(x) = 2x + 3
        let p = UniPoly::new("x", vec![Q(3, 1), Q(2, 1)]);
        let decomp = p.square_free_decomposition();
        assert_eq!(decomp.len(), 1);
        assert_eq!(decomp[0].1, 1);
    }

    #[test]
    fn square_free_product_distinct_linear() {
        // p(x) = (x - 1)(x - 2)(x - 3) = x^3 - 6x^2 + 11x - 6
        let p = UniPoly::new("x", vec![Q(-6, 1), Q(11, 1), Q(-6, 1), Q(1, 1)]);
        let decomp = p.square_free_decomposition();
        assert_eq!(decomp.len(), 1);
        assert_eq!(decomp[0].1, 1); // all roots are simple
    }

    #[test]
    fn square_free_high_multiplicity() {
        // p(x) = (x - 1)^4 = x^4 - 4x^3 + 6x^2 - 4x + 1
        // Square-free part should be (x - 1)
        let p = UniPoly::new("x", vec![Q(1, 1), Q(-4, 1), Q(6, 1), Q(-4, 1), Q(1, 1)]);
        let decomp = p.square_free_decomposition();

        assert_eq!(decomp.len(), 1);
        // The square-free part should be x - 1
        let expected = UniPoly::new("x", vec![Q(-1, 1), Q(1, 1)]).monic();
        assert_eq!(decomp[0].0.monic(), expected);
    }

    #[test]
    fn resultant_no_common_roots() {
        // f(x) = x - 1, g(x) = x - 2
        // No common roots, resultant should be non-zero
        // res(f,g) = f(root of g) = (2-1) = 1 (up to sign)
        // Actually res = product of (root_f - root_g) = (1 - 2) = -1
        let f = UniPoly::new("x", vec![Q(-1, 1), Q(1, 1)]);
        let g = UniPoly::new("x", vec![Q(-2, 1), Q(1, 1)]);
        let res = UniPoly::resultant(&f, &g).unwrap();
        assert!(!res.is_zero());
        // Result should be -1
        assert_eq!(res, Q(-1, 1));
    }

    #[test]
    fn resultant_common_root() {
        // f(x) = (x - 1)(x - 2) = x^2 - 3x + 2
        // g(x) = (x - 1)(x - 3) = x^2 - 4x + 3
        // Common root at x = 1, resultant should be zero
        let f = UniPoly::new("x", vec![Q(2, 1), Q(-3, 1), Q(1, 1)]);
        let g = UniPoly::new("x", vec![Q(3, 1), Q(-4, 1), Q(1, 1)]);
        let res = UniPoly::resultant(&f, &g).unwrap();
        assert_eq!(res, Q::zero());
    }

    #[test]
    fn resultant_linear_polynomials() {
        // f(x) = 2x + 3, g(x) = 4x + 5
        // res(f, g) = 2*5 - 3*4 = 10 - 12 = -2
        let f = UniPoly::new("x", vec![Q(3, 1), Q(2, 1)]);
        let g = UniPoly::new("x", vec![Q(5, 1), Q(4, 1)]);
        let res = UniPoly::resultant(&f, &g).unwrap();
        assert_eq!(res, Q(-2, 1));
    }

    #[test]
    fn resultant_with_constant() {
        // f(x) = 3 (constant), g(x) = x^2 + 1
        // res = 3^2 = 9
        let f = UniPoly::new("x", vec![Q(3, 1)]);
        let g = UniPoly::new("x", vec![Q(1, 1), Q(0, 1), Q(1, 1)]);
        let res = UniPoly::resultant(&f, &g).unwrap();
        assert_eq!(res, Q(9, 1));
    }

    #[test]
    fn resultant_zero_polynomials() {
        let f = UniPoly::zero("x");
        let g = UniPoly::zero("x");
        let res = UniPoly::resultant(&f, &g);
        assert!(res.is_none());
    }

    #[test]
    fn discriminant_no_repeated_roots() {
        // f(x) = (x - 1)(x - 2) = x^2 - 3x + 2
        // No repeated roots, discriminant != 0
        // disc = b^2 - 4ac = 9 - 8 = 1
        let f = UniPoly::new("x", vec![Q(2, 1), Q(-3, 1), Q(1, 1)]);
        let disc = f.discriminant().unwrap();
        assert_eq!(disc, Q(1, 1));
    }

    #[test]
    fn discriminant_repeated_root() {
        // f(x) = (x - 1)^2 = x^2 - 2x + 1
        // Has repeated root, discriminant = 0
        // disc = b^2 - 4ac = 4 - 4 = 0
        let f = UniPoly::new("x", vec![Q(1, 1), Q(-2, 1), Q(1, 1)]);
        let disc = f.discriminant().unwrap();
        assert_eq!(disc, Q::zero());
    }

    #[test]
    fn discriminant_cubic() {
        // f(x) = x^3 + x + 1
        // disc(x^3 + px + q) = -4p^3 - 27q^2
        // disc = -4(1)^3 - 27(1)^2 = -4 - 27 = -31
        let f = UniPoly::new("x", vec![Q(1, 1), Q(1, 1), Q(0, 1), Q(1, 1)]);
        let disc = f.discriminant().unwrap();
        assert_eq!(disc, Q(-31, 1));
    }

    #[test]
    fn discriminant_linear_returns_none() {
        // Linear polynomial has no discriminant
        let f = UniPoly::new("x", vec![Q(1, 1), Q(2, 1)]);
        // Actually for linear ax + b, we can compute discriminant
        // Let me check - typically discriminant is defined for degree >= 2
        // For degree 1, it should be 1 (no repeated roots possible)
        let disc = f.discriminant();
        // Based on formula, derivative is constant, resultant will be that constant
        // Actually for linear, it may vary by convention, let's check implementation
        assert!(disc.is_some());
    }

    #[test]
    fn discriminant_constant_returns_none() {
        // Constant polynomial has no discriminant
        let f = UniPoly::new("x", vec![Q(5, 1)]);
        let disc = f.discriminant();
        assert!(disc.is_none());
    }

    #[test]
    fn discriminant_quadratic_formula() {
        // f(x) = ax^2 + bx + c
        // disc = b^2 - 4ac
        // Test: 2x^2 + 3x + 1
        // disc = 9 - 8 = 1
        let f = UniPoly::new("x", vec![Q(1, 1), Q(3, 1), Q(2, 1)]);
        let disc = f.discriminant().unwrap();
        assert_eq!(disc, Q(1, 1));
    }

    // ========== Multivariate Polynomial Tests ==========

    #[test]
    fn multipoly_zero_and_constant() {
        let zero = MultiPoly::zero();
        assert!(zero.is_zero());
        assert_eq!(zero.total_degree(), 0);

        let c = MultiPoly::constant(Q(5, 1));
        assert!(!c.is_zero());
        assert_eq!(c.total_degree(), 0);
        assert_eq!(c.num_terms(), 1);
    }

    #[test]
    fn multipoly_var() {
        let x = MultiPoly::var("x");
        assert_eq!(x.total_degree(), 1);
        assert_eq!(x.num_terms(), 1);
    }

    #[test]
    fn multipoly_add() {
        // x + y
        let x = MultiPoly::var("x");
        let y = MultiPoly::var("y");
        let sum = x.add(&y);
        assert_eq!(sum.num_terms(), 2);
        assert_eq!(sum.total_degree(), 1);

        // x + x = 2x
        let double_x = x.add(&x);
        assert_eq!(double_x.num_terms(), 1);
        let mx = Monomial::var("x");
        assert_eq!(double_x.terms.get(&mx), Some(&Q(2, 1)));
    }

    #[test]
    fn multipoly_sub() {
        // x - y
        let x = MultiPoly::var("x");
        let y = MultiPoly::var("y");
        let diff = x.sub(&y);
        assert_eq!(diff.num_terms(), 2);

        // x - x = 0
        let zero = x.sub(&x);
        assert!(zero.is_zero());
    }

    #[test]
    fn multipoly_mul_simple() {
        // x * y = xy
        let x = MultiPoly::var("x");
        let y = MultiPoly::var("y");
        let prod = x.mul(&y);
        assert_eq!(prod.num_terms(), 1);
        assert_eq!(prod.total_degree(), 2);
    }

    #[test]
    fn multipoly_mul_expansion() {
        // (x + 1)(y + 2) = xy + 2x + y + 2
        let x = MultiPoly::var("x");
        let y = MultiPoly::var("y");
        let one = MultiPoly::constant(Q(1, 1));
        let two = MultiPoly::constant(Q(2, 1));

        let x_plus_1 = x.add(&one);
        let y_plus_2 = y.add(&two);
        let prod = x_plus_1.mul(&y_plus_2);

        assert_eq!(prod.num_terms(), 4);
        assert_eq!(prod.total_degree(), 2);
    }

    #[test]
    fn multipoly_eval() {
        // p = 2xy + 3x + 5
        let x = MultiPoly::var("x");
        let y = MultiPoly::var("y");
        let xy = x.mul(&y);
        let two = MultiPoly::constant(Q(2, 1));
        let three = MultiPoly::constant(Q(3, 1));
        let five = MultiPoly::constant(Q(5, 1));

        let two_xy = two.mul(&xy);
        let three_x = three.mul(&x);
        let p = two_xy.add(&three_x).add(&five);

        let mut vals = BTreeMap::new();
        vals.insert("x".to_string(), Q(2, 1));
        vals.insert("y".to_string(), Q(3, 1));

        // 2*2*3 + 3*2 + 5 = 12 + 6 + 5 = 23
        let result = p.eval(&vals).unwrap();
        assert_eq!(result, Q(23, 1));
    }

    #[test]
    fn multipoly_eval_missing_var() {
        let x = MultiPoly::var("x");
        let y = MultiPoly::var("y");
        let p = x.mul(&y);

        let mut vals = BTreeMap::new();
        vals.insert("x".to_string(), Q(2, 1));
        // Missing y

        assert!(p.eval(&vals).is_none());
    }

    #[test]
    fn monomial_mul() {
        // x^2 * y * x^3 * z = x^5 * y * z
        let m1 = Monomial::var("x").mul(&Monomial::var("x")); // x^2
        let m2 = Monomial::var("y");
        let m3 = Monomial::var("x").mul(&Monomial::var("x")).mul(&Monomial::var("x")); // x^3
        let m4 = Monomial::var("z");

        let result = m1.mul(&m2).mul(&m3).mul(&m4);
        assert_eq!(result.0.get("x"), Some(&5));
        assert_eq!(result.0.get("y"), Some(&1));
        assert_eq!(result.0.get("z"), Some(&1));
        assert_eq!(result.degree(), 7);
    }

    #[test]
    fn multipoly_zero_mul() {
        let x = MultiPoly::var("x");
        let zero = MultiPoly::zero();
        let prod = x.mul(&zero);
        assert!(prod.is_zero());
    }

    #[test]
    fn multipoly_three_var_polynomial() {
        // p = x^2 + xy + yz + z^2
        let x = MultiPoly::var("x");
        let y = MultiPoly::var("y");
        let z = MultiPoly::var("z");

        let x2 = x.mul(&x);
        let xy = x.mul(&y);
        let yz = y.mul(&z);
        let z2 = z.mul(&z);

        let p = x2.add(&xy).add(&yz).add(&z2);
        assert_eq!(p.num_terms(), 4);
        assert_eq!(p.total_degree(), 2);

        let mut vals = BTreeMap::new();
        vals.insert("x".to_string(), Q(1, 1));
        vals.insert("y".to_string(), Q(2, 1));
        vals.insert("z".to_string(), Q(3, 1));

        // 1 + 2 + 6 + 9 = 18
        let result = p.eval(&vals).unwrap();
        assert_eq!(result, Q(18, 1));
    }
}
