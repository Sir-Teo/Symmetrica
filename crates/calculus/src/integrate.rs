//! Integration rules (v1, conservative + Phase J: integration by parts).

use crate::diff::diff;
use arith::{q_div, q_mul, Q};
use expr_core::{ExprId, Op, Payload, Store};
use polys::{expr_to_unipoly, partial_fractions_simple, UniPoly};
use simplify::simplify;

/// Try to integrate expression w.r.t. `var`. Returns None if rule not supported.
pub fn integrate(store: &mut Store, id: ExprId, var: &str) -> Option<ExprId> {
    // helper: does expr depend on var?
    fn depends_on_var(st: &Store, id: ExprId, var: &str) -> bool {
        match (&st.get(id).op, &st.get(id).payload) {
            (Op::Symbol, Payload::Sym(s)) => s == var,
            (Op::Integer, _) | (Op::Rational, _) => false,
            _ => st.get(id).children.iter().any(|&c| depends_on_var(st, c, var)),
        }
    }
    // helper: extract numeric coefficient and rest from a product
    fn split_coeff_mul(st: &mut Store, id: ExprId) -> ((i64, i64), ExprId) {
        match (&st.get(id).op, &st.get(id).payload) {
            (Op::Integer, Payload::Int(k)) => ((*k, 1), st.int(1)),
            (Op::Rational, Payload::Rat(n, d)) => ((*n, *d), st.int(1)),
            (Op::Mul, _) => {
                let mut coeff = (1i64, 1i64);
                let mut rest: Vec<ExprId> = Vec::new();
                let children = st.get(id).children.clone();
                for f in children {
                    match (&st.get(f).op, &st.get(f).payload) {
                        (Op::Integer, Payload::Int(k)) => {
                            coeff = q_mul(coeff, (*k, 1));
                        }
                        (Op::Rational, Payload::Rat(n, d)) => {
                            coeff = q_mul(coeff, (*n, *d));
                        }
                        _ => rest.push(f),
                    }
                }
                let rest_id = if rest.is_empty() { st.int(1) } else { st.mul(rest) };
                (coeff, rest_id)
            }
            _ => ((1, 1), id),
        }
    }
    // helper: build coeff * expr
    fn with_coeff(st: &mut Store, coeff: (i64, i64), expr: ExprId) -> ExprId {
        if coeff == (1, 1) {
            return expr;
        }
        let c = st.rat(coeff.0, coeff.1);
        let prod = st.mul(vec![c, expr]);
        simplify(st, prod)
    }

    match store.get(id).op {
        Op::Integer => {
            if let Payload::Int(k) = store.get(id).payload {
                let x = store.sym(var);
                let ck = store.int(k);
                Some(store.mul(vec![ck, x]))
            } else {
                None
            }
        }
        Op::Rational => {
            if let Payload::Rat(n, d) = store.get(id).payload {
                let x = store.sym(var);
                let c = store.rat(n, d);
                Some(store.mul(vec![c, x]))
            } else {
                None
            }
        }
        Op::Symbol => match &store.get(id).payload {
            Payload::Sym(s) if s == var => {
                // ∫ x dx = x^2/2
                let two = store.int(2);
                let x = store.sym(var);
                let x2 = store.pow(x, two);
                let half = store.rat(1, 2);
                Some(store.mul(vec![half, x2]))
            }
            _ => {
                // treat as constant symbol c: ∫ c dx = c*x
                let x = store.sym(var);
                Some(store.mul(vec![id, x]))
            }
        },
        Op::Add => {
            let mut terms: Vec<ExprId> = Vec::new();
            for &t in &store.get(id).children.clone() {
                let it = integrate(store, t, var)?;
                terms.push(it);
            }
            let sum = store.add(terms);
            Some(simplify(store, sum))
        }
        Op::Mul => {
            // Try integration by parts for product patterns
            if let Some(res) = try_integration_by_parts(store, id, var) {
                return Some(res);
            }
            // Try rational integration via partial fractions if applicable
            if let Some(res) = integrate_rational(store, id, var) {
                return Some(res);
            }
            // factor out numeric coefficient
            let (coeff, rest) = split_coeff_mul(store, id);
            // f'/f pattern: look for a factor u^{-1} and check remaining equals u' up to numeric factor
            if store.get(rest).op == Op::Mul {
                let factors = store.get(rest).children.clone();
                // iterate all positions to find u^{-1}
                for (idx, &f) in factors.iter().enumerate() {
                    if store.get(f).op == Op::Pow {
                        let u_node = store.get(f);
                        if u_node.children.len() == 2 {
                            let u = u_node.children[0];
                            let e = u_node.children[1];
                            if matches!(
                                (&store.get(e).op, &store.get(e).payload),
                                (Op::Integer, Payload::Int(-1))
                            ) {
                                // build product of remaining factors
                                let mut others: Vec<ExprId> =
                                    Vec::with_capacity(factors.len().saturating_sub(1));
                                for (j, &g) in factors.iter().enumerate() {
                                    if j != idx {
                                        others.push(g);
                                    }
                                }
                                let others_id = if others.is_empty() {
                                    store.int(1)
                                } else {
                                    store.mul(others)
                                };
                                // compare to u' up to numeric coefficient
                                let du = diff(store, u, var);
                                let (coeff_o, rest_o) = split_coeff_mul(store, others_id);
                                let (coeff_d, rest_d) = split_coeff_mul(store, du);
                                if rest_o == rest_d {
                                    let scale = q_div(coeff_o, coeff_d);
                                    let total = q_mul(coeff, scale);
                                    let ln_u = store.func("ln", vec![u]);
                                    return Some(with_coeff(store, total, ln_u));
                                }
                            }
                        }
                    }
                }
            }
            // constant times integrable function, only if we truly factored something out
            if coeff != (1, 1) {
                let ir = integrate(store, rest, var)?;
                Some(with_coeff(store, coeff, ir))
            } else if rest != id {
                let ir = integrate(store, rest, var)?;
                Some(ir)
            } else {
                // Try rational integration via partial fractions
                integrate_rational(store, id, var)
            }
        }
        Op::Pow => {
            // ∫ x^n dx rule
            let base = store.get(id).children[0];
            let exp = store.get(id).children[1];
            if let (Op::Symbol, Payload::Sym(s)) = (&store.get(base).op, &store.get(base).payload) {
                if s == var {
                    let k_value = match (&store.get(exp).op, &store.get(exp).payload) {
                        (Op::Integer, Payload::Int(k)) => Some(*k),
                        _ => None,
                    };
                    if let Some(k) = k_value {
                        if k == -1 {
                            // ∫ x^-1 dx = ln x
                            let ln = store.func("ln", vec![base]);
                            return Some(ln);
                        } else {
                            // x^(k+1)/(k+1)
                            let k1 = store.int(k + 1);
                            let xkp1 = store.pow(base, k1);
                            let coeff = q_div((1, 1), (k + 1, 1));
                            return Some(with_coeff(store, coeff, xkp1));
                        }
                    }
                }
            }
            // Try ∫ 1/den(x) dx via partial fractions if den splits
            if let Some(res) = integrate_rational(store, id, var) {
                return Some(res);
            }
            None
        }
        Op::Function => {
            // exp(ax+b), sin(ax+b), cos(ax+b)
            let (fname, u) = {
                let n = store.get(id);
                let name = match &n.payload {
                    Payload::Func(s) => s.clone(),
                    _ => return None,
                };
                if n.children.len() != 1 {
                    return None;
                }
                (name, n.children[0])
            };
            // check du is constant
            let du = diff(store, u, var);
            let a = match (&store.get(du).op, &store.get(du).payload) {
                (Op::Integer, Payload::Int(k)) => (*k, 1),
                (Op::Rational, Payload::Rat(n, d)) => (*n, *d),
                _ => {
                    // if independent of var entirely, treat whole function as constant
                    if !depends_on_var(store, id, var) {
                        let x = store.sym(var);
                        return Some(store.mul(vec![id, x]));
                    }
                    return None;
                }
            };
            if a == (0, 1) {
                return None;
            }
            let inv_a = q_div((1, 1), a);
            let res = match fname.as_str() {
                "exp" => id,
                "sin" => {
                    let c = store.func("cos", vec![u]);
                    let neg1 = store.int(-1);
                    store.mul(vec![neg1, c])
                }
                "cos" => store.func("sin", vec![u]),
                _ => return None,
            };
            Some(with_coeff(store, inv_a, res))
        }
    }
}

