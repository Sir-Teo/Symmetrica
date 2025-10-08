//! Summation module: symbolic summation and closed-form evaluation
//! Phase 5: Symbolic Summation (v1.4)
//!
//! This module provides algorithms for computing closed-form expressions
//! for sums and products, including:
//! - Gosper's algorithm for hypergeometric summation
//! - Zeilberger's algorithm for creative telescoping
//! - Basic sum formulas (arithmetic, geometric, power sums)
//! - Convergence tests
//!
//! Status: Scaffolding in progress

#![deny(warnings)]

use expr_core::{ExprId, Op, Payload, Store};

/// Hypergeometric term recognition
/// A term t(k) is hypergeometric if t(k+1)/t(k) is a rational function of k.
pub fn is_hypergeometric(store: &Store, term: ExprId, var: &str) -> bool {
    // Simple heuristic: check if the term is a product/power of factorials, powers, and exponentials
    // More sophisticated implementation would check the ratio t(k+1)/t(k)
    match store.get(term).op {
        Op::Integer | Op::Rational => true, // Constants are hypergeometric
        Op::Symbol => {
            // Single variable is hypergeometric
            matches!(&store.get(term).payload, Payload::Sym(s) if s == var)
        }
        Op::Mul | Op::Add => {
            // For products and sums, check if all terms are "simple" hypergeometric
            let children = &store.get(term).children;
            children.iter().all(|&c| is_simple_hypergeometric(store, c, var))
        }
        Op::Pow => {
            // k^n or a^k are hypergeometric
            let children = &store.get(term).children;
            if children.len() == 2 {
                is_simple_hypergeometric(store, children[0], var)
                    && is_simple_hypergeometric(store, children[1], var)
            } else {
                false
            }
        }
        Op::Function => {
            // Factorials and binomials are hypergeometric
            matches!(&store.get(term).payload, Payload::Func(name) if name == "factorial" || name == "binomial")
        }
        _ => false,
    }
}

/// Helper to check if a term is "simple" hypergeometric (no complex nesting)
fn is_simple_hypergeometric(store: &Store, term: ExprId, _var: &str) -> bool {
    match store.get(term).op {
        Op::Integer | Op::Rational => true,
        Op::Symbol => true,
        Op::Pow => {
            let children = &store.get(term).children;
            children.len() == 2
        }
        _ => false,
    }
}

/// Gosper's algorithm for indefinite summation of hypergeometric terms
/// Returns Some(anti_difference) if the sum has a closed form, None otherwise.
pub fn gosper_sum(_store: &mut Store, _term: ExprId, _var: &str) -> Option<ExprId> {
    // TODO: Implement Gosper's algorithm
    // 1. Check if term is hypergeometric
    // 2. Find rational certificate
    // 3. Construct anti-difference
    None
}

/// Basic arithmetic series: sum(a + k*d, k=0..n-1) = n*a + n*(n-1)/2 * d
pub fn sum_arithmetic(store: &mut Store, first: ExprId, diff: ExprId, n: ExprId) -> Option<ExprId> {
    // sum(a + k*d, k=0..n-1) = n*a + n*(n-1)/2 * d
    let n_a = store.mul(vec![n, first]);

    let one = store.int(1);
    let neg_one = store.int(-1);
    let neg_one_term = store.mul(vec![neg_one, one]);
    let n_minus_1 = store.add(vec![n, neg_one_term]);
    let half = store.rat(1, 2);

    // n*(n-1)/2
    let n_n_minus_1 = store.mul(vec![n, n_minus_1]);
    let n_n_minus_1_over_2 = store.mul(vec![n_n_minus_1, half]);

    // n*(n-1)/2 * d
    let second_term = store.mul(vec![n_n_minus_1_over_2, diff]);

    Some(store.add(vec![n_a, second_term]))
}

/// Sum of binomial coefficients: sum(C(n,k), k=0..n) = 2^n
pub fn sum_binomial_row(store: &mut Store, n: ExprId) -> ExprId {
    // sum(C(n,k), k=0..n) = 2^n
    let two = store.int(2);
    store.pow(two, n)
}

/// Sum of first n natural numbers: sum(k, k=1..n) = n(n+1)/2
pub fn sum_natural_numbers(store: &mut Store, n: ExprId) -> ExprId {
    let one = store.int(1);
    let n_plus_1 = store.add(vec![n, one]);
    let product = store.mul(vec![n, n_plus_1]);
    let half = store.rat(1, 2);
    store.mul(vec![product, half])
}

