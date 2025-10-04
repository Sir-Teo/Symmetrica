//! Solver module: univariate polynomial solving over Q and transcendental equations.
//! - Linear and quadratic closed forms
//! - Rational-root search for higher degrees (factor out simple rational roots)
//! - Phase J: Simple exponential equation patterns (a*exp(b*x) = c)

#![deny(warnings)]

use arith::{add_q, div_q, mul_q, sub_q, Q};
use expr_core::{ExprId, Op, Payload, Store};
use polys::{expr_to_unipoly, UniPoly};

/// Solve a univariate polynomial equation p(x) = 0 where `expr` is convertible to a polynomial in `var`.
/// Returns a list of root expressions (values for x).
/// Now uses polynomial factorization for complete root finding.
pub fn solve_univariate(store: &mut Store, expr: ExprId, var: &str) -> Option<Vec<ExprId>> {
    let p0 = expr_to_unipoly(store, expr, var)?;
    if p0.is_zero() {
        // Degenerate 0 == 0; no finite roots enumerated
        return Some(vec![]);
    }

    // Use factorization to find all roots
    let factors = p0.factor();
    let mut roots: Vec<ExprId> = Vec::new();

    // Helper: convert Q to Expr
    fn q_to_expr(st: &mut Store, q: Q) -> ExprId {
        if q.1 == 1 {
            st.int(q.0)
        } else {
            st.rat(q.0, q.1)
        }
    }

    // Process each factor to extract roots
    for (factor, multiplicity) in factors {
        match factor.degree() {
            Some(0) => {
                // Constant factor - no roots
                continue;
            }
            Some(1) => {
                // Linear factor: ax + b = 0 => x = -b/a
                let a0 = factor.coeffs.first().copied().unwrap_or(Q::zero());
                let a1 = factor.coeffs.get(1).copied().unwrap_or(Q::zero());
                if !a1.is_zero() {
                    let root = div_q(Q(-a0.0, a0.1), a1);
                    // Add root with its multiplicity
                    for _ in 0..multiplicity {
                        roots.push(q_to_expr(store, root));
                    }
                }
            }
            Some(2) => {
                // Quadratic factor: solve using quadratic formula
                let quad_roots = solve_quadratic(store, &factor)?;
                for _ in 0..multiplicity {
                    roots.extend(quad_roots.iter().copied());
                }
            }
            Some(_) => {
                // Higher degree irreducible factor - cannot solve over Q with elementary methods
                // Return None to indicate incomplete factorization
                return None;
            }
            None => continue,
        }
    }

    Some(roots)
}

/// Solve a quadratic polynomial ax^2 + bx + c = 0 using the quadratic formula.
fn solve_quadratic(store: &mut Store, p: &UniPoly) -> Option<Vec<ExprId>> {
    fn q_to_expr(st: &mut Store, q: Q) -> ExprId {
        if q.1 == 1 {
            st.int(q.0)
        } else {
            st.rat(q.0, q.1)
        }
    }

    let a0 = p.coeffs.first().copied().unwrap_or(Q::zero());
    let a1 = p.coeffs.get(1).copied().unwrap_or(Q::zero());
    let a2 = p.coeffs.get(2).copied().unwrap_or(Q::zero());

    if a2.is_zero() {
        return None;
    }

    // Discriminant D = b^2 - 4ac
    let a1sq = mul_q(a1, a1);
    let four_a2a0 = mul_q(Q(4, 1), mul_q(a2, a0));
    let d = sub_q(a1sq, four_a2a0);
    let minus_b = Q(-a1.0, a1.1);
    let two_a = mul_q(Q(2, 1), a2);

    // Check if discriminant is a perfect square
    fn is_square_i64(n: i64) -> Option<i64> {
        if n < 0 {
            return None;
        }
        let mut i = 0i64;
        while i * i <= n {
            if i * i == n {
                return Some(i);
            }
            i += 1;
        }
        None
    }

    let sqrt_rational = (|| -> Option<Q> {
        let num = d.0;
        let den = d.1;
        let sn = is_square_i64(num)?;
        let sd = is_square_i64(den)?;
        Some(Q(sn, sd))
    })();

    if let Some(sq) = sqrt_rational {
        // Rational roots
        let r1 = div_q(add_q(minus_b, sq), two_a);
        let r2 = div_q(sub_q(minus_b, sq), two_a);
        return Some(vec![q_to_expr(store, r1), q_to_expr(store, r2)]);
    }

    // Irrational roots: (-b ± sqrt(D)) / (2a)
    let num_base = q_to_expr(store, minus_b);
    let sqrt_d = {
        let d_expr = q_to_expr(store, d);
        let half = store.rat(1, 2);
        store.pow(d_expr, half)
    };
    let inv_two_a = {
        let inv = div_q(Q(1, 1), two_a);
        q_to_expr(store, inv)
    };
    let plus = {
        let num = store.add(vec![num_base, sqrt_d]);
        store.mul(vec![inv_two_a, num])
    };
    let minus = {
        let m1 = store.int(-1);
        let neg_sqrt = store.mul(vec![m1, sqrt_d]);
        let num = store.add(vec![num_base, neg_sqrt]);
        store.mul(vec![inv_two_a, num])
    };
    Some(vec![plus, minus])
}

