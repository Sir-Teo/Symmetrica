//! Gröbner Bases Module
//! Phase 4: Advanced Solving via Gröbner Bases (v1.3)
//!
//! This module provides Gröbner basis computation for solving
//! systems of polynomial equations. Key algorithms:
//! - Buchberger's algorithm for basis construction
//! - S-polynomial computation
//! - Polynomial reduction
//! - Monomial orderings (lex, grlex, grevlex)
//!
//! Status: Foundation implementation

#![deny(warnings)]

use expr_core::{ExprId, Op, Payload, Store};
use std::collections::HashMap;

/// Monomial ordering types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonomialOrder {
    /// Lexicographic order
    Lex,
    /// Graded lexicographic order
    GrLex,
    /// Graded reverse lexicographic order
    GRevLex,
}

/// Represents a monomial as a map from variable names to exponents
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Monomial {
    /// Variable name -> exponent
    pub exponents: HashMap<String, i64>,
}

impl Monomial {
    /// Create a monomial from an expression
    pub fn from_expr(store: &Store, expr: ExprId) -> Option<Self> {
        let mut exponents = HashMap::new();
        extract_monomial(store, expr, &mut exponents)?;
        Some(Monomial { exponents })
    }

    /// Total degree of the monomial
    pub fn degree(&self) -> i64 {
        self.exponents.values().sum()
    }

    /// Compare two monomials using given ordering
    pub fn compare(
        &self,
        other: &Self,
        order: MonomialOrder,
        vars: &[String],
    ) -> std::cmp::Ordering {
        use std::cmp::Ordering;

        match order {
            MonomialOrder::Lex => {
                // Compare lexicographically
                for var in vars {
                    let exp1 = self.exponents.get(var).unwrap_or(&0);
                    let exp2 = other.exponents.get(var).unwrap_or(&0);
                    match exp1.cmp(exp2) {
                        Ordering::Equal => continue,
                        ord => return ord,
                    }
                }
                Ordering::Equal
            }
            MonomialOrder::GrLex => {
                // Compare by total degree first, then lexicographically
                match self.degree().cmp(&other.degree()) {
                    Ordering::Equal => self.compare(other, MonomialOrder::Lex, vars),
                    ord => ord,
                }
            }
            MonomialOrder::GRevLex => {
                // Compare by total degree first, then reverse lexicographically
                match self.degree().cmp(&other.degree()) {
                    Ordering::Equal => {
                        for var in vars.iter().rev() {
                            let exp1 = self.exponents.get(var).unwrap_or(&0);
                            let exp2 = other.exponents.get(var).unwrap_or(&0);
                            match exp2.cmp(exp1) {
                                Ordering::Equal => continue,
                                ord => return ord,
                            }
                        }
                        Ordering::Equal
                    }
                    ord => ord,
                }
            }
        }
    }
}

/// Extract monomial structure from expression
fn extract_monomial(
    store: &Store,
    expr: ExprId,
    exponents: &mut HashMap<String, i64>,
) -> Option<()> {
    match store.get(expr).op {
        Op::Integer | Op::Rational => Some(()), // Constant term
        Op::Symbol => {
            if let Payload::Sym(var) = &store.get(expr).payload {
                *exponents.entry(var.clone()).or_insert(0) += 1;
            }
            Some(())
        }
        Op::Pow => {
            let children = &store.get(expr).children;
            if children.len() != 2 {
                return None;
            }
            let base = children[0];
            let exp = children[1];

            // Get exponent as integer
            if let (Op::Integer, Payload::Int(e)) = (&store.get(exp).op, &store.get(exp).payload) {
                if let Payload::Sym(var) = &store.get(base).payload {
                    *exponents.entry(var.clone()).or_insert(0) += e;
                    return Some(());
                }
            }
            None
        }
        Op::Mul => {
            let children = &store.get(expr).children;
            for &child in children {
                extract_monomial(store, child, exponents)?;
            }
            Some(())
        }
        _ => None,
    }
}

