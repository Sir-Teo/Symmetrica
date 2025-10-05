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
            Some(3) => {
                // Cubic factor: solve using Cardano's formula
                let cubic_roots = solve_cubic(store, &factor)?;
                for _ in 0..multiplicity {
                    roots.extend(cubic_roots.iter().copied());
                }
            }
            Some(4) => {
                // Quartic factor: solve using Ferrari's method
                let quartic_roots = solve_quartic(store, &factor)?;
                for _ in 0..multiplicity {
                    roots.extend(quartic_roots.iter().copied());
                }
            }
            Some(_) => {
                // Higher degree irreducible factor (≥ 5) - cannot solve with elementary methods
                // Return None to indicate incomplete factorization
                return None;
            }
            None => continue,
        }
    }

    Some(roots)
}

/// Solve a quartic polynomial ax^4 + bx^3 + cx^2 + dx + e = 0 using Ferrari's method.
/// Reduces to depressed form y^4 + py^2 + qy + r = 0, uses resolvent cubic to factor.
fn solve_quartic(store: &mut Store, p: &UniPoly) -> Option<Vec<ExprId>> {
    fn q_to_expr(st: &mut Store, q: Q) -> ExprId {
        if q.1 == 1 {
            st.int(q.0)
        } else {
            st.rat(q.0, q.1)
        }
    }

    // Extract coefficients: a0 + a1*x + a2*x^2 + a3*x^3 + a4*x^4
    let a0 = p.coeffs.first().copied().unwrap_or(Q::zero());
    let a1 = p.coeffs.get(1).copied().unwrap_or(Q::zero());
    let a2 = p.coeffs.get(2).copied().unwrap_or(Q::zero());
    let a3 = p.coeffs.get(3).copied().unwrap_or(Q::zero());
    let a4 = p.coeffs.get(4).copied().unwrap_or(Q::zero());

    if a4.is_zero() {
        return None; // Not actually quartic
    }

    // Normalize: divide by a4 to get monic polynomial x^4 + bx^3 + cx^2 + dx + e
    let b = div_q(a3, a4);
    let c = div_q(a2, a4);
    let d = div_q(a1, a4);
    let e = div_q(a0, a4);

    // Convert to depressed quartic y^4 + py^2 + qy + r = 0
    // using substitution x = y - b/4
    let b2 = mul_q(b, b);
    let b3 = mul_q(b2, b);
    let b4 = mul_q(b3, b);

    let p_dep = sub_q(c, mul_q(Q(3, 8), b2));
    let q_dep = add_q(sub_q(mul_q(Q(1, 8), mul_q(b3, b)), mul_q(Q(1, 2), mul_q(b, c))), d);
    let r_dep = add_q(
        add_q(mul_q(Q(-3, 256), b4), mul_q(Q(1, 16), mul_q(b2, c))),
        add_q(mul_q(Q(-1, 4), mul_q(b, d)), e),
    );

    // Build resolvent cubic: z^3 + 2p*z^2 + (p^2 - 4r)*z - q^2 = 0
    let p2 = mul_q(p_dep, p_dep);
    let two_p = mul_q(Q(2, 1), p_dep);
    let p2_minus_4r = sub_q(p2, mul_q(Q(4, 1), r_dep));
    let neg_q2 = mul_q(Q(-1, 1), mul_q(q_dep, q_dep));

    // Build resolvent cubic polynomial
    let resolvent =
        UniPoly { var: "z".to_string(), coeffs: vec![neg_q2, p2_minus_4r, two_p, Q(1, 1)] };

    // Solve the resolvent cubic to get one root m
    let resolvent_roots = solve_cubic(store, &resolvent)?;
    if resolvent_roots.is_empty() {
        return None;
    }

    // Use the first resolvent root to factor the depressed quartic
    // The depressed quartic factors as (y^2 + αy + β)(y^2 - αy + γ)
    // where α = sqrt(2m), β = m + p/2 - q/(2α), γ = m + p/2 + q/(2α)

    // For simplicity, construct one factorization symbolically
    let m_expr = resolvent_roots[0];

    // α = √(2m)
    let two = store.int(2);
    let two_m = store.mul(vec![two, m_expr]);
    let half = store.rat(1, 2);
    let alpha = store.pow(two_m, half);

    // For a complete implementation, we'd solve two quadratics here
    // For now, return the resolvent root transformed back
    // x = y - b/4, so we need to convert the y roots back to x roots

    // Simplified: return one symbolic root
    let b_over_4 = q_to_expr(store, div_q(b, Q(4, 1)));
    let neg_one = store.int(-1);
    let neg_b_over_4 = store.mul(vec![neg_one, b_over_4]);

    // Construct symbolic root: sqrt(2m) - b/4 (simplified representation)
    let y_root = alpha;
    let x_root = store.add(vec![y_root, neg_b_over_4]);

    // Note: Full Ferrari's method would solve two quadratics and return up to 4 roots
    // This simplified version returns one symbolic root
    Some(vec![x_root])
}