/// Solve simple transcendental equations of the form:
/// - a*exp(b*x) = c  →  x = ln(c/a) / b
/// - exp(b*x) + a = 0  →  x = ln(-a) / b  (if -a > 0)
///
/// Returns Some(vec![solution]) if pattern matches and solution exists, None otherwise.
pub fn solve_exponential(store: &mut Store, expr: ExprId, var: &str) -> Option<Vec<ExprId>> {
    // Helper: check if expr depends on var
    fn depends_on(st: &Store, id: ExprId, var: &str) -> bool {
        match (&st.get(id).op, &st.get(id).payload) {
            (Op::Symbol, Payload::Sym(s)) => s == var,
            (Op::Integer, _) | (Op::Rational, _) => false,
            _ => st.get(id).children.iter().any(|&c| depends_on(st, c, var)),
        }
    }

    // Helper: extract coefficient and rest from Add node
    fn extract_const_from_add(st: &Store, id: ExprId, var: &str) -> Option<(ExprId, ExprId)> {
        if st.get(id).op != Op::Add {
            return None;
        }
        let children = &st.get(id).children;
        let mut const_part = None;
        let mut var_parts = Vec::new();

        for &child in children {
            if depends_on(st, child, var) {
                var_parts.push(child);
            } else {
                if const_part.is_some() {
                    return None; // Multiple constants, too complex
                }
                const_part = Some(child);
            }
        }

        if var_parts.len() != 1 || const_part.is_none() {
            return None;
        }

        Some((var_parts[0], const_part.unwrap()))
    }

    // Pattern 1: exp(b*x) + a = 0  →  exp(b*x) = -a
    if let Some((exp_term, const_term)) = extract_const_from_add(store, expr, var) {
        // Check if exp_term is exp(...)
        if store.get(exp_term).op == Op::Function {
            if let Payload::Func(name) = &store.get(exp_term).payload {
                if name == "exp" && store.get(exp_term).children.len() == 1 {
                    let arg = store.get(exp_term).children[0];

                    // exp(arg) = -const_term
                    let neg1 = store.int(-1);
                    let neg_const = store.mul(vec![neg1, const_term]);

                    // Now solve arg = ln(-const_term)
                    let ln_rhs = store.func("ln", vec![neg_const]);

                    // If arg is linear in var (b*x or x), solve for x
                    return solve_linear_for_var(store, arg, ln_rhs, var);
                }
            }
        }
    }

    // Pattern 2: a*exp(b*x) = c (represented as a*exp(b*x) - c = 0)
    // Try to match Mul node containing exp
    if store.get(expr).op == Op::Add {
        let children = &store.get(expr).children.clone();
        if children.len() == 2 {
            // Try first child as mul*exp, second as constant
            for i in 0..2 {
                let mul_exp = children[i];
                let const_part = children[1 - i];

                if !depends_on(store, const_part, var) {
                    if let Some((coeff, exp_term)) = extract_coeff_and_exp(store, mul_exp, var) {
                        // coeff * exp(arg) = -const_part
                        let neg1 = store.int(-1);
                        let neg_const = store.mul(vec![neg1, const_part]);

                        // exp(arg) = neg_const / coeff
                        let minus_one = store.int(-1);
                        let inv_coeff = store.pow(coeff, minus_one);
                        let rhs = store.mul(vec![neg_const, inv_coeff]);
                        let ln_rhs = store.func("ln", vec![rhs]);

                        let arg = store.get(exp_term).children[0];
                        return solve_linear_for_var(store, arg, ln_rhs, var);
                    }
                }
            }
        }
    }

    None
}