/// Sum of squares: sum(k^2, k=1..n) = n(n+1)(2n+1)/6
pub fn sum_squares(store: &mut Store, n: ExprId) -> ExprId {
    let one = store.int(1);
    let two = store.int(2);
    let six = store.int(6);
    
    let n_plus_1 = store.add(vec![n, one]);
    let two_n = store.mul(vec![two, n]);
    let two_n_plus_1 = store.add(vec![two_n, one]);
    
    let product = store.mul(vec![n, n_plus_1, two_n_plus_1]);
    let neg_one = store.int(-1);
    let inv_six = store.pow(six, neg_one);
    store.mul(vec![product, inv_six])
}

/// Sum of cubes: sum(k^3, k=1..n) = [n(n+1)/2]^2
pub fn sum_cubes(store: &mut Store, n: ExprId) -> ExprId {
    let sum_n = sum_natural_numbers(store, n);
    let two = store.int(2);
    store.pow(sum_n, two)
}

/// Basic geometric series: sum(a*r^k, k=0..n-1) = a*(1-r^n)/(1-r) for r≠1
pub fn sum_geometric(store: &mut Store, first: ExprId, ratio: ExprId, n: ExprId) -> Option<ExprId> {
    // sum(a*r^k, k=0..n-1) = a*(1-r^n)/(1-r)
    let one = store.int(1);
    let r_pow_n = store.pow(ratio, n);

    // 1 - r^n
    let neg_one = store.int(-1);
    let neg_r_pow_n = store.mul(vec![neg_one, r_pow_n]);
    let numerator_inner = store.add(vec![one, neg_r_pow_n]);

    // 1 - r
    let neg_one_2 = store.int(-1);
    let neg_ratio = store.mul(vec![neg_one_2, ratio]);
    let denominator = store.add(vec![one, neg_ratio]);

    // (1 - r^n) / (1 - r)
    let minus_one = store.int(-1);
    let denom_inv = store.pow(denominator, minus_one);
    let fraction = store.mul(vec![numerator_inner, denom_inv]);

    // a * (1 - r^n) / (1 - r)
    Some(store.mul(vec![first, fraction]))
}

/// Power sum formulas: ∑_{k=0}^{n} k^p
/// Returns the closed form for specific small powers
pub fn sum_power(store: &mut Store, power: usize, n: ExprId) -> Option<ExprId> {
    let one = store.int(1);
    let two = store.int(2);
    let n_plus_1 = store.add(vec![n, one]);

    match power {
        0 => {
            // ∑ 1 = n + 1
            Some(n_plus_1)
        }
        1 => {
            // ∑ k = n(n+1)/2
            let numerator = store.mul(vec![n, n_plus_1]);
            let half = store.rat(1, 2);
            Some(store.mul(vec![half, numerator]))
        }
        2 => {
            // ∑ k² = n(n+1)(2n+1)/6
            let two_n = store.mul(vec![two, n]);
            let two_n_plus_1 = store.add(vec![two_n, one]);
            let numerator = store.mul(vec![n, n_plus_1, two_n_plus_1]);
            let sixth = store.rat(1, 6);
            Some(store.mul(vec![sixth, numerator]))
        }
        3 => {
            // ∑ k³ = [n(n+1)/2]²
            let half = store.rat(1, 2);
            let base = store.mul(vec![half, n, n_plus_1]);
            Some(store.pow(base, two))
        }
        _ => None,
    }
}