/// Integration by parts: ∫ u dv = uv - ∫ v du
/// Uses LIATE heuristic (Logarithmic, Inverse trig, Algebraic, Trigonometric, Exponential)
/// to choose u and dv from a product.
fn try_integration_by_parts(st: &mut Store, id: ExprId, var: &str) -> Option<ExprId> {
    if st.get(id).op != Op::Mul {
        return None;
    }

    let children = st.get(id).children.clone();
    if children.len() != 2 {
        return None; // Only handle simple two-factor products
    }

    // Helper: does expr depend on var?
    fn depends_on_var(st: &Store, id: ExprId, var: &str) -> bool {
        match (&st.get(id).op, &st.get(id).payload) {
            (Op::Symbol, Payload::Sym(s)) => s == var,
            (Op::Integer, _) | (Op::Rational, _) => false,
            _ => st.get(id).children.iter().any(|&c| depends_on_var(st, c, var)),
        }
    }

    // Helper: LIATE priority (lower is higher priority for u)
    fn liate_priority(st: &Store, id: ExprId, var: &str) -> i32 {
        if !depends_on_var(st, id, var) {
            return 100; // constants go in dv
        }
        match &st.get(id).op {
            Op::Function => {
                if let Payload::Func(name) = &st.get(id).payload {
                    match name.as_str() {
                        "ln" | "log" => 1,                   // Logarithmic (highest priority for u)
                        "arcsin" | "arccos" | "arctan" => 2, // Inverse trig
                        "sin" | "cos" | "tan" => 4,          // Trigonometric
                        "exp" => 5,                          // Exponential (lowest priority)
                        _ => 50,
                    }
                } else {
                    50
                }
            }
            Op::Pow => {
                // x^n is algebraic
                let base = st.get(id).children[0];
                if matches!((&st.get(base).op, &st.get(base).payload), (Op::Symbol, Payload::Sym(s)) if s == var)
                {
                    3 // Algebraic
                } else {
                    50
                }
            }
            Op::Symbol => {
                if let Payload::Sym(s) = &st.get(id).payload {
                    if s == var {
                        return 3; // x is algebraic
                    }
                }
                100
            }
            _ => 50,
        }
    }

    let f0 = children[0];
    let f1 = children[1];

    // Skip if either factor doesn't depend on var (will be handled by constant factor rule)
    if !depends_on_var(st, f0, var) || !depends_on_var(st, f1, var) {
        return None;
    }

    let p0 = liate_priority(st, f0, var);
    let p1 = liate_priority(st, f1, var);

    // Choose u (lower priority) and dv (higher priority)
    let (u, dv) = if p0 < p1 { (f0, f1) } else { (f1, f0) };

    // Compute du and v
    let du = diff(st, u, var);
    let v = integrate(st, dv, var)?;

    // ∫ u dv = uv - ∫ v du
    let uv = st.mul(vec![u, v]);
    let v_du = st.mul(vec![v, du]);

    // Try to integrate v*du
    let integral_v_du = integrate(st, v_du, var)?;

    let neg1 = st.int(-1);
    let minus_integral = st.mul(vec![neg1, integral_v_du]);
    let result = st.add(vec![uv, minus_integral]);

    Some(simplify(st, result))
}