/// Compute the S-polynomial of two polynomials
/// S(f, g) = (lcm(LT(f), LT(g)) / LT(f)) * f - (lcm(LT(f), LT(g)) / LT(g)) * g
pub fn s_polynomial(
    store: &mut Store,
    f: ExprId,
    g: ExprId,
    vars: &[String],
    order: MonomialOrder,
) -> Option<ExprId> {
    // Extract terms (flatten nested adds)
    fn collect_terms(store: &Store, p: ExprId, out: &mut Vec<ExprId>) {
        match store.get(p).op {
            Op::Add => {
                for &ch in &store.get(p).children {
                    collect_terms(store, ch, out);
                }
            }
            _ => out.push(p),
        }
    }
    fn terms_of(store: &Store, p: ExprId) -> Vec<ExprId> {
        let mut v = Vec::new();
        collect_terms(store, p, &mut v);
        v
    }

    fn leading_term(
        store: &Store,
        p: ExprId,
        vars: &[String],
        order: MonomialOrder,
    ) -> Option<ExprId> {
        let mut best: Option<(Monomial, ExprId)> = None;
        for t in terms_of(store, p) {
            if let Some(m) = Monomial::from_expr(store, t) {
                match &best {
                    None => best = Some((m, t)),
                    Some((bm, _)) => {
                        if m.compare(bm, order, vars) == std::cmp::Ordering::Greater {
                            best = Some((m, t));
                        }
                    }
                }
            }
        }
        best.map(|(_, t)| t)
    }

    // Build monomial expression from exponent map
    fn monomial_expr(store: &mut Store, exps: &HashMap<String, i64>) -> ExprId {
        let mut factors: Vec<ExprId> = Vec::new();
        for (v, e) in exps.iter() {
            if *e == 0 {
                continue;
            }
            let sym = store.sym(v);
            if *e == 1 {
                factors.push(sym);
            } else {
                let ei = store.int(*e);
                let p = store.pow(sym, ei);
                factors.push(p);
            }
        }
        if factors.is_empty() {
            return store.int(1);
        }
        if factors.len() == 1 {
            return factors[0];
        }
        store.mul(factors)
    }

    fn lcm_exponents(a: &Monomial, b: &Monomial) -> HashMap<String, i64> {
        let mut out = a.exponents.clone();
        for (k, vb) in &b.exponents {
            let va = *out.get(k).unwrap_or(&0);
            if vb > &va {
                out.insert(k.clone(), *vb);
            }
        }
        out
    }

    fn exponent_diff(a: &HashMap<String, i64>, b: &Monomial) -> HashMap<String, i64> {
        let mut out = HashMap::new();
        for (k, va) in a {
            let vb = *b.exponents.get(k).unwrap_or(&0);
            let d = *va - vb;
            if d != 0 {
                out.insert(k.clone(), d);
            }
        }
        out
    }

    let lt_f = leading_term(store, f, vars, order)?;
    let lt_g = leading_term(store, g, vars, order)?;
    let mf = Monomial::from_expr(store, lt_f)?;
    let mg = Monomial::from_expr(store, lt_g)?;

    let lcm_exp = lcm_exponents(&mf, &mg);
    let mult_f_exp = exponent_diff(&lcm_exp, &mf);
    let mult_g_exp = exponent_diff(&lcm_exp, &mg);

    let mult_f = monomial_expr(store, &mult_f_exp);
    let mult_g = monomial_expr(store, &mult_g_exp);

    let mf_f = store.mul(vec![mult_f, f]);
    let mg_g = store.mul(vec![mult_g, g]);
    let neg_one = store.int(-1);
    let minus_mg_g = store.mul(vec![neg_one, mg_g]);
    Some(store.add(vec![mf_f, minus_mg_g]))
}

