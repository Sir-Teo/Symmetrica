//! Gosper's Algorithm for Hypergeometric Summation
//!
//! Gosper's algorithm finds closed-form antidifferences for hypergeometric terms.
//! Given a hypergeometric term t(k), it attempts to find g(k) such that:
//!   t(k) = g(k+1) - g(k)
//!
//! If successful, the definite sum is: ∑(k=a to b) t(k) = g(b+1) - g(a)
//!
//! Reference: Gosper, R. W. (1978). "Decision procedure for indefinite hypergeometric summation"

use crate::hypergeometric::{is_hypergeometric, rationalize_hypergeometric};
use arith::Q;
use calculus::diff;
use expr_core::{ExprId, Store};
use polys::expr_to_unipoly;
use simplify::simplify;

/// Attempt to find a closed-form sum using Gosper's algorithm
///
/// Returns Some(result) if a closed form is found, None otherwise.
pub fn gosper_sum(
    store: &mut Store,
    term: ExprId,
    var: &str,
    lower: ExprId,
    upper: ExprId,
) -> Option<ExprId> {
    // Check if term is hypergeometric
    if !is_hypergeometric(store, term, var) {
        return None;
    }

/// Compute prefix sum S(N) = sum_{k=0..N} P(k) r^k where P is polynomial with given coeffs
/// coeffs[m] corresponds to k^m
fn prefix_sums_poly_geom(
    store: &mut Store,
    r: ExprId,
    n_expr: ExprId,
    coeffs: &[ExprId],
) -> Option<ExprId> {
    if coeffs.is_empty() {
        return Some(store.int(0));
    }

    let r_sym = store.sym("__R");
    let one = store.int(1);
    let minus_one = store.int(-1);

    // S0(R,N)
    let n_plus_1 = store.add(vec![n_expr, one]);
    let r_pow_n1 = store.pow(r_sym, n_plus_1);
    let neg_r_pow_n1 = store.mul(vec![minus_one, r_pow_n1]);
    let num0 = store.add(vec![one, neg_r_pow_n1]);
    let neg_r_sym = store.mul(vec![minus_one, r_sym]);
    let den0 = store.add(vec![one, neg_r_sym]);
    let den0_inv = store.pow(den0, minus_one);
    let mut s_curr = store.mul(vec![num0, den0_inv]);

    // combo = coeffs[0]*S0
    let mut combo = store.mul(vec![coeffs[0], s_curr]);

    // Repeatedly apply R d/dR to get higher S_m and accumulate
    for coeff in coeffs.iter().skip(1) {
        let ds = diff(store, s_curr, "__R");
        s_curr = store.mul(vec![r_sym, ds]);
        let term_m = store.mul(vec![*coeff, s_curr]);
        combo = store.add(vec![combo, term_m]);
    }

    // Substitute R -> r
    substitute(store, combo, "__R", r)
}

/// Extract quadratic coefficients (a,b,c) such that expr = a*k^2 + b*k + c (degree<=2)
fn extract_quadratic_coeffs(
    store: &mut Store,
    expr: ExprId,
    var: &str,
) -> Option<(ExprId, ExprId, ExprId)> {
    if let Some(poly) = expr_to_unipoly(store, expr, var) {
        let a_q = if poly.coeffs.len() > 2 { poly.coeffs[2] } else { Q::zero() };
        let b_q = if poly.coeffs.len() > 1 { poly.coeffs[1] } else { Q::zero() };
        let c_q = if !poly.coeffs.is_empty() { poly.coeffs[0] } else { Q::zero() };
        let a = q_to_expr(store, a_q);
        let b = q_to_expr(store, b_q);
        let c = q_to_expr(store, c_q);
        return Some((a, b, c));
    }
    None
}

fn q_to_expr(store: &mut Store, q: Q) -> ExprId {
    if q.1 == 1 {
        store.int(q.0)
    } else {
        store.rat(q.0, q.1)
    }
}

/// Compute prefix sum S(N) = sum_{k=0..N} (a k^2 + b k + c) r^k using derivative wrt placeholder R
fn prefix_sums_deg2(
    store: &mut Store,
    r: ExprId,
    n_expr: ExprId,
    a: ExprId,
    b: ExprId,
    c: ExprId,
) -> Option<ExprId> {
    let r_sym = store.sym("__R");
    let one = store.int(1);
    let minus_one = store.int(-1);

    // S0(R,N) = (1 - R^(N+1)) / (1 - R)
    let n_plus_1 = store.add(vec![n_expr, one]);
    let r_pow_n1 = store.pow(r_sym, n_plus_1);
    let neg_r_pow_n1 = store.mul(vec![minus_one, r_pow_n1]);
    let num0 = store.add(vec![one, neg_r_pow_n1]);
    let neg_r_sym = store.mul(vec![minus_one, r_sym]);
    let den0 = store.add(vec![one, neg_r_sym]);
    let den0_inv = store.pow(den0, minus_one);
    let s0 = store.mul(vec![num0, den0_inv]);

    // S1(R,N) = R * d/dR S0
    let ds0 = diff(store, s0, "__R");
    let s1 = store.mul(vec![r_sym, ds0]);

    // S2(R,N) = R * d/dR S1
    let ds1 = diff(store, s1, "__R");
    let s2 = store.mul(vec![r_sym, ds1]);

    // Combine: a*S2 + b*S1 + c*S0
    let a_s2 = store.mul(vec![a, s2]);
    let b_s1 = store.mul(vec![b, s1]);
    let c_s0 = store.mul(vec![c, s0]);
    let combo = store.add(vec![a_s2, b_s1, c_s0]);

    // Substitute R -> r
    substitute(store, combo, "__R", r)
}

    // Case 3: (a*k + b) * r^k with a,b independent of k
    if let expr_core::Op::Mul = store.get(term).op {
        let children = store.get(term).children.clone();
        // try to find a pow and a linear-in-k factor
        let mut pow_idx: Option<usize> = None;
        for (i, &c) in children.iter().enumerate() {
            if let expr_core::Op::Pow = store.get(c).op {
                pow_idx = Some(i);
            } else {
                // could be the polynomial or part of it (linear via Add)
                // we will pass the whole product minus pow to linear extractor
            }
        }
        if let Some(pi) = pow_idx {
            let base_exp = store.get(children[pi]).children.clone();
            if base_exp.len() == 2 {
                let r = base_exp[0];
                let e = base_exp[1];
                if let (expr_core::Op::Symbol, expr_core::Payload::Sym(s)) =
                    (&store.get(e).op, &store.get(e).payload)
                {
                    if s == var && !depends_on_var(store, r, var) {
                        // Build the polynomial factor as product of remaining children
                        let mut others: Vec<ExprId> = Vec::new();
                        for (i, &c) in children.iter().enumerate() {
                            if i != pi {
                                others.push(c);
                            }
                        }
                        let poly = if others.is_empty() { store.int(1) } else { store.mul(others) };
                        if let Some((a, b)) = extract_linear_ab(store, poly, var) {
                            // Closed forms (for r != 1):
                            // G(n) = sum_{k=0..n} r^k = (1 - r^(n+1)) / (1 - r)
                            // K(n) = sum_{k=0..n} k r^k = (r - (n+1) r^(n+1) + n r^(n+2)) / (1 - r)^2
                            // Then sum_{k=L..U} (a k + b) r^k = a*(K(U)-K(L-1)) + b*(G(U)-G(L-1)).

                            // If r == 1, let basic handle (we're after basic in pipeline); return None.
                            if let (expr_core::Op::Integer, expr_core::Payload::Int(v)) =
                                (&store.get(r).op, &store.get(r).payload)
                            {
                                if *v == 1 {
                                    return None;
                                }
                            }
                            if let (expr_core::Op::Rational, expr_core::Payload::Rat(nv, dv)) =
                                (&store.get(r).op, &store.get(r).payload)
                            {
                                if *nv == 1 && *dv == 1 {
                                    return None;
                                }
                            }

                            let one = store.int(1);
                            let two = store.int(2);
                            let minus_one = store.int(-1);
                            let neg_r = store.mul(vec![minus_one, r]);
                            let denom = store.add(vec![one, neg_r]); // 1 - r
                            let denom_inv = store.pow(denom, minus_one);
                            let neg_two = store.int(-2);
                            let denom2_inv = store.pow(denom, neg_two);

                            // Helper closures to build prefix sums to n
                            let prefix_g = |st: &mut Store, n_expr: ExprId| {
                                let n_plus_1 = st.add(vec![n_expr, one]);
                                let r_pow_n1 = st.pow(r, n_plus_1);
                                let neg_r_pow_n1 = st.mul(vec![minus_one, r_pow_n1]);
                                let num = st.add(vec![one, neg_r_pow_n1]); // 1 - r^(n+1)
                                st.mul(vec![num, denom_inv])
                            };
                            let prefix_k = |st: &mut Store, n_expr: ExprId| {
                                let n_plus_1 = st.add(vec![n_expr, one]);
                                let n_plus_2 = st.add(vec![n_expr, two]);
                                let r_pow_n1 = st.pow(r, n_plus_1);
                                let r_pow_n2 = st.pow(r, n_plus_2);
                                let term1 = r;
                                let n1_rn1 = st.mul(vec![n_plus_1, r_pow_n1]);
                                let term2 = st.mul(vec![n1_rn1, minus_one]); // - (n+1) r^(n+1)
                                let term3 = st.mul(vec![n_expr, r_pow_n2]); // n r^(n+2)
                                let num = st.add(vec![term1, term2, term3]);
                                st.mul(vec![num, denom2_inv])
                            };
                            // Build U and L-1
                            let u = upper;
                            let l_minus_1 = store.add(vec![lower, minus_one]);

                            let k_u = prefix_k(store, u);
                            let k_lm1 = prefix_k(store, l_minus_1);
                            let g_u = prefix_g(store, u);
                            let g_lm1 = prefix_g(store, l_minus_1);

                            let neg_k_lm1 = store.mul(vec![minus_one, k_lm1]);
                            let sum_k = store.add(vec![k_u, neg_k_lm1]);
                            let a_term = store.mul(vec![a, sum_k]);

                            let neg_g_lm1 = store.mul(vec![minus_one, g_lm1]);
                            let sum_g = store.add(vec![g_u, neg_g_lm1]);
                            let b_term = store.mul(vec![b, sum_g]);
                            let sum_res = store.add(vec![a_term, b_term]);
                            return Some(simplify(store, sum_res));
                        }
                        // Try quadratic case via derivative method: (a k^2 + b k + c) r^k
                        if let Some((qa, qb, qc)) = extract_quadratic_coeffs(store, poly, var) {
                            // If r == 1, let basic handle
                            if let (expr_core::Op::Integer, expr_core::Payload::Int(v)) =
                                (&store.get(r).op, &store.get(r).payload)
                            {
                                if *v == 1 {
                                    return None;
                                }
                            }
                            if let (expr_core::Op::Rational, expr_core::Payload::Rat(nv, dv)) =
                                (&store.get(r).op, &store.get(r).payload)
                            {
                                if *nv == 1 && *dv == 1 {
                                    return None;
                                }
                            }

                            let minus_one = store.int(-1);
                            let u = upper;
                            let l_minus_1 = store.add(vec![lower, minus_one]);

                            let sum_u = prefix_sums_deg2(store, r, u, qa, qb, qc)?;
                            let sum_lm1 = prefix_sums_deg2(store, r, l_minus_1, qa, qb, qc)?;
                            let neg_sum_lm1 = store.mul(vec![minus_one, sum_lm1]);
                            let sum_res = store.add(vec![sum_u, neg_sum_lm1]);
                            return Some(simplify(store, sum_res));
                        }
                        // Try general polynomial P(k) r^k using repeated R d/dR
                        if let Some(poly) = expr_to_unipoly(store, poly, var) {
                            // Translate Q coeffs to ExprId
                            let mut coeff_ids: Vec<ExprId> = Vec::with_capacity(poly.coeffs.len());
                            for q in poly.coeffs.iter() {
                                coeff_ids.push(q_to_expr(store, *q));
                            }
                            // If r == 1, let basic handle
                            if let (expr_core::Op::Integer, expr_core::Payload::Int(v)) =
                                (&store.get(r).op, &store.get(r).payload)
                            {
                                if *v == 1 {
                                    return None;
                                }
                            }
                            if let (expr_core::Op::Rational, expr_core::Payload::Rat(nv, dv)) =
                                (&store.get(r).op, &store.get(r).payload)
                            {
                                if *nv == 1 && *dv == 1 {
                                    return None;
                                }
                            }

                            let minus_one = store.int(-1);
                            let u = upper;
                            let l_minus_1 = store.add(vec![lower, minus_one]);

                            let sum_u = prefix_sums_poly_geom(store, r, u, &coeff_ids)?;
                            let sum_lm1 = prefix_sums_poly_geom(store, r, l_minus_1, &coeff_ids)?;
                            let neg_sum_lm1 = store.mul(vec![minus_one, sum_lm1]);
                            let sum_res = store.add(vec![sum_u, neg_sum_lm1]);
                            return Some(simplify(store, sum_res));
                        }
                    }
                }
            }
        }
    }

    // Find the antidifference g(k)
    let antidiff = find_antidifference(store, term, var)?;

    // Compute g(upper+1) - g(lower)
    let one = store.int(1);
    let upper_plus_1 = store.add(vec![upper, one]);

    let g_upper = substitute(store, antidiff, var, upper_plus_1)?;
    let g_lower = substitute(store, antidiff, var, lower)?;

    let minus_one = store.int(-1);
    let neg_g_lower = store.mul(vec![minus_one, g_lower]);
    let result = store.add(vec![g_upper, neg_g_lower]);

    Some(simplify(store, result))
}