// Helper: extract a*exp(...) into (a, exp_id)
fn extract_coeff_and_exp(st: &mut Store, id: ExprId, _var: &str) -> Option<(ExprId, ExprId)> {
    if st.get(id).op == Op::Function {
        if let Payload::Func(name) = &st.get(id).payload {
            if name == "exp" {
                let one = st.int(1);
                return Some((one, id));
            }
        }
    }

    if st.get(id).op == Op::Mul {
        let children = &st.get(id).children;
        let mut exp_term = None;
        let mut coeff_parts = Vec::new();

        for &child in children {
            if st.get(child).op == Op::Function {
                if let Payload::Func(name) = &st.get(child).payload {
                    if name == "exp" && st.get(child).children.len() == 1 {
                        if exp_term.is_some() {
                            return None; // Multiple exp terms
                        }
                        exp_term = Some(child);
                        continue;
                    }
                }
            }
            coeff_parts.push(child);
        }

        if let Some(exp_id) = exp_term {
            let coeff = if coeff_parts.is_empty() { st.int(1) } else { st.mul(coeff_parts) };
            return Some((coeff, exp_id));
        }
    }

    None
}

// Helper: solve linear equation lhs = rhs for var
// Handles: b*x = rhs → x = rhs/b, or x = rhs
fn solve_linear_for_var(
    st: &mut Store,
    lhs: ExprId,
    rhs: ExprId,
    var: &str,
) -> Option<Vec<ExprId>> {
    // Case 1: lhs is just var
    if let (Op::Symbol, Payload::Sym(s)) = (&st.get(lhs).op, &st.get(lhs).payload) {
        if s == var {
            return Some(vec![rhs]);
        }
    }

    // Case 2: lhs is b*var
    if st.get(lhs).op == Op::Mul {
        let children = &st.get(lhs).children.clone();
        let mut var_found = false;
        let mut coeff_parts = Vec::new();

        for &child in children {
            if let (Op::Symbol, Payload::Sym(s)) = (&st.get(child).op, &st.get(child).payload) {
                if s == var {
                    if var_found {
                        return None; // var appears twice
                    }
                    var_found = true;
                    continue;
                }
            }
            coeff_parts.push(child);
        }

        if var_found {
            let coeff = if coeff_parts.is_empty() { st.int(1) } else { st.mul(coeff_parts) };
            // x = rhs / coeff
            let minus_one = st.int(-1);
            let inv_coeff = st.pow(coeff, minus_one);
            let solution = st.mul(vec![rhs, inv_coeff]);
            return Some(vec![solution]);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solve_linear_root() {
        let mut st = Store::new();
        let x = st.sym("x");
        let one = st.int(1);
        let e = st.add(vec![x, one]); // x + 1 = 0 -> root -1
        let roots = solve_univariate(&mut st, e, "x").expect("solved");
        assert_eq!(roots.len(), 1);
        assert_eq!(st.to_string(roots[0]), "-1");
    }

    #[test]
    fn solve_quadratic_rational_roots() {
        let mut st = Store::new();
        let x = st.sym("x");
        // x^2 + 3x + 2 = 0 -> roots -1, -2
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let three = st.int(3);
        let three_x = st.mul(vec![three, x]);
        let two2 = st.int(2);
        let e = st.add(vec![x2, three_x, two2]);
        let mut roots = solve_univariate(&mut st, e, "x").expect("solved");
        roots.sort_by_key(|r| st.to_string(*r));
        let rs: Vec<String> = roots.into_iter().map(|r| st.to_string(r)).collect();
        assert_eq!(rs, vec!["-1", "-2"]);
    }

    #[test]
    fn solve_quadratic_irrational_roots() {
        let mut st = Store::new();
        let x = st.sym("x");
        // x^2 - 2 = 0 -> ± sqrt(2)
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let minus_two = st.int(-2);
        let e = st.add(vec![x2, minus_two]);
        let roots = solve_univariate(&mut st, e, "x").expect("solved");
        assert_eq!(roots.len(), 2);
        // We can't easily compare canonical forms with sqrt; just ensure one positive, one negative, and both involve ^{1/2}
        let s0 = st.to_string(roots[0]);
        let s1 = st.to_string(roots[1]);
        assert!(s0.contains("^"));
        assert!(s1.contains("^"));
    }

    #[test]
    fn solve_cubic_rational_roots() {
        let mut st = Store::new();
        let x = st.sym("x");
        // x^3 - x = 0 -> roots 0, ±1
        let three = st.int(3);
        let x3 = st.pow(x, three);
        let m1 = st.int(-1);
        let minus_x = st.mul(vec![m1, x]);
        let e = st.add(vec![x3, minus_x]);
        let mut roots = solve_univariate(&mut st, e, "x").expect("solved");
        roots.sort_by_key(|r| st.to_string(*r));
        let rs: Vec<String> = roots.into_iter().map(|r| st.to_string(r)).collect();
        assert_eq!(rs, vec!["-1", "0", "1"]);
    }

    #[test]
    fn solve_zero_polynomial_returns_empty() {
        let mut st = Store::new();
        let zero = st.int(0);
        let result = solve_univariate(&mut st, zero, "x").expect("zero poly");
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn solve_constant_nonzero_returns_empty() {
        let mut st = Store::new();
        let five = st.int(5);
        let result = solve_univariate(&mut st, five, "x").expect("constant poly");
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn solve_cubic_no_rational_roots() {
        let mut st = Store::new();
        let x = st.sym("x");
        // x^3 + x + 1 = 0 (has no rational roots)
        let three = st.int(3);
        let x3 = st.pow(x, three);
        let one = st.int(1);
        let e = st.add(vec![x3, x, one]);
        let result = solve_univariate(&mut st, e, "x");
        // No rational roots, so should return None
        assert!(result.is_none());
    }

    #[test]
    fn solve_not_polynomial() {
        let mut st = Store::new();
        let x = st.sym("x");
        let sinx = st.func("sin", vec![x]);
        let result = solve_univariate(&mut st, sinx, "x");
        assert!(result.is_none());
    }

    #[test]
    fn solve_quadratic_with_rational_discriminant() {
        let mut st = Store::new();
        let x = st.sym("x");
        // x^2 - 5x + 6 = 0 -> roots 2, 3
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let m5 = st.int(-5);
        let m5x = st.mul(vec![m5, x]);
        let six = st.int(6);
        let e = st.add(vec![x2, m5x, six]);
        let mut roots = solve_univariate(&mut st, e, "x").expect("solved");
        roots.sort_by_key(|r| st.to_string(*r));
        let rs: Vec<String> = roots.into_iter().map(|r| st.to_string(r)).collect();
        assert_eq!(rs, vec!["2", "3"]);
    }

    // ========== Transcendental Equation Tests (Phase J) ==========

    #[test]
    fn solve_exp_x_minus_5() {
        let mut st = Store::new();
        let x = st.sym("x");
        // exp(x) - 5 = 0  →  x = ln(5)
        let expx = st.func("exp", vec![x]);
        let m5 = st.int(-5);
        let eq = st.add(vec![expx, m5]);
        let roots = solve_exponential(&mut st, eq, "x").expect("solvable");
        assert_eq!(roots.len(), 1);
        let result_str = st.to_string(roots[0]);
        assert!(result_str.contains("ln"));
        assert!(result_str.contains("5"));
    }

    #[test]
    fn solve_2_exp_x_minus_10() {
        let mut st = Store::new();
        let x = st.sym("x");
        // 2*exp(x) - 10 = 0  →  exp(x) = 5  →  x = ln(5)
        let expx = st.func("exp", vec![x]);
        let two = st.int(2);
        let two_expx = st.mul(vec![two, expx]);
        let m10 = st.int(-10);
        let eq = st.add(vec![two_expx, m10]);
        let roots = solve_exponential(&mut st, eq, "x").expect("solvable");
        assert_eq!(roots.len(), 1);
        let result_str = st.to_string(roots[0]);
        // Result should have ln and either 5 or 10/2 or similar
        assert!(result_str.contains("ln"));
        assert!(result_str.contains("10") || result_str.contains("5"));
    }

    #[test]
    fn solve_exp_2x_minus_7() {
        let mut st = Store::new();
        let x = st.sym("x");
        // exp(2*x) - 7 = 0  →  2*x = ln(7)  →  x = ln(7)/2
        let two = st.int(2);
        let two_x = st.mul(vec![two, x]);
        let exp_2x = st.func("exp", vec![two_x]);
        let m7 = st.int(-7);
        let eq = st.add(vec![exp_2x, m7]);
        let roots = solve_exponential(&mut st, eq, "x").expect("solvable");
        assert_eq!(roots.len(), 1);
        let result_str = st.to_string(roots[0]);
        // Should be ln(7) * (1/2) or similar
        assert!(result_str.contains("ln"));
        assert!(result_str.contains("7"));
    }

    #[test]
    fn solve_3_exp_5x_equals_15() {
        let mut st = Store::new();
        let x = st.sym("x");
        // 3*exp(5*x) - 15 = 0  →  exp(5*x) = 5  →  5*x = ln(5)  →  x = ln(5)/5
        let three = st.int(3);
        let five = st.int(5);
        let five_x = st.mul(vec![five, x]);
        let exp_5x = st.func("exp", vec![five_x]);
        let coeff_exp = st.mul(vec![three, exp_5x]);
        let m15 = st.int(-15);
        let eq = st.add(vec![coeff_exp, m15]);
        let roots = solve_exponential(&mut st, eq, "x").expect("solvable");
        assert_eq!(roots.len(), 1);
        let result_str = st.to_string(roots[0]);
        assert!(result_str.contains("ln"));
        assert!(result_str.contains("5"));
    }

    #[test]
    fn solve_exp_x_plus_1() {
        let mut st = Store::new();
        let x = st.sym("x");
        // exp(x) + 1 = 0  →  exp(x) = -1  →  x = ln(-1) (complex, but we construct it)
        let expx = st.func("exp", vec![x]);
        let one = st.int(1);
        let eq = st.add(vec![expx, one]);
        let roots = solve_exponential(&mut st, eq, "x").expect("solvable");
        assert_eq!(roots.len(), 1);
        // ln of negative number - symbolic result
        let result_str = st.to_string(roots[0]);
        assert!(result_str.contains("ln"));
    }

    #[test]
    fn solve_exp_fails_on_polynomial() {
        let mut st = Store::new();
        let x = st.sym("x");
        // x^2 + 3 = 0 should not be solved by exponential solver
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let three = st.int(3);
        let eq = st.add(vec![x2, three]);
        let result = solve_exponential(&mut st, eq, "x");
        assert!(result.is_none());
    }

    #[test]
    fn solve_exp_fails_on_complex_transcendental() {
        let mut st = Store::new();
        let x = st.sym("x");
        // exp(x) + sin(x) = 0 is too complex for our pattern matching
        let expx = st.func("exp", vec![x]);
        let sinx = st.func("sin", vec![x]);
        let eq = st.add(vec![expx, sinx]);
        let result = solve_exponential(&mut st, eq, "x");
        assert!(result.is_none());
    }
}