/// Reduce polynomial f with respect to set of polynomials G
/// Returns the remainder after division
pub fn reduce(
    store: &mut Store,
    f: ExprId,
    basis: &[ExprId],
    vars: &[String],
    order: MonomialOrder,
) -> ExprId {
    // Helpers (duplicated from s_polynomial for now)
    fn collect_terms(store: &Store, p: ExprId, out: &mut Vec<ExprId>) {
        match store.get(p).op {
            Op::Add => {
                for &ch in &store.get(p).children {
                    collect_terms(store, ch, out);
                }
            }
            _ => out.push(p),
        }
    }
    fn terms_of(store: &Store, p: ExprId) -> Vec<ExprId> {
        let mut v = Vec::new();
        collect_terms(store, p, &mut v);
        v
    }

    fn leading_term(
        store: &Store,
        p: ExprId,
        vars: &[String],
        order: MonomialOrder,
    ) -> Option<ExprId> {
        let mut best: Option<(Monomial, ExprId)> = None;
        for t in terms_of(store, p) {
            if let Some(m) = Monomial::from_expr(store, t) {
                match &best {
                    None => best = Some((m, t)),
                    Some((bm, _)) => {
                        if m.compare(bm, order, vars) == std::cmp::Ordering::Greater {
                            best = Some((m, t));
                        }
                    }
                }
            }
        }
        best.map(|(_, t)| t)
    }

    fn monomial_expr(store: &mut Store, exps: &HashMap<String, i64>) -> ExprId {
        let mut factors: Vec<ExprId> = Vec::new();
        for (v, e) in exps.iter() {
            if *e == 0 {
                continue;
            }
            let sym = store.sym(v);
            if *e == 1 {
                factors.push(sym);
            } else {
                let ei = store.int(*e);
                factors.push(store.pow(sym, ei));
            }
        }
        if factors.is_empty() {
            return store.int(1);
        }
        if factors.len() == 1 {
            return factors[0];
        }
        store.mul(factors)
    }

    fn exp_ge(a: &HashMap<String, i64>, b: &HashMap<String, i64>) -> bool {
        for (k, vb) in b {
            let va = *a.get(k).unwrap_or(&0);
            if va < *vb {
                return false;
            }
        }
        true
    }

    fn exp_sub(a: &HashMap<String, i64>, b: &HashMap<String, i64>) -> HashMap<String, i64> {
        let mut out = HashMap::new();
        for (k, va) in a {
            let vb = *b.get(k).unwrap_or(&0);
            let d = *va - vb;
            if d != 0 {
                out.insert(k.clone(), d);
            }
        }
        out
    }

    fn rebuild_without_term(store: &mut Store, p: ExprId, t: ExprId) -> ExprId {
        let mut terms = terms_of(store, p);
        if let Some(pos) = terms.iter().position(|&e| e == t) {
            terms.remove(pos);
        }
        match terms.len() {
            0 => store.int(0),
            1 => terms[0],
            _ => store.add(terms),
        }
    }

    let mut p = f;
    let mut changed = true;
    let max_steps = 256;
    let mut steps = 0;
    while changed && steps < max_steps {
        steps += 1;
        changed = false;
        // Pick a term from p
        let lt_p = if let Some(t) = leading_term(store, p, vars, order) {
            t
        } else {
            break;
        };
        let mp = if let Some(m) = Monomial::from_expr(store, lt_p) {
            m
        } else {
            break;
        };

        // Try to reduce with basis
        'outer: for &g in basis {
            if let Some(lt_g) = leading_term(store, g, vars, order) {
                if let (Some(mg),) = (Monomial::from_expr(store, lt_g),) {
                    if exp_ge(&mp.exponents, &mg.exponents) {
                        // If g is a monomial polynomial (single term), then q*g == lt_p:
                        // remove lt_p directly to avoid coefficient arithmetic.
                        let g_is_monomial_poly = !matches!(store.get(g).op, Op::Add)
                            && Monomial::from_expr(store, g).is_some();
                        if g_is_monomial_poly {
                            p = rebuild_without_term(store, p, lt_p);
                        } else {
                            let q_exp = exp_sub(&mp.exponents, &mg.exponents);
                            let q = monomial_expr(store, &q_exp);
                            let qg = store.mul(vec![q, g]);
                            let neg_one = store.int(-1);
                            let sub = store.mul(vec![neg_one, qg]);
                            p = store.add(vec![p, sub]);
                        }
                        changed = true;
                        break 'outer;
                    }
                }
            }
        }
    }
    p
}

/// Buchberger's algorithm for computing Gröbner basis
/// Returns a Gröbner basis for the ideal generated by the input polynomials
pub fn buchberger(
    store: &mut Store,
    polys: Vec<ExprId>,
    vars: Vec<String>,
    order: MonomialOrder,
) -> Vec<ExprId> {
    if polys.is_empty() {
        return vec![];
    }

    let mut basis = polys.clone();
    let mut pairs: Vec<(usize, usize)> = Vec::new();

    // Generate all pairs
    for i in 0..basis.len() {
        for j in (i + 1)..basis.len() {
            pairs.push((i, j));
        }
    }

    // Simplified Buchberger: process pairs and add non-zero remainders
    let mut iteration = 0;
    let max_iterations = 100; // Prevent infinite loops

    while !pairs.is_empty() && iteration < max_iterations {
        iteration += 1;
        let (i, j) = pairs.pop().unwrap();

        if i >= basis.len() || j >= basis.len() {
            continue;
        }

        // Compute S-polynomial
        if let Some(s) = s_polynomial(store, basis[i], basis[j], &vars, order) {
            // Reduce S-polynomial with respect to current basis
            let remainder = reduce(store, s, &basis, &vars, order);

            // Check if remainder is non-zero (simplified: check if not zero constant)
            let is_zero = matches!(
                (&store.get(remainder).op, &store.get(remainder).payload),
                (Op::Integer, Payload::Int(0))
            );

            if !is_zero {
                // Add remainder to basis and generate new pairs
                let new_idx = basis.len();
                basis.push(remainder);

                for k in 0..new_idx {
                    pairs.push((k, new_idx));
                }
            }
        }
    }

    basis
}