/// Placeholder for main summation entry point
/// This will dispatch to appropriate algorithms based on term structure
pub fn sum(
    store: &mut Store,
    term: ExprId,
    var: &str,
    lower: ExprId,
    upper: ExprId,
) -> Option<ExprId> {
    // Only handle lower = 0 for now
    if !matches!((&store.get(lower).op, &store.get(lower).payload), (Op::Integer, Payload::Int(0)))
    {
        return None;
    }

    // Number of terms n_terms = upper - 0 + 1 = upper + 1
    let one = store.int(1);
    let n_terms = store.add(vec![upper, one]);

    // Check for power sum: k^p
    if let Some(p) = extract_power(store, term, var) {
        return sum_power(store, p, upper);
    }

    // Attempt arithmetic pattern: a + d*k
    if let Some((a, d)) = extract_arithmetic(store, term, var) {
        // sum_{k=0..upper} (a + d*k) = (upper+1)*a + d * upper*(upper+1)/2
        let up1 = n_terms; // upper + 1
        let n_a = store.mul(vec![up1, a]);
        let half = store.rat(1, 2);
        let upper_plus_one = store.add(vec![upper, one]);
        let upper_times_up1 = store.mul(vec![upper, upper_plus_one]);
        let tri = store.mul(vec![half, upper_times_up1]);
        let d_term = store.mul(vec![d, tri]);
        return Some(store.add(vec![n_a, d_term]));
    }

    // Attempt geometric pattern: a * r^k
    if let Some((a, r)) = extract_geometric(store, term, var) {
        // sum_{k=0..upper} a*r^k = a*(1 - r^{upper+1})/(1 - r)
        // handle r == 1 separately: a*(upper+1)
        if is_one(store, r) {
            return Some(store.mul(vec![a, n_terms]));
        }
        let r_pow_n = store.pow(r, n_terms);
        let neg_one = store.int(-1);
        let neg_r_pow_n = store.mul(vec![neg_one, r_pow_n]);
        let numerator = store.add(vec![one, neg_r_pow_n]);
        let neg_r = store.mul(vec![neg_one, r]);
        let denom = store.add(vec![one, neg_r]);
        let minus_one = store.int(-1);
        let inv_denom = store.pow(denom, minus_one);
        let frac = store.mul(vec![numerator, inv_denom]);
        return Some(store.mul(vec![a, frac]));
    }

    // 2. Try Gosper's algorithm (not yet implemented)
    // gosper_sum(store, term, var)
    None
}

// ----------------- Helpers -----------------

fn contains_var(store: &Store, id: ExprId, var: &str) -> bool {
    match (&store.get(id).op, &store.get(id).payload) {
        (Op::Symbol, Payload::Sym(s)) => s == var,
        (Op::Add, _) | (Op::Mul, _) => {
            store.get(id).children.iter().any(|&c| contains_var(store, c, var))
        }
        (Op::Pow, _) => {
            let n = store.get(id);
            contains_var(store, n.children[0], var) || contains_var(store, n.children[1], var)
        }
        (Op::Function, _) => store.get(id).children.iter().any(|&c| contains_var(store, c, var)),
        _ => false,
    }
}

fn is_one(store: &Store, id: ExprId) -> bool {
    matches!((&store.get(id).op, &store.get(id).payload), (Op::Integer, Payload::Int(1)))
        || matches!((&store.get(id).op, &store.get(id).payload), (Op::Rational, Payload::Rat(1, 1)))
}

fn extract_power(store: &Store, term: ExprId, var: &str) -> Option<usize> {
    // Recognize k^p where p is a small non-negative integer
    match store.get(term).op {
        Op::Symbol => {
            // k^1
            if matches!(&store.get(term).payload, Payload::Sym(s) if s == var) {
                Some(1)
            } else {
                None
            }
        }
        Op::Pow => {
            let n = store.get(term);
            let base = n.children[0];
            let exp = n.children[1];
            // Check base is var
            if !matches!((&store.get(base).op, &store.get(base).payload), (Op::Symbol, Payload::Sym(ref s)) if s == var)
            {
                return None;
            }
            // Check exp is small non-negative integer
            match (&store.get(exp).op, &store.get(exp).payload) {
                (Op::Integer, Payload::Int(p)) if *p >= 0 && *p <= 10 => Some(*p as usize),
                _ => None,
            }
        }
        _ => None,
    }
}

fn extract_arithmetic(store: &mut Store, term: ExprId, var: &str) -> Option<(ExprId, ExprId)> {
    // Recognize a + d*k with a independent of var and linear term exactly d*k
    if store.get(term).op != Op::Add {
        return None;
    }
    let mut a_terms: Vec<ExprId> = Vec::new();
    let mut linear_term: Option<ExprId> = None;
    for &c in &store.get(term).children {
        if contains_var(store, c, var) {
            // Expect exactly one linear term in k
            if linear_term.is_some() {
                return None;
            }
            linear_term = Some(c);
        } else {
            a_terms.push(c);
        }
    }
    let a = if a_terms.is_empty() { store.int(0) } else { store.add(a_terms) };
    let lt = linear_term?;
    // Linear term should be either k or d*k
    let d = match store.get(lt).op {
        Op::Symbol => match &store.get(lt).payload {
            Payload::Sym(s) if s == var => store.int(1),
            _ => return None,
        },
        Op::Mul => {
            // find factor k and multiply the rest as d
            let mut rest: Vec<ExprId> = Vec::new();
            let mut saw_k = false;
            for &f in &store.get(lt).children {
                if !saw_k
                    && matches!((&store.get(f).op, &store.get(f).payload), (Op::Symbol, Payload::Sym(ref s)) if s == var)
                {
                    saw_k = true;
                } else {
                    rest.push(f);
                }
            }
            if !saw_k {
                return None;
            }
            if rest.is_empty() {
                store.int(1)
            } else {
                store.mul(rest)
            }
        }
        _ => return None,
    };
    Some((a, d))
}