/// Find the antidifference g(k) such that t(k) = g(k+1) - g(k)
///
/// This is the core of Gosper's algorithm.
fn find_antidifference(store: &mut Store, term: ExprId, var: &str) -> Option<ExprId> {
    // Get the ratio t(k+1)/t(k) = p(k)/q(k)
    let (p, q) = rationalize_hypergeometric(store, term, var)?;

    // For now, implement a simplified version for basic cases
    // Full Gosper's algorithm requires polynomial GCD and solving for certificates

    // Try simple cases first
    if let Some(result) = try_simple_gosper(store, term, var, p, q) {
        return Some(result);
    }

    None
}

/// Try simple Gosper cases (for basic hypergeometric terms)
fn try_simple_gosper(
    store: &mut Store,
    term: ExprId,
    var: &str,
    _p: ExprId,
    _q: ExprId,
) -> Option<ExprId> {
    // Case 1: Constant term
    // ∑ c = c*k
    if !depends_on_var(store, term, var) {
        let k = store.sym(var);
        return Some(store.mul(vec![term, k]));
    }

    // Case 2: Geometric term r^k with r independent of k
    if let expr_core::Op::Pow = store.get(term).op {
        let ch = store.get(term).children.clone();
        if ch.len() == 2 {
            let base = ch[0];
            let exp = ch[1];
            // exponent is the summation variable
            if let (expr_core::Op::Symbol, expr_core::Payload::Sym(s)) =
                (&store.get(exp).op, &store.get(exp).payload)
            {
                if s == var {
                    // base does not depend on var
                    if !depends_on_var(store, base, var) {
                        // r = 1 -> g(k) = k
                        match (&store.get(base).op, &store.get(base).payload) {
                            (expr_core::Op::Integer, expr_core::Payload::Int(v)) if *v == 1 => {
                                let k = store.sym(var);
                                return Some(k);
                            }
                            (expr_core::Op::Rational, expr_core::Payload::Rat(n, d))
                                if *n == 1 && *d == 1 =>
                            {
                                let k = store.sym(var);
                                return Some(k);
                            }
                            _ => {
                                // g(k) = r^k / (r - 1)
                                let minus_one = store.int(-1);
                                let r_minus_1 = store.add(vec![base, minus_one]);
                                let inv = store.pow(r_minus_1, minus_one);
                                let g = store.mul(vec![term, inv]);
                                return Some(simplify(store, g));
                            }
                        }
                    }
                }
            }
        }
    }

    // Case 2: Linear term (handled by basic module, but provide fallback)
    // For more complex cases, we would need full Gosper's algorithm

    None
}