/// Solve a system of polynomial equations using Gröbner bases
/// Returns solution(s) if they exist
pub fn solve_system(
    _store: &mut Store,
    _equations: Vec<ExprId>,
    _vars: Vec<String>,
) -> Option<Vec<HashMap<String, ExprId>>> {
    // TODO: Implement using Gröbner basis and back-substitution
    // 1. Compute Gröbner basis with lex ordering
    // 2. Check if basis is in triangular form
    // 3. Back-substitute to find solutions
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monomial_from_constant() {
        let mut st = Store::new();
        let five = st.int(5);
        let mono = Monomial::from_expr(&st, five).unwrap();
        assert_eq!(mono.degree(), 0);
    }

    #[test]
    fn test_monomial_from_variable() {
        let mut st = Store::new();
        let x = st.sym("x");
        let mono = Monomial::from_expr(&st, x).unwrap();
        assert_eq!(mono.degree(), 1);
        assert_eq!(mono.exponents.get("x"), Some(&1));
    }

    #[test]
    fn test_monomial_from_power() {
        let mut st = Store::new();
        let x = st.sym("x");
        let three = st.int(3);
        let x_cubed = st.pow(x, three);
        let mono = Monomial::from_expr(&st, x_cubed).unwrap();
        assert_eq!(mono.degree(), 3);
        assert_eq!(mono.exponents.get("x"), Some(&3));
    }

    #[test]
    fn test_monomial_from_product() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let xy = st.mul(vec![x, y]);
        let mono = Monomial::from_expr(&st, xy).unwrap();
        assert_eq!(mono.degree(), 2);
        assert_eq!(mono.exponents.get("x"), Some(&1));
        assert_eq!(mono.exponents.get("y"), Some(&1));
    }

    #[test]
    fn test_monomial_compare_lex() {
        let mut mono1 = Monomial { exponents: HashMap::new() };
        mono1.exponents.insert("x".to_string(), 2);
        mono1.exponents.insert("y".to_string(), 1);

        let mut mono2 = Monomial { exponents: HashMap::new() };
        mono2.exponents.insert("x".to_string(), 1);
        mono2.exponents.insert("y".to_string(), 3);

        let vars = vec!["x".to_string(), "y".to_string()];
        assert_eq!(mono1.compare(&mono2, MonomialOrder::Lex, &vars), std::cmp::Ordering::Greater);
    }

    #[test]
    fn test_monomial_compare_grlex() {
        let mut mono1 = Monomial { exponents: HashMap::new() };
        mono1.exponents.insert("x".to_string(), 2);
        mono1.exponents.insert("y".to_string(), 1);

        let mut mono2 = Monomial { exponents: HashMap::new() };
        mono2.exponents.insert("x".to_string(), 1);
        mono2.exponents.insert("y".to_string(), 3);

        let vars = vec!["x".to_string(), "y".to_string()];
        // mono1 has degree 3, mono2 has degree 4, so mono2 > mono1
        assert_eq!(mono1.compare(&mono2, MonomialOrder::GrLex, &vars), std::cmp::Ordering::Less);
    }

    #[test]
    fn test_buchberger_empty() {
        let mut st = Store::new();
        let basis = buchberger(&mut st, vec![], vec![], MonomialOrder::Lex);
        assert_eq!(basis.len(), 0);
    }

    #[test]
    fn test_buchberger_single_poly() {
        let mut st = Store::new();
        let x = st.sym("x");
        let basis = buchberger(&mut st, vec![x], vec!["x".to_string()], MonomialOrder::Lex);
        assert_eq!(basis.len(), 1);
    }

    #[test]
    fn test_reduce_identity() {
        let mut st = Store::new();
        let x = st.sym("x");
        let reduced = reduce(&mut st, x, &[], &["x".to_string()], MonomialOrder::Lex);
        assert_eq!(reduced, x);
    }

    #[test]
    fn test_reduce_simple() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let xy = st.mul(vec![x, y]);
        let one = st.int(1);
        let f = st.add(vec![x2, xy, one]);
        let basis = vec![x, y];
        let r = reduce(&mut st, f, &basis, &["x".to_string(), "y".to_string()], MonomialOrder::Lex);
        // Expect remainder 1
        assert!(matches!((&st.get(r).op, &st.get(r).payload), (Op::Integer, Payload::Int(1))));
    }

    #[test]
    fn test_monomial_compare_grevlex() {
        let mut mono1 = Monomial { exponents: HashMap::new() };
        mono1.exponents.insert("x".to_string(), 3);
        mono1.exponents.insert("y".to_string(), 1);

        let mut mono2 = Monomial { exponents: HashMap::new() };
        mono2.exponents.insert("x".to_string(), 2);
        mono2.exponents.insert("y".to_string(), 2);

        let vars = vec!["x".to_string(), "y".to_string()];
        // Both have same degree 4, so use reverse lex
        let ord = mono1.compare(&mono2, MonomialOrder::GRevLex, &vars);
        assert!(ord != std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_s_polynomial_basic() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");

        // f = x^2, g = xy
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let xy = st.mul(vec![x, y]);

        let vars = vec!["x".to_string(), "y".to_string()];
        let s = s_polynomial(&mut st, x2, xy, &vars, MonomialOrder::Lex);
        assert!(s.is_some());
    }

    #[test]
    fn test_buchberger_two_polys() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let one = st.int(1);
        let neg_one = st.int(-1);

        // f = x - 1, g = y - 1
        let f = st.add(vec![x, neg_one]);
        let neg_one_2 = st.int(-1);
        let minus_one = st.mul(vec![neg_one_2, one]);
        let g = st.add(vec![y, minus_one]);

        let vars = vec!["x".to_string(), "y".to_string()];
        let basis = buchberger(&mut st, vec![f, g], vars, MonomialOrder::Lex);
        assert!(basis.len() >= 2);
    }

    #[test]
    fn test_monomial_from_mul_with_powers() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let two = st.int(2);
        let three = st.int(3);
        let x2 = st.pow(x, two);
        let y3 = st.pow(y, three);
        let prod = st.mul(vec![x2, y3]);

        let mono = Monomial::from_expr(&st, prod).unwrap();
        assert_eq!(mono.degree(), 5);
        assert_eq!(mono.exponents.get("x"), Some(&2));
        assert_eq!(mono.exponents.get("y"), Some(&3));
    }

    #[test]
    fn test_reduce_with_monomial_basis() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two);

        // Reduce x^2 with basis [x^2]
        let basis = vec![x2];
        let r = reduce(&mut st, x2, &basis, &["x".to_string()], MonomialOrder::Lex);

        // Should reduce to 0
        assert!(matches!((&st.get(r).op, &st.get(r).payload), (Op::Integer, Payload::Int(0))));
    }

    #[test]
    fn test_monomial_equal_comparison() {
        let mut mono1 = Monomial { exponents: HashMap::new() };
        mono1.exponents.insert("x".to_string(), 2);

        let mono2 = mono1.clone();

        let vars = vec!["x".to_string()];
        assert_eq!(mono1.compare(&mono2, MonomialOrder::Lex, &vars), std::cmp::Ordering::Equal);
        assert_eq!(mono1.compare(&mono2, MonomialOrder::GrLex, &vars), std::cmp::Ordering::Equal);
        assert_eq!(mono1.compare(&mono2, MonomialOrder::GRevLex, &vars), std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_extract_monomial_with_constant_mul() {
        let mut st = Store::new();
        let x = st.sym("x");
        let five = st.int(5);
        let five_x = st.mul(vec![five, x]);

        let mono = Monomial::from_expr(&st, five_x).unwrap();
        assert_eq!(mono.degree(), 1);
        assert_eq!(mono.exponents.get("x"), Some(&1));
    }

    #[test]
    fn test_solve_system_placeholder() {
        let mut st = Store::new();
        let x = st.sym("x");
        let result = solve_system(&mut st, vec![x], vec!["x".to_string()]);
        // Currently returns None as it's a placeholder
        assert!(result.is_none());
    }
}
