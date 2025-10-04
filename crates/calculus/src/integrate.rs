//! Integration rules (v1, conservative).

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
            // Prefer rational integration via partial fractions if applicable
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
                let num_expr = if num_factors.is_empty() {
                    None
                } else {
                    Some(st.mul(num_factors))
                };
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
        let c_neg = if neg_a.1 == 1 {
            st.int(neg_a.0)
        } else {
            st.rat(neg_a.0, neg_a.1)
        };
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