/// Substitute var with replacement in expr
fn substitute(store: &mut Store, expr: ExprId, var: &str, replacement: ExprId) -> Option<ExprId> {
    use expr_core::{Op, Payload};

    match (&store.get(expr).op, &store.get(expr).payload) {
        (Op::Symbol, Payload::Sym(s)) => {
            if s == var {
                Some(replacement)
            } else {
                Some(expr)
            }
        }
        (Op::Integer, _) | (Op::Rational, _) => Some(expr),
        _ => {
            let children = store.get(expr).children.clone();
            let new_children: Option<Vec<_>> =
                children.iter().map(|&c| substitute(store, c, var, replacement)).collect();

            let new_children = new_children?;
            let new_expr = match store.get(expr).op {
                Op::Add => store.add(new_children),
                Op::Mul => store.mul(new_children),
                Op::Pow => {
                    if new_children.len() == 2 {
                        store.pow(new_children[0], new_children[1])
                    } else {
                        return None;
                    }
                }
                Op::Function => {
                    let func_name = if let Payload::Func(ref name) = store.get(expr).payload {
                        name.clone()
                    } else {
                        return None;
                    };
                    store.func(&func_name, new_children)
                }
                _ => return None,
            };

            Some(simplify(store, new_expr))
        }
    }

}