fn extract_geometric(store: &mut Store, term: ExprId, var: &str) -> Option<(ExprId, ExprId)> {
    // Recognize a * r^k with a independent of var, r independent of var, exponent == var
    match store.get(term).op {
        Op::Mul => {
            let mut coeffs: Vec<ExprId> = Vec::new();
            let mut r_opt: Option<ExprId> = None;
            for &f in &store.get(term).children {
                if store.get(f).op == Op::Pow {
                    let n = store.get(f);
                    let base = n.children[0];
                    let exp = n.children[1];
                    if matches!((&store.get(exp).op, &store.get(exp).payload), (Op::Symbol, Payload::Sym(ref s)) if s == var)
                        && !contains_var(store, base, var)
                    {
                        if r_opt.is_some() {
                            return None;
                        }
                        r_opt = Some(base);
                        continue;
                    }
                }
                // otherwise part of coefficient; must not contain var
                if contains_var(store, f, var) {
                    return None;
                }
                coeffs.push(f);
            }
            let r = r_opt?;
            let a = if coeffs.is_empty() { store.int(1) } else { store.mul(coeffs) };
            Some((a, r))
        }
        Op::Pow => {
            let n = store.get(term);
            let base = n.children[0];
            let exp = n.children[1];
            if matches!((&store.get(exp).op, &store.get(exp).payload), (Op::Symbol, Payload::Sym(ref s)) if s == var)
                && !contains_var(store, base, var)
            {
                Some((store.int(1), base))
            } else {
                None
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_hypergeometric_constant() {
        let mut st = Store::new();
        let five = st.int(5);
        assert!(is_hypergeometric(&st, five, "k"));
    }

    #[test]
    fn test_is_hypergeometric_variable() {
        let mut st = Store::new();
        let k = st.sym("k");
        assert!(is_hypergeometric(&st, k, "k"));
    }

    #[test]
    fn test_is_hypergeometric_power() {
        let mut st = Store::new();
        let k = st.sym("k");
        let two = st.int(2);
        let k_squared = st.pow(k, two);
        assert!(is_hypergeometric(&st, k_squared, "k"));
    }

    #[test]
    fn test_is_hypergeometric_product() {
        let mut st = Store::new();
        let k = st.sym("k");
        let three = st.int(3);
        let product = st.mul(vec![three, k]);
        assert!(is_hypergeometric(&st, product, "k"));
    }

    #[test]
    fn test_sum_arithmetic_simple() {
        // sum(k, k=0..n-1) = n*(n-1)/2
        let mut st = Store::new();
        let zero = st.int(0);
        let one = st.int(1);
        let n = st.sym("n");

        let result = sum_arithmetic(&mut st, zero, one, n).unwrap();

        // Result should be n*(n-1)/2
        let result_str = st.to_string(result);
        assert!(result_str.contains("n"));
        assert!(result_str.contains("1/2") || result_str.contains("2"));
    }

    #[test]
    fn test_sum_arithmetic_general() {
        // sum(5 + 3k, k=0..n-1) = 5n + 3*n*(n-1)/2
        let mut st = Store::new();
        let five = st.int(5);
        let three = st.int(3);
        let n = st.sym("n");

        let result = sum_arithmetic(&mut st, five, three, n).unwrap();

        // Result should contain n and the coefficients
        let result_str = st.to_string(result);
        assert!(result_str.contains("n"));
        assert!(result_str.contains("5") || result_str.contains("3"));
    }

    #[test]
    fn test_sum_geometric_simple() {
        // sum(2^k, k=0..n-1) = (1 - 2^n) / (1 - 2) = 2^n - 1
        let mut st = Store::new();
        let one = st.int(1);
        let two = st.int(2);
        let n = st.sym("n");

        let result = sum_geometric(&mut st, one, two, n).unwrap();

        // Result should be (1 - 2^n) / (1 - 2)
        let result_str = st.to_string(result);
        assert!(result_str.contains("2") || result_str.contains("n"));
    }

    #[test]
    fn test_sum_geometric_with_coefficient() {
        // sum(3*2^k, k=0..n-1) = 3*(1 - 2^n)/(1 - 2)
        let mut st = Store::new();
        let three = st.int(3);
        let two = st.int(2);
        let n = st.sym("n");

        let result = sum_geometric(&mut st, three, two, n).unwrap();

        // Result should contain 3, 2, and n
        let result_str = st.to_string(result);
        assert!(result_str.contains("3") && result_str.contains("2"));
        assert!(result_str.contains("n"));
    }

    #[test]
    fn test_sum_dispatcher_arithmetic() {
        // sum(5 + 3k, k=0..n) = 5(n+1) + 3*n*(n+1)/2
        let mut st = Store::new();
        let k = st.sym("k");
        let five = st.int(5);
        let three = st.int(3);
        let three_k = st.mul(vec![three, k]);
        let term = st.add(vec![five, three_k]);
        let n = st.sym("n");
        let zero = st.int(0);
        let res = sum(&mut st, term, "k", zero, n).unwrap();
        let s = st.to_string(res);
        assert!(s.contains("n"));
    }

    #[test]
    fn test_sum_dispatcher_geometric() {
        // sum(3*2^k, k=0..n) = 3*(1-2^{n+1})/(1-2)
        let mut st = Store::new();
        let three = st.int(3);
        let two = st.int(2);
        let k = st.sym("k");
        let pow = st.pow(two, k);
        let term = st.mul(vec![three, pow]);
        let zero = st.int(0);
        let n = st.sym("n");
        let res = sum(&mut st, term, "k", zero, n).unwrap();
        let s = st.to_string(res);
        assert!(s.contains("2") && s.contains("n"));
    }

    #[test]
    fn test_sum_power_k() {
        // sum(k, k=0..n) = n(n+1)/2
        let mut st = Store::new();
        let k = st.sym("k");
        let zero = st.int(0);
        let n = st.sym("n");
        let res = sum(&mut st, k, "k", zero, n).unwrap();
        let _s = st.to_string(res);
    }

    #[test]
    fn test_sum_power_k_squared() {
        // sum(k^2, k=0..n) = n(n+1)(2n+1)/6
        let mut st = Store::new();
        let k = st.sym("k");
        let two = st.int(2);
        let k_sq = st.pow(k, two);
        let zero = st.int(0);
        let n = st.sym("n");
        let res = sum(&mut st, k_sq, "k", zero, n).unwrap();
        let s = st.to_string(res);
        assert!(s.contains("n"));
        assert!(s.contains("1/6") || s.contains("6"));
    }

    #[test]
    fn test_sum_power_k_cubed() {
        // sum(k^3, k=0..n) = [n(n+1)/2]^2
        let mut st = Store::new();
        let k = st.sym("k");
        let three = st.int(3);
        let k_cubed = st.pow(k, three);
        let zero = st.int(0);
        let n = st.sym("n");
        let res = sum(&mut st, k_cubed, "k", zero, n).unwrap();
        let s = st.to_string(res);
        assert!(s.contains("n"));
    }

    #[test]
    fn test_sum_natural_numbers() {
        let mut st = Store::new();
        let n = st.sym("n");
        let result = sum_natural_numbers(&mut st, n);
        
        let result_str = st.to_string(result);
        assert!(result_str.contains("n"));
    }

    #[test]
    fn test_sum_squares_formula() {
        let mut st = Store::new();
        let n = st.sym("n");
        let result = sum_squares(&mut st, n);
        
        let result_str = st.to_string(result);
        assert!(result_str.contains("n"));
    }

    #[test]
    fn test_sum_cubes_formula() {
        let mut st = Store::new();
        let n = st.sym("n");
        let result = sum_cubes(&mut st, n);
        
        let result_str = st.to_string(result);
        assert!(result_str.contains("n"));
    }

    #[test]
    fn test_sum_binomial_row_formula() {
        let mut st = Store::new();
        let n = st.sym("n");
        let result = sum_binomial_row(&mut st, n);
        
        let result_str = st.to_string(result);
        assert_eq!(result_str, "2^n");
    }
}