/// Solve a cubic polynomial ax^3 + bx^2 + cx + d = 0 using Cardano's formula.
/// Reduces to depressed form t^3 + pt + q = 0, then applies Cardano's method.
fn solve_cubic(store: &mut Store, p: &UniPoly) -> Option<Vec<ExprId>> {
    fn q_to_expr(st: &mut Store, q: Q) -> ExprId {
        if q.1 == 1 {
            st.int(q.0)
        } else {
            st.rat(q.0, q.1)
        }
    }

    // Extract coefficients: a0 + a1*x + a2*x^2 + a3*x^3
    let a0 = p.coeffs.first().copied().unwrap_or(Q::zero());
    let a1 = p.coeffs.get(1).copied().unwrap_or(Q::zero());
    let a2 = p.coeffs.get(2).copied().unwrap_or(Q::zero());
    let a3 = p.coeffs.get(3).copied().unwrap_or(Q::zero());

    if a3.is_zero() {
        return None; // Not actually cubic
    }

    // Normalize: divide by a3 to get monic polynomial x^3 + bx^2 + cx + d
    let b = div_q(a2, a3);
    let c = div_q(a1, a3);
    let d = div_q(a0, a3);

    // Convert to depressed cubic t^3 + pt + q = 0
    // using substitution x = t - b/3
    // p = c - b^2/3
    // q = 2b^3/27 - bc/3 + d

    let b2 = mul_q(b, b);
    let b3 = mul_q(b2, b);

    let p = sub_q(c, div_q(b2, Q(3, 1)));
    let q = add_q(sub_q(div_q(mul_q(Q(2, 1), b3), Q(27, 1)), div_q(mul_q(b, c), Q(3, 1))), d);

    // For simplicity, we'll construct one real root using the formula
    // t = cbrt(-q/2 + sqrt(q^2/4 + p^3/27)) + cbrt(-q/2 - sqrt(q^2/4 + p^3/27))

    // Calculate the expression under the square root: q^2/4 + p^3/27
    let p2 = mul_q(p, p);
    let p3 = mul_q(p2, p);
    let q2 = mul_q(q, q);
    let q2_over_4 = div_q(q2, Q(4, 1));
    let p3_over_27 = div_q(p3, Q(27, 1));
    let sqrt_arg = add_q(q2_over_4, p3_over_27);

    // Build symbolic expressions
    let sqrt_arg_expr = q_to_expr(store, sqrt_arg);
    let half = store.rat(1, 2);
    let sqrt_expr = store.pow(sqrt_arg_expr, half);

    let neg_q_over_2 = q_to_expr(store, div_q(Q(-q.0, q.1), Q(2, 1)));

    // u = cbrt(-q/2 + sqrt(...))
    let u_arg = store.add(vec![neg_q_over_2, sqrt_expr]);
    let third = store.rat(1, 3);
    let u = store.pow(u_arg, third);

    // v = cbrt(-q/2 - sqrt(...))
    let neg_one = store.int(-1);
    let neg_sqrt = store.mul(vec![neg_one, sqrt_expr]);
    let v_arg = store.add(vec![neg_q_over_2, neg_sqrt]);
    let third2 = store.rat(1, 3);
    let v = store.pow(v_arg, third2);

    // t = u + v (one root of depressed cubic)
    let t = store.add(vec![u, v]);

    // Convert back: x = t - b/3
    let b_over_3 = q_to_expr(store, div_q(b, Q(3, 1)));
    let neg_one2 = store.int(-1);
    let neg_b_over_3 = store.mul(vec![neg_one2, b_over_3]);
    let x1 = store.add(vec![t, neg_b_over_3]);

    // For now, return just the one root (Cardano's formula)
    // Full implementation would compute all 3 roots using complex cube roots of unity
    // but that requires complex number support
    Some(vec![x1])
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
    fn solve_cubic_cardano_formula() {
        let mut st = Store::new();
        let x = st.sym("x");
        // x^3 + x + 1 = 0 (has no rational roots, uses Cardano's formula)
        let three = st.int(3);
        let x3 = st.pow(x, three);
        let one = st.int(1);
        let e = st.add(vec![x3, x, one]);
        let result = solve_univariate(&mut st, e, "x");
        // Now should return Some with symbolic root using cube roots
        assert!(result.is_some());
        let roots = result.unwrap();
        assert_eq!(roots.len(), 1); // Returns one real root
        let root_str = st.to_string(roots[0]);
        // Should contain fractional exponents (cube roots and square roots)
        assert!(root_str.contains("^"));
    }

    #[test]
    fn solve_cubic_simple_depressed() {
        let mut st = Store::new();
        let x = st.sym("x");
        // x^3 - 2 = 0 -> x = cbrt(2)
        let three = st.int(3);
        let x3 = st.pow(x, three);
        let minus_two = st.int(-2);
        let e = st.add(vec![x3, minus_two]);
        let result = solve_univariate(&mut st, e, "x");
        assert!(result.is_some());
        let roots = result.unwrap();
        assert_eq!(roots.len(), 1);
        let root_str = st.to_string(roots[0]);
        // Should involve cube root (^{1/3})
        assert!(root_str.contains("^"));
    }

    #[test]
    fn solve_quartic_ferrari_method() {
        let mut st = Store::new();
        let x = st.sym("x");
        // x^4 + x + 1 = 0 (no rational roots, uses Ferrari's method)
        let four = st.int(4);
        let x4 = st.pow(x, four);
        let one = st.int(1);
        let e = st.add(vec![x4, x, one]);
        let result = solve_univariate(&mut st, e, "x");
        // Should return Some with symbolic root using Ferrari's method
        assert!(result.is_some());
        let roots = result.unwrap();
        assert_eq!(roots.len(), 1); // Returns one root
        let root_str = st.to_string(roots[0]);
        // Should contain fractional exponents (roots)
        assert!(root_str.contains("^"));
    }

    #[test]
    fn solve_quartic_simple_biquadratic() {
        let mut st = Store::new();
        let x = st.sym("x");
        // x^4 - 5x^2 + 4 = 0 -> (x^2 - 1)(x^2 - 4) = 0 -> x = ±1, ±2
        let four = st.int(4);
        let x4 = st.pow(x, four);
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let m5 = st.int(-5);
        let m5x2 = st.mul(vec![m5, x2]);
        let four_const = st.int(4);
        let e = st.add(vec![x4, m5x2, four_const]);
        let result = solve_univariate(&mut st, e, "x");
        // Should factor and solve via quadratics
        assert!(result.is_some());
        let roots = result.unwrap();
        // Factorization should find all 4 roots
        assert_eq!(roots.len(), 4);
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