// Attempt to interpret `id` as a rational function num/den in variable `var` and integrate
// using partial fractions for denominators that split into distinct linear factors over Q.
fn integrate_rational(st: &mut Store, id: ExprId, var: &str) -> Option<ExprId> {
    // Extract numerator and denominator polynomials if expression is of the form
    //   Mul(..., Pow(den, -1)) or just Pow(den, -1) or a plain rational polynomial (den=1)
    fn decompose(st: &mut Store, id: ExprId, var: &str) -> Option<(UniPoly, UniPoly)> {
        match st.get(id).op {
            Op::Pow => {
                let n = st.get(id);
                let b = n.children[0];
                let e = n.children[1];
                if matches!((&st.get(e).op, &st.get(e).payload), (Op::Integer, Payload::Int(-1))) {
                    let den = expr_to_unipoly(st, b, var)?;
                    let num = UniPoly::new(var, vec![Q(1, 1)]);
                    return Some((num, den));
                }
                None
            }
            Op::Mul => {
                let children = st.get(id).children.clone();
                let mut den_opt: Option<ExprId> = None;
                let mut num_factors: Vec<ExprId> = Vec::new();
                for &c in &children {
                    if st.get(c).op == Op::Pow {
                        let n = st.get(c);
                        if n.children.len() == 2 {
                            let e = n.children[1];
                            if matches!(
                                (&st.get(e).op, &st.get(e).payload),
                                (Op::Integer, Payload::Int(-1))
                            ) {
                                if den_opt.is_some() {
                                    return None;
                                } // only support single reciprocal
                                den_opt = Some(n.children[0]);
                                continue;
                            }
                        }
                    }
                    num_factors.push(c);
                }
                let den_e = den_opt?;
                let num_expr =
                    if num_factors.is_empty() { None } else { Some(st.mul(num_factors)) };
                let num_poly = match num_expr {
                    Some(ne) => expr_to_unipoly(st, ne, var)?,
                    None => UniPoly::new(var, vec![Q(1, 1)]),
                };
                let den_poly = expr_to_unipoly(st, den_e, var)?;
                Some((num_poly, den_poly))
            }
            _ => None,
        }
    }

    let (num, den) = decompose(st, id, var)?;
    // Proper handling using partial fractions (includes quotient if improper)
    let (q, terms) = partial_fractions_simple(&num, &den)?;

    // Integrate polynomial quotient q(x) term-wise to expression
    fn poly_integral_expr(st: &mut Store, p: &UniPoly) -> ExprId {
        if p.is_zero() {
            return st.int(0);
        }
        let x = st.sym(&p.var);
        let mut terms_expr: Vec<ExprId> = Vec::new();
        for (k, &c) in p.coeffs.iter().enumerate() {
            if c.is_zero() {
                continue;
            }
            // ∫ c x^k dx = c * x^{k+1}/(k+1)
            let k1 = (k as i64) + 1;
            let coeff = q_div((c.0, c.1), (k1, 1));
            let k1_expr = st.int(k1);
            let pow = st.pow(x, k1_expr);
            let term = if coeff.1 == 1 {
                let c_int = st.int(coeff.0);
                st.mul(vec![c_int, pow])
            } else {
                let c_rat = st.rat(coeff.0, coeff.1);
                st.mul(vec![c_rat, pow])
            };
            terms_expr.push(term);
        }
        st.add(terms_expr)
    }

    let mut parts: Vec<ExprId> = Vec::new();
    let poly_int = poly_integral_expr(st, &q);
    if !matches!((&st.get(poly_int).op, &st.get(poly_int).payload), (Op::Integer, Payload::Int(0)))
    {
        parts.push(poly_int);
    }

    // ∫ A/(x - a) dx = A * ln(x - a)
    let x = st.sym(var);
    for (residue, root) in terms {
        let neg_a = (-root.0, root.1);
        let c_neg = if neg_a.1 == 1 { st.int(neg_a.0) } else { st.rat(neg_a.0, neg_a.1) };
        let x_minus_a = st.add(vec![x, c_neg]);
        let ln = st.func("ln", vec![x_minus_a]);
        let term = if residue == Q(1, 1) {
            ln
        } else if residue == Q(-1, 1) {
            let m1 = st.int(-1);
            st.mul(vec![m1, ln])
        } else if residue.1 == 1 {
            let c_res = st.int(residue.0);
            st.mul(vec![c_res, ln])
        } else {
            let c_res = st.rat(residue.0, residue.1);
            st.mul(vec![c_res, ln])
        };
        parts.push(term);
    }

    if parts.is_empty() {
        return None;
    }
    let sum = st.add(parts);
    Some(simplify(st, sum))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integrate_constant_symbol() {
        let mut st = Store::new();
        let c = st.sym("c");
        let res = integrate(&mut st, c, "x").expect("const");
        // ∫ c dx = c * x
        let x = st.sym("x");
        let expected = st.mul(vec![c, x]);
        assert_eq!(st.to_string(res), st.to_string(expected));
    }

    #[test]
    fn integrate_add_rule() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let expr = st.add(vec![x, two]);
        let res = integrate(&mut st, expr, "x").expect("sum");
        assert!(st.to_string(res).contains("x"));
    }

    #[test]
    fn integrate_mul_constant_factor() {
        let mut st = Store::new();
        let x = st.sym("x");
        let three = st.int(3);
        let expr = st.mul(vec![three, x]);
        let res = integrate(&mut st, expr, "x").expect("cx");
        // ∫ 3x dx = 3 * x^2/2
        assert!(st.to_string(res).contains("3"));
        assert!(st.to_string(res).contains("x"));
    }

    #[test]
    fn integrate_rational_constant() {
        let mut st = Store::new();
        let half = st.rat(1, 2);
        let x = st.sym("x");
        let expected = st.mul(vec![half, x]);
        let res = integrate(&mut st, half, "x").expect("rat");
        assert_eq!(st.to_string(res), st.to_string(expected));
    }

    #[test]
    fn integrate_fails_on_unsupported() {
        let mut st = Store::new();
        let x = st.sym("x");
        // ∫ sin(x^2) dx not supported
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let sinx2 = st.func("sin", vec![x2]);
        let res = integrate(&mut st, sinx2, "x");
        assert!(res.is_none());
    }

    #[test]
    fn integrate_integer_const() {
        let mut st = Store::new();
        let five = st.int(5);
        let res = integrate(&mut st, five, "x").expect("const");
        let res_str = st.to_string(res);
        assert!(res_str.contains("5"));
        assert!(res_str.contains("x"));
    }

    #[test]
    fn integrate_power_negative_exponent() {
        let mut st = Store::new();
        let x = st.sym("x");
        let m2 = st.int(-2);
        let xm2 = st.pow(x, m2);
        let res = integrate(&mut st, xm2, "x").expect("x^-2");
        // ∫ x^-2 dx = x^-1 / -1 = -x^-1
        assert!(st.to_string(res).contains("x"));
    }

    #[test]
    fn integrate_exp_constant_derivative() {
        let mut st = Store::new();
        let x = st.sym("x");
        let expx = st.func("exp", vec![x]);
        let res = integrate(&mut st, expx, "x").expect("exp(x)");
        assert!(st.to_string(res).contains("exp"));
    }

    #[test]
    fn integrate_rational_via_pf_fails_on_complex() {
        let mut st = Store::new();
        let x = st.sym("x");
        // 1/(x^2 + 1) has no rational roots
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let one = st.int(1);
        let den = st.add(vec![x2, one]);
        let m1 = st.int(-1);
        let expr = st.pow(den, m1);
        let res = integrate(&mut st, expr, "x");
        assert!(res.is_none());
    }

    // ========== Integration by Parts Tests (Phase J) ==========

    #[test]
    fn integrate_by_parts_x_sin_x() {
        let mut st = Store::new();
        let x = st.sym("x");
        let sinx = st.func("sin", vec![x]);
        let expr = st.mul(vec![x, sinx]);
        let res = integrate(&mut st, expr, "x").expect("x*sin(x)");

        // ∫ x sin(x) dx = -x cos(x) + sin(x)
        // Verify by differentiation
        let deriv = diff(&mut st, res, "x");
        let simplified = simplify(&mut st, deriv);
        let original_simplified = simplify(&mut st, expr);

        // Check that derivative equals original
        assert_eq!(st.get(simplified).digest, st.get(original_simplified).digest);
    }

    #[test]
    fn integrate_by_parts_x_exp_x() {
        let mut st = Store::new();
        let x = st.sym("x");
        let expx = st.func("exp", vec![x]);
        let expr = st.mul(vec![x, expx]);
        let res = integrate(&mut st, expr, "x").expect("x*exp(x)");

        // ∫ x exp(x) dx = x exp(x) - exp(x)
        // Verify by differentiation
        let deriv = diff(&mut st, res, "x");
        let simplified = simplify(&mut st, deriv);
        let original_simplified = simplify(&mut st, expr);

        assert_eq!(st.get(simplified).digest, st.get(original_simplified).digest);
    }

    #[test]
    fn integrate_by_parts_x_cos_x() {
        let mut st = Store::new();
        let x = st.sym("x");
        let cosx = st.func("cos", vec![x]);
        let expr = st.mul(vec![x, cosx]);
        let res = integrate(&mut st, expr, "x").expect("x*cos(x)");

        // ∫ x cos(x) dx = x sin(x) + cos(x)
        // Verify by differentiation
        let deriv = diff(&mut st, res, "x");
        let simplified = simplify(&mut st, deriv);
        let original_simplified = simplify(&mut st, expr);

        assert_eq!(st.get(simplified).digest, st.get(original_simplified).digest);
    }

    #[test]
    fn integrate_by_parts_x2_sin_x() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let sinx = st.func("sin", vec![x]);
        let expr = st.mul(vec![x2, sinx]);
        let res = integrate(&mut st, expr, "x").expect("x^2*sin(x)");

        // ∫ x^2 sin(x) dx should work with repeated integration by parts
        // Verify by differentiation
        let deriv = diff(&mut st, res, "x");
        let simplified = simplify(&mut st, deriv);
        let original_simplified = simplify(&mut st, expr);

        assert_eq!(st.get(simplified).digest, st.get(original_simplified).digest);
    }

    #[test]
    fn integrate_by_parts_x2_exp_x() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let expx = st.func("exp", vec![x]);
        let expr = st.mul(vec![x2, expx]);
        let res = integrate(&mut st, expr, "x").expect("x^2*exp(x)");

        // ∫ x^2 exp(x) dx should work with repeated integration by parts
        // Verify by differentiation
        let deriv = diff(&mut st, res, "x");
        let simplified = simplify(&mut st, deriv);
        let original_simplified = simplify(&mut st, expr);

        assert_eq!(st.get(simplified).digest, st.get(original_simplified).digest);
    }

    #[test]
    fn integrate_by_parts_ln_x_times_x() {
        let mut st = Store::new();
        let x = st.sym("x");
        let lnx = st.func("ln", vec![x]);
        let expr = st.mul(vec![x, lnx]);
        let res = integrate(&mut st, expr, "x").expect("x*ln(x)");

        // ∫ x ln(x) dx = (x^2/2) ln(x) - x^2/4
        // Verify by differentiation (allowing for integration constant differences)
        let deriv = diff(&mut st, res, "x");
        let simplified = simplify(&mut st, deriv);
        let original_simplified = simplify(&mut st, expr);

        assert_eq!(st.get(simplified).digest, st.get(original_simplified).digest);
    }

    // Note: ∫ exp(x) sin(x) dx is not implemented as it requires solving a system
    // (applying integration by parts twice leads to a linear equation).
    // This would cause infinite recursion with the current implementation.

    #[test]
    fn integrate_by_parts_x3_cos_x() {
        let mut st = Store::new();
        let x = st.sym("x");
        let three = st.int(3);
        let x3 = st.pow(x, three);
        let cosx = st.func("cos", vec![x]);
        let expr = st.mul(vec![x3, cosx]);
        let res = integrate(&mut st, expr, "x").expect("x^3*cos(x)");

        // Verify by differentiation
        let deriv = diff(&mut st, res, "x");
        let simplified = simplify(&mut st, deriv);
        let original_simplified = simplify(&mut st, expr);

        assert_eq!(st.get(simplified).digest, st.get(original_simplified).digest);
    }
}