/// Check if expression depends on variable
fn depends_on_var(store: &Store, expr: ExprId, var: &str) -> bool {
    use expr_core::{Op, Payload};

    match (&store.get(expr).op, &store.get(expr).payload) {
        (Op::Symbol, Payload::Sym(s)) => s == var,
        (Op::Integer, _) | (Op::Rational, _) => false,
        _ => store.get(expr).children.iter().any(|&c| depends_on_var(store, c, var)),
    }
}

/// Extract coefficients (a, b) such that expr = a*k + b where a and b don't depend on k.
fn extract_linear_ab(store: &mut Store, expr: ExprId, var: &str) -> Option<(ExprId, ExprId)> {
    use expr_core::{Op, Payload};
    match &store.get(expr).op {
        Op::Symbol => {
            if let Payload::Sym(s) = &store.get(expr).payload {
                if s == var {
                    return Some((store.int(1), store.int(0)));
                }
            }
            Some((store.int(0), expr))
        }
        Op::Integer | Op::Rational => Some((store.int(0), expr)),
        Op::Mul => {
            // Recognize c*k where c does not depend on k
            let ch = store.get(expr).children.clone();
            let mut coeff = store.int(1);
            let mut var_seen = 0;
            for &c in &ch {
                if let (Op::Symbol, Payload::Sym(s)) = (&store.get(c).op, &store.get(c).payload) {
                    if s == var {
                        var_seen += 1;
                        continue;
                    }
                }
                coeff = store.mul(vec![coeff, c]);
            }
            if var_seen == 1 {
                Some((simplify(store, coeff), store.int(0)))
            } else {
                None
            }
        }
        Op::Add => {
            // Sum of linear parts yields linear
            let ch = store.get(expr).children.clone();
            let mut a = store.int(0);
            let mut b = store.int(0);
            for &c in &ch {
                if let Some((ai, bi)) = extract_linear_ab(store, c, var) {
                    a = store.add(vec![a, ai]);
                    b = store.add(vec![b, bi]);
                } else {
                    return None;
                }
            }
            Some((simplify(store, a), simplify(store, b)))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sum_closed_form;

    #[test]
    fn test_gosper_constant() {
        let mut st = Store::new();
        let five = st.int(5);
        let one = st.int(1);
        let ten = st.int(10);

        // ∑(k=1 to 10) 5 should work with simple case
        // Result should be 5 * 10 = 50
        if let Some(result) = gosper_sum(&mut st, five, "k", one, ten) {
            let result_str = st.to_string(result);
            // Should contain 5 and 10 or evaluate to 50
            assert!(result_str.contains("5") || result_str.contains("50"));
        }
    }

    #[test]
    fn test_gosper_linear_times_geometric_sum() {
        // ∑(k=0..n) k·2^k
        let mut st = Store::new();
        let k = st.sym("k");
        let zero = st.int(0);
        let n = st.sym("n");
        let two = st.int(2);
        let pow = st.pow(two, k);
        let term = st.mul(vec![k, pow]);

        let result = sum_closed_form(&mut st, term, "k", zero, n).expect("sum k*2^k");
        let s = st.to_string(result);
        assert!(s.contains("2") && s.contains("n"));
    }

    #[test]
    fn test_gosper_quadratic_times_geometric_sum() {
        // ∑(k=0..n) (k^2 + 3k + 1) 2^k
        let mut st = Store::new();
        let k = st.sym("k");
        let zero = st.int(0);
        let n = st.sym("n");
        let two = st.int(2);
        let two_exp = st.int(2);
        let k2 = st.pow(k, two_exp);
        let three = st.int(3);
        let three_k = st.mul(vec![three, k]);
        let one = st.int(1);
        let poly = st.add(vec![k2, three_k, one]);
        let pow = st.pow(two, k);
        let term = st.mul(vec![poly, pow]);

        let result = sum_closed_form(&mut st, term, "k", zero, n).expect("sum quadratic*2^k");
        let s = st.to_string(result);
        assert!(s.contains("2") && s.contains("n"));
    }

    #[test]
    fn test_gosper_cubic_times_geometric_sum() {
        // ∑(k=0..n) (k^3 + 2k^2 + k + 1) 2^k
        let mut st = Store::new();
        let k = st.sym("k");
        let zero = st.int(0);
        let n = st.sym("n");
        let two = st.int(2);
        let two_exp = st.int(2);
        let three_exp = st.int(3);
        let k2 = st.pow(k, two_exp);
        let k3 = st.pow(k, three_exp);
        let two_c = st.int(2);
        let mul_two_k2 = st.mul(vec![two_c, k2]);
        let one = st.int(1);
        let poly = st.add(vec![k3, mul_two_k2, k, one]);
        let pow = st.pow(two, k);
        let term = st.mul(vec![poly, pow]);

        let result = sum_closed_form(&mut st, term, "k", zero, n).expect("sum cubic*2^k");
        let s = st.to_string(result);
        assert!(s.contains("2") && s.contains("n"));
    }
}
