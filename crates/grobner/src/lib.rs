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
    _store: &mut Store,
    _f: ExprId,
    _g: ExprId,
    _vars: &[String],
    _order: MonomialOrder,
) -> Option<ExprId> {
    // Simplified stub: full implementation requires leading term extraction
    // and LCM computation of monomials
    None
}

/// Reduce polynomial f with respect to set of polynomials G
/// Returns the remainder after division
pub fn reduce(
    _store: &mut Store,
    _f: ExprId,
    _basis: &[ExprId],
    _vars: &[String],
    _order: MonomialOrder,
) -> ExprId {
    // Simplified stub: full implementation requires multivariate division algorithm
    _f
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
}
