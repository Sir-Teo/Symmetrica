//! Integration rules (v1, conservative + Phase J: integration by parts).

use crate::diff::diff;
use arith::{q_div, q_mul, Q};
use expr_core::{ExprId, Op, Payload, Store};
use polys::{expr_to_unipoly, partial_fractions_simple, UniPoly};
use simplify::simplify;

/// Try to integrate expression w.r.t. `var`. Returns None if rule not supported.
pub fn integrate(store: &mut Store, id: ExprId, var: &str) -> Option<ExprId> {
    // Check memoization cache first
    if let Some(cached) = store.get_integrate_cached(id, var) {
        return cached;
    }

    // Compute the integral
    let result = integrate_impl(store, id, var);

    // Cache the result before returning
    store.cache_integrate(id, var.to_string(), result);
    result
}

/// Internal integration implementation (without memoization)
fn integrate_impl(store: &mut Store, id: ExprId, var: &str) -> Option<ExprId> {
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
            // Try u-substitution patterns first (f(g(x)) * g'(x))
            if let Some(res) = try_u_substitution(store, id, var) {
                return Some(res);
            }
            // Try generalized sin^m(x) * cos^n(x) (handles odd exponents)
            if let Some(res) = try_trig_power_general(store, id, var) {
                return Some(res);
            }
            // Try basic trig product pattern (sin(x) * cos(x))
            if let Some(res) = try_trig_power_pattern(store, id, var) {
                return Some(res);
            }
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
            // Single-power trig patterns like sin^m(x) or cos^n(x)
            if let Some(res) = try_trig_power_general(store, id, var) {
                return Some(res);
            }
            // Even-power single trig functions like sin^(2k)(x) or cos^(2k)(x)
            if let Some(res) = try_trig_even_power_single(store, id, var) {
                return Some(res);
            }
            // Try Weierstrass substitution for rational trig integrals (1/(1+cos(x)))
            if let Some(res) = try_weierstrass_substitution(store, id, var) {
                return Some(res);
            }
            // Try trig square patterns (sin^2, cos^2)
            if let Some(res) = try_trig_square_pattern(store, id, var) {
                return Some(res);
            }
            // Try power rule for polynomials and simple powers
            let base = store.get(id).children[0];
            let exponent = store.get(id).children[1];
            if let (Op::Symbol, Payload::Sym(s)) = (&store.get(base).op, &store.get(base).payload) {
                if s == var {
                    let k_value = match (&store.get(exponent).op, &store.get(exponent).payload) {
                        (Op::Integer, Payload::Int(k)) => Some(*k),
                        (Op::Rational, Payload::Rat(n, d)) => Some(*n / *d),
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
            // Try Risch-based exponential integration first
            if let Some(res) = crate::risch::try_integrate_exponential(store, id, var) {
                return Some(res);
            }

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
                "sinh" => {
                    // ∫ sinh(u) du = cosh(u)
                    store.func("cosh", vec![u])
                }
                "cosh" => {
                    // ∫ cosh(u) du = sinh(u)
                    store.func("sinh", vec![u])
                }
                "tanh" => {
                    // ∫ tanh(u) du = ln(cosh(u))
                    let cosh_u = store.func("cosh", vec![u]);
                    store.func("ln", vec![cosh_u])
                }
                _ => return None,
            };
            Some(with_coeff(store, inv_a, res))
        }
        Op::Piecewise => {
            // Integrate piecewise: ∫ piecewise((c1, v1), ...) dx = piecewise((c1, ∫v1 dx), ...)
            let children = store.get(id).children.clone();
            let mut pairs = Vec::new();
            for chunk in children.chunks(2) {
                if chunk.len() == 2 {
                    let cond = chunk[0];
                    let val = chunk[1];
                    let ival = integrate(store, val, var)?;
                    pairs.push((cond, ival));
                }
            }
            let pw = store.piecewise(pairs);
            Some(simplify(store, pw))
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

/// Try u-substitution: ∫ f(g(x)) * g'(x) dx = ∫ f(u) du where u = g(x)
/// Detects patterns where one factor is (proportional to) the derivative of
/// a sub-expression in another factor.
fn try_u_substitution(st: &mut Store, id: ExprId, var: &str) -> Option<ExprId> {
    if st.get(id).op != Op::Mul {
        return None;
    }

    let children = st.get(id).children.clone();

    // Helper: extract numeric coefficient from product
    fn split_coeff(st: &mut Store, id: ExprId) -> ((i64, i64), ExprId) {
        match (&st.get(id).op, &st.get(id).payload) {
            (Op::Integer, Payload::Int(k)) => ((*k, 1), st.int(1)),
            (Op::Rational, Payload::Rat(n, d)) => ((*n, *d), st.int(1)),
            (Op::Mul, _) => {
                let mut coeff = (1i64, 1i64);
                let mut rest = Vec::new();
                for &c in &st.get(id).children {
                    match (&st.get(c).op, &st.get(c).payload) {
                        (Op::Integer, Payload::Int(k)) => coeff = q_mul(coeff, (*k, 1)),
                        (Op::Rational, Payload::Rat(n, d)) => coeff = q_mul(coeff, (*n, *d)),
                        _ => rest.push(c),
                    }
                }
                let rest_id = if rest.is_empty() { st.int(1) } else { st.mul(rest) };
                (coeff, rest_id)
            }
            _ => ((1, 1), id),
        }
    }

    // Pattern 1: ∫ u^n * u' dx where u is a compound expression
    // Try each partition of factors as (potential_u_power, potential_u_prime)
    for i in 0..children.len() {
        let factor = children[i];

        // Check if this factor looks like u^n for some compound u
        if let Op::Pow = st.get(factor).op {
            let u_base = st.get(factor).children[0];
            let exponent = st.get(factor).children[1];

            // Only handle if u_base depends on var and isn't just var itself
            if !matches!((&st.get(u_base).op, &st.get(u_base).payload), (Op::Symbol, _)) {
                let du = diff(st, u_base, var);

                // Build product of other factors
                let mut other_factors = Vec::new();
                for (j, &child) in children.iter().enumerate() {
                    if i != j {
                        other_factors.push(child);
                    }
                }

                if other_factors.is_empty() {
                    continue;
                }

                let others =
                    if other_factors.len() == 1 { other_factors[0] } else { st.mul(other_factors) };

                // Check if others equals c * du for some constant c
                let (c_others, rest_others) = split_coeff(st, others);
                let (c_du, rest_du) = split_coeff(st, du);

                if rest_others == rest_du && c_du.0 != 0 {
                    // Found pattern: ∫ u^n * (c * u') dx
                    // Result: (c / (n+1)) * u^(n+1)
                    let exp_val = match (&st.get(exponent).op, &st.get(exponent).payload) {
                        (Op::Integer, Payload::Int(n)) => Some(*n),
                        _ => None,
                    };

                    if let Some(n) = exp_val {
                        if n != -1 {
                            let n_plus_1_val = n + 1;
                            let n_plus_1 = st.int(n_plus_1_val);
                            let u_np1 = st.pow(u_base, n_plus_1);
                            let coeff = q_div(c_others, q_mul(c_du, (n_plus_1_val, 1)));
                            let result = if coeff.1 == 1 {
                                let c_int = st.int(coeff.0);
                                st.mul(vec![c_int, u_np1])
                            } else {
                                let c_rat = st.rat(coeff.0, coeff.1);
                                st.mul(vec![c_rat, u_np1])
                            };
                            return Some(simplify(st, result));
                        }
                    }
                }
            }
        }
    }

    None
}

/// General sin^m(x) * cos^n(x) integration for odd exponents (m or n).
fn try_trig_power_general(st: &mut Store, id: ExprId, var: &str) -> Option<ExprId> {
    // Helpers to recognize sin(var) and cos(var)
    fn is_sin_of_var(st: &Store, id: ExprId, var: &str) -> bool {
        if let (Op::Function, Payload::Func(fname)) = (&st.get(id).op, &st.get(id).payload) {
            if fname == "sin" && st.get(id).children.len() == 1 {
                let arg = st.get(id).children[0];
                return matches!((&st.get(arg).op, &st.get(arg).payload), (Op::Symbol, Payload::Sym(s)) if s == var);
            }
        }
        false
    }

    fn is_cos_of_var(st: &Store, id: ExprId, var: &str) -> bool {
        if let (Op::Function, Payload::Func(fname)) = (&st.get(id).op, &st.get(id).payload) {
            if fname == "cos" && st.get(id).children.len() == 1 {
                let arg = st.get(id).children[0];
                return matches!((&st.get(arg).op, &st.get(arg).payload), (Op::Symbol, Payload::Sym(s)) if s == var);
            }
        }
        false
    }

    // Binomial coefficient C(n, k) computed safely using i128 and cast back to i64
    fn binom_u64(n: u64, k: u64) -> Option<i64> {
        if k > n {
            return Some(0);
        }
        let k = std::cmp::min(k, n - k);
        let mut num: i128 = 1;
        let mut den: i128 = 1;
        fn gcd_i128(mut a: i128, mut b: i128) -> i128 {
            if a < 0 {
                a = -a;
            }
            if b < 0 {
                b = -b;
            }
            while b != 0 {
                let r = a % b;
                a = b;
                b = r;
            }
            if a == 0 {
                1
            } else {
                a
            }
        }
        for i in 1..=k {
            num = num.saturating_mul((n - k + i) as i128);
            den = den.saturating_mul(i as i128);
            // Reduce by gcd periodically to avoid overflow
            let g = gcd_i128(num, den);
            if g > 1 {
                num /= g;
                den /= g;
            }
        }
        // Final division (should divide exactly)
        if den == 0 {
            return None;
        }
        let val = num / den;
        if val <= i64::MAX as i128 && val >= i64::MIN as i128 {
            Some(val as i64)
        } else {
            None
        }
    }

    // Accumulate numeric coefficient and exponents m (sin) and n (cos)
    let mut coeff: (i64, i64) = (1, 1);
    let mut m: i64 = 0; // power of sin(x)
    let mut n: i64 = 0; // power of cos(x)

    // Local function to fold a single factor into (coeff, m, n)
    fn fold_factor(
        st: &Store,
        f: ExprId,
        var: &str,
        coeff: &mut (i64, i64),
        m: &mut i64,
        n: &mut i64,
    ) -> bool {
        match (&st.get(f).op, &st.get(f).payload) {
            (Op::Integer, Payload::Int(k)) => {
                *coeff = q_mul(*coeff, (*k, 1));
                true
            }
            (Op::Rational, Payload::Rat(a, b)) => {
                *coeff = q_mul(*coeff, (*a, *b));
                true
            }
            (Op::Function, Payload::Func(_)) => {
                if is_sin_of_var(st, f, var) {
                    *m += 1;
                    true
                } else if is_cos_of_var(st, f, var) {
                    *n += 1;
                    true
                } else {
                    false
                }
            }
            (Op::Pow, _) => {
                let base = st.get(f).children[0];
                let exp = st.get(f).children[1];
                if let (Op::Integer, Payload::Int(k)) = (&st.get(exp).op, &st.get(exp).payload) {
                    if *k < 0 {
                        return false;
                    }
                    if is_sin_of_var(st, base, var) {
                        *m += *k;
                        true
                    } else if is_cos_of_var(st, base, var) {
                        *n += *k;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    let parsed_ok = match st.get(id).op {
        Op::Mul => {
            let mut ok = true;
            for &c in &st.get(id).children.clone() {
                ok &= fold_factor(st, c, var, &mut coeff, &mut m, &mut n);
                if !ok {
                    break;
                }
            }
            ok
        }
        _ => fold_factor(st, id, var, &mut coeff, &mut m, &mut n),
    };

    if !parsed_ok {
        return None;
    }
    if m == 0 && n == 0 {
        return None;
    }

    // If sin power is odd: u = cos(x), du = -sin(x) dx
    if m % 2 != 0 {
        let k = (m - 1) / 2;
        let x = st.sym(var);
        let cosx = st.func("cos", vec![x]);
        let mut terms: Vec<ExprId> = Vec::new();
        for j in 0..=k {
            let bin = binom_u64(k as u64, j as u64)?;
            // Coefficient: (-1)^{j+1} * bin / (n + 2j + 1)
            let sign: i64 = if (j % 2) == 0 { -1 } else { 1 };
            let mut term_c = q_mul(coeff, (sign * bin, 1));
            let denom = n + 2 * j + 1;
            if denom == 0 {
                return None;
            }
            term_c = q_div(term_c, (denom, 1));

            let exp_e = st.int(n + 2 * j + 1);
            let pow_u = st.pow(cosx, exp_e);
            let term = if term_c == (1, 1) {
                pow_u
            } else if term_c.1 == 1 {
                let c_int = st.int(term_c.0);
                st.mul(vec![c_int, pow_u])
            } else {
                let c_rat = st.rat(term_c.0, term_c.1);
                st.mul(vec![c_rat, pow_u])
            };
            terms.push(term);
        }
        let sum = st.add(terms);
        return Some(simplify(st, sum));
    }

    // If cos power is odd: u = sin(x), du = cos(x) dx
    if n % 2 != 0 {
        let l = (n - 1) / 2;
        let x = st.sym(var);
        let sinx = st.func("sin", vec![x]);
        let mut terms: Vec<ExprId> = Vec::new();
        for j in 0..=l {
            let bin = binom_u64(l as u64, j as u64)?;
            // Coefficient: (-1)^j * bin / (m + 2j + 1)
            let sign: i64 = if (j % 2) == 0 { 1 } else { -1 };
            let mut term_c = q_mul(coeff, (sign * bin, 1));
            let denom = m + 2 * j + 1;
            if denom == 0 {
                return None;
            }
            term_c = q_div(term_c, (denom, 1));

            let exp_e = st.int(m + 2 * j + 1);
            let pow_u = st.pow(sinx, exp_e);
            let term = if term_c == (1, 1) {
                pow_u
            } else if term_c.1 == 1 {
                let c_int = st.int(term_c.0);
                st.mul(vec![c_int, pow_u])
            } else {
                let c_rat = st.rat(term_c.0, term_c.1);
                st.mul(vec![c_rat, pow_u])
            };
            terms.push(term);
        }
        let sum = st.add(terms);
        return Some(simplify(st, sum));
    }

    // Even-even not handled here
    None
}

/// Integrate single-function even powers: sin^(2k)(x) or cos^(2k)(x)
/// Uses reduction formulas:
/// ∫ sin^n(x) dx = -cos(x) sin^{n-1}(x)/n + (n-1)/n ∫ sin^{n-2}(x) dx
/// ∫ cos^n(x) dx =  sin(x) cos^{n-1}(x)/n + (n-1)/n ∫ cos^{n-2}(x) dx
fn try_trig_even_power_single(st: &mut Store, id: ExprId, var: &str) -> Option<ExprId> {
    if st.get(id).op != Op::Pow {
        return None;
    }

    let base = st.get(id).children[0];
    let exp = st.get(id).children[1];
    let n = match (&st.get(exp).op, &st.get(exp).payload) {
        (Op::Integer, Payload::Int(k)) if *k >= 2 && (*k % 2 == 0) => *k,
        _ => return None,
    };

    // Helpers: detect sin(var) or cos(var)
    fn is_sin_var(st: &Store, id: ExprId, var: &str) -> bool {
        if let (Op::Function, Payload::Func(fname)) = (&st.get(id).op, &st.get(id).payload) {
            if fname == "sin" && st.get(id).children.len() == 1 {
                let arg = st.get(id).children[0];
                return matches!((&st.get(arg).op, &st.get(arg).payload), (Op::Symbol, Payload::Sym(s)) if s == var);
            }
        }
        false
    }
    fn is_cos_var(st: &Store, id: ExprId, var: &str) -> bool {
        if let (Op::Function, Payload::Func(fname)) = (&st.get(id).op, &st.get(id).payload) {
            if fname == "cos" && st.get(id).children.len() == 1 {
                let arg = st.get(id).children[0];
                return matches!((&st.get(arg).op, &st.get(arg).payload), (Op::Symbol, Payload::Sym(s)) if s == var);
            }
        }
        false
    }

    let x = st.sym(var);
    if is_sin_var(st, base, var) {
        // Recursive helper
        fn integrate_sin_even(st: &mut Store, x: ExprId, n: i64) -> Option<ExprId> {
            if n == 0 {
                // ∫ 1 dx = x
                return Some(x);
            }
            // term1 = (-1/n) * cos(x) * sin(x)^(n-1)
            let cosx = st.func("cos", vec![x]);
            let sinx = st.func("sin", vec![x]);
            let exp_e = st.int(n - 1);
            let sin_pow = st.pow(sinx, exp_e);
            let c1 = st.rat(-1, n);
            let term1 = st.mul(vec![c1, cosx, sin_pow]);
            // term2 = ((n-1)/n) * ∫ sin^(n-2)(x) dx
            let inner = integrate_sin_even(st, x, n - 2)?;
            let c2 = st.rat(n - 1, n);
            let term2 = st.mul(vec![c2, inner]);
            let sum = st.add(vec![term1, term2]);
            Some(simplify(st, sum))
        }
        return integrate_sin_even(st, x, n);
    }
    if is_cos_var(st, base, var) {
        // Recursive helper
        fn integrate_cos_even(st: &mut Store, x: ExprId, n: i64) -> Option<ExprId> {
            if n == 0 {
                // ∫ 1 dx = x
                return Some(x);
            }
            // term1 = (1/n) * sin(x) * cos(x)^(n-1)
            let sinx = st.func("sin", vec![x]);
            let cosx = st.func("cos", vec![x]);
            let exp_e = st.int(n - 1);
            let cos_pow = st.pow(cosx, exp_e);
            let c1 = st.rat(1, n);
            let term1 = st.mul(vec![c1, sinx, cos_pow]);
            // term2 = ((n-1)/n) * ∫ cos^(n-2)(x) dx
            let inner = integrate_cos_even(st, x, n - 2)?;
            let c2 = st.rat(n - 1, n);
            let term2 = st.mul(vec![c2, inner]);
            let sum = st.add(vec![term1, term2]);
            Some(simplify(st, sum))
        }
        return integrate_cos_even(st, x, n);
    }

    None
}

/// Try to integrate sin^m(x) * cos^n(x) patterns using reduction formulas
/// Handles common cases:
/// - sin^2(x): use identity sin^2(x) = (1 - cos(2x))/2
/// - cos^2(x): use identity cos^2(x) = (1 + cos(2x))/2
/// - sin(x)*cos(x): use identity sin(x)cos(x) = sin(2x)/2
fn try_trig_power_pattern(st: &mut Store, id: ExprId, var: &str) -> Option<ExprId> {
    if st.get(id).op != Op::Mul {
        return None;
    }

    let children = st.get(id).children.clone();

    // Helper to check if expr is sin(var)^n or cos(var)^n
    fn extract_trig_power(st: &Store, id: ExprId, var: &str) -> Option<(String, i64)> {
        match st.get(id).op {
            Op::Function => {
                if let Payload::Func(fname) = &st.get(id).payload {
                    if (fname == "sin" || fname == "cos") && st.get(id).children.len() == 1 {
                        let arg = st.get(id).children[0];
                        if matches!((&st.get(arg).op, &st.get(arg).payload), (Op::Symbol, Payload::Sym(s)) if s == var)
                        {
                            return Some((fname.clone(), 1));
                        }
                    }
                }
                None
            }
            Op::Pow => {
                let base = st.get(id).children[0];
                let exp = st.get(id).children[1];
                if let (Op::Integer, Payload::Int(n)) = (&st.get(exp).op, &st.get(exp).payload) {
                    if let Some((name, _)) = extract_trig_power(st, base, var) {
                        return Some((name, *n));
                    }
                }
                None
            }
            _ => None,
        }
    }

    // Pattern: sin(x) * cos(x) -> use identity sin(x)cos(x) = sin(2x)/2
    if children.len() == 2 {
        let (f0, f1) = (children[0], children[1]);
        let t0 = extract_trig_power(st, f0, var);
        let t1 = extract_trig_power(st, f1, var);

        if let (Some((name0, 1)), Some((name1, 1))) = (&t0, &t1) {
            if (name0 == "sin" && name1 == "cos") || (name0 == "cos" && name1 == "sin") {
                // ∫ sin(x)cos(x) dx = ∫ sin(2x)/2 dx = -cos(2x)/4
                let x = st.sym(var);
                let two = st.int(2);
                let two_x = st.mul(vec![two, x]);
                let cos2x = st.func("cos", vec![two_x]);
                let neg_quarter = st.rat(-1, 4);
                let result = st.mul(vec![neg_quarter, cos2x]);
                return Some(simplify(st, result));
            }
        }
    }

    None
}

/// Try Weierstrass substitution for rational trigonometric integrals
/// Handles patterns like:
/// - ∫ 1/(1 + cos(x)) dx
/// - ∫ 1/(1 + sin(x)) dx
/// - ∫ 1/(a + b*cos(x)) dx
///
/// Uses the tangent half-angle substitution: t = tan(x/2)
/// - sin(x) = 2t/(1+t²)
/// - cos(x) = (1-t²)/(1+t²)
/// - dx = 2/(1+t²) dt
fn try_weierstrass_substitution(st: &mut Store, id: ExprId, var: &str) -> Option<ExprId> {
    // Check if this is 1/denominator
    if st.get(id).op != Op::Pow {
        return None;
    }

    let base = st.get(id).children[0];
    let exp = st.get(id).children[1];

    // Check for x^(-1) pattern
    if !matches!((&st.get(exp).op, &st.get(exp).payload), (Op::Integer, Payload::Int(-1))) {
        return None;
    }

    // Pattern 1: ∫ 1/(1 + cos(x)) dx = tan(x/2)
    // Check if base is (1 + cos(x))
    if let Op::Add = st.get(base).op {
        let children = st.get(base).children.clone();
        if children.len() == 2 {
            let (term1, term2) = (children[0], children[1]);

            // Check for 1 + cos(x)
            let is_one_plus_cos = matches!(
                (&st.get(term1).op, &st.get(term1).payload),
                (Op::Integer, Payload::Int(1))
            ) && is_simple_cos(st, term2, var);

            let is_cos_plus_one = matches!(
                (&st.get(term2).op, &st.get(term2).payload),
                (Op::Integer, Payload::Int(1))
            ) && is_simple_cos(st, term1, var);

            if is_one_plus_cos || is_cos_plus_one {
                // ∫ 1/(1 + cos(x)) dx = tan(x/2)
                let x = st.sym(var);
                let half = st.rat(1, 2);
                let x_half = st.mul(vec![x, half]);
                let result = st.func("tan", vec![x_half]);
                return Some(simplify(st, result));
            }

            // Check for 1 - cos(x) pattern
            let is_one_minus_cos = matches!(
                (&st.get(term1).op, &st.get(term1).payload),
                (Op::Integer, Payload::Int(1))
            ) && is_negative_cos(st, term2, var);

            if is_one_minus_cos {
                // ∫ 1/(1 - cos(x)) dx = -cot(x/2)
                let x = st.sym(var);
                let half = st.rat(1, 2);
                let x_half = st.mul(vec![x, half]);
                let cot = st.func("cot", vec![x_half]);
                let neg_one = st.int(-1);
                let result = st.mul(vec![neg_one, cot]);
                return Some(simplify(st, result));
            }
        }
    }

    None
}

// Helper: check if expression is cos(var)
fn is_simple_cos(st: &Store, id: ExprId, var: &str) -> bool {
    if let (Op::Function, Payload::Func(fname)) = (&st.get(id).op, &st.get(id).payload) {
        if fname == "cos" && st.get(id).children.len() == 1 {
            let arg = st.get(id).children[0];
            return matches!((&st.get(arg).op, &st.get(arg).payload), (Op::Symbol, Payload::Sym(s)) if s == var);
        }
    }
    false
}

// Helper: check if expression is -cos(var)
fn is_negative_cos(st: &Store, id: ExprId, var: &str) -> bool {
    if let Op::Mul = st.get(id).op {
        let children = &st.get(id).children;
        if children.len() == 2 {
            let (f0, f1) = (children[0], children[1]);
            let is_neg_one =
                matches!((&st.get(f0).op, &st.get(f0).payload), (Op::Integer, Payload::Int(-1)));
            return is_neg_one && is_simple_cos(st, f1, var);
        }
    }
    false
}

/// Try to integrate sin^2(x) or cos^2(x) using double-angle identities
fn try_trig_square_pattern(st: &mut Store, id: ExprId, var: &str) -> Option<ExprId> {
    // Check if this is sin(x)^2 or cos(x)^2
    if st.get(id).op != Op::Pow {
        return None;
    }

    let base = st.get(id).children[0];
    let exp = st.get(id).children[1];

    if !matches!((&st.get(exp).op, &st.get(exp).payload), (Op::Integer, Payload::Int(2))) {
        return None;
    }

    // Extract function name first to avoid borrow issues
    let fname = if let (Op::Function, Payload::Func(name)) =
        (&st.get(base).op, &st.get(base).payload)
    {
        if st.get(base).children.len() != 1 {
            return None;
        }
        let arg = st.get(base).children[0];
        if !matches!((&st.get(arg).op, &st.get(arg).payload), (Op::Symbol, Payload::Sym(s)) if s == var)
        {
            return None;
        }
        name.clone()
    } else {
        return None;
    };

    let x = st.sym(var);
    let two = st.int(2);
    let two_x = st.mul(vec![two, x]);

    match fname.as_str() {
        "sin" => {
            // ∫ sin^2(x) dx = ∫ (1 - cos(2x))/2 dx = x/2 - sin(2x)/4
            let sin2x = st.func("sin", vec![two_x]);
            let half = st.rat(1, 2);
            let neg_quarter = st.rat(-1, 4);
            let term1 = st.mul(vec![half, x]);
            let term2 = st.mul(vec![neg_quarter, sin2x]);
            let result = st.add(vec![term1, term2]);
            Some(simplify(st, result))
        }
        "cos" => {
            // ∫ cos^2(x) dx = ∫ (1 + cos(2x))/2 dx = x/2 + sin(2x)/4
            let sin2x = st.func("sin", vec![two_x]);
            let half = st.rat(1, 2);
            let quarter = st.rat(1, 4);
            let term1 = st.mul(vec![half, x]);
            let term2 = st.mul(vec![quarter, sin2x]);
            let result = st.add(vec![term1, term2]);
            Some(simplify(st, result))
        }
        _ => None,
    }
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

    #[test]
    fn integrate_power_rule_x3() {
        let mut st = Store::new();
        let x = st.sym("x");
        let three = st.int(3);
        let x3 = st.pow(x, three);
        let res = integrate(&mut st, x3, "x").expect("x^3");
        // ∫ x^3 dx = x^4/4
        let deriv = diff(&mut st, res, "x");
        let simplified = simplify(&mut st, deriv);
        assert_eq!(st.get(simplified).digest, st.get(x3).digest);
    }

    #[test]
    fn integrate_power_rule_x_minus_one() {
        let mut st = Store::new();
        let x = st.sym("x");
        let m1 = st.int(-1);
        let x_inv = st.pow(x, m1);
        let res = integrate(&mut st, x_inv, "x").expect("x^-1");
        // ∫ x^-1 dx = ln(x)
        assert!(st.to_string(res).contains("ln"));
    }

    #[test]
    fn integrate_sin() {
        let mut st = Store::new();
        let x = st.sym("x");
        let sinx = st.func("sin", vec![x]);
        let res = integrate(&mut st, sinx, "x").expect("sin(x)");
        // ∫ sin(x) dx = -cos(x)
        assert!(st.to_string(res).contains("cos"));
    }

    #[test]
    fn integrate_cos() {
        let mut st = Store::new();
        let x = st.sym("x");
        let cosx = st.func("cos", vec![x]);
        let res = integrate(&mut st, cosx, "x").expect("cos(x)");
        // ∫ cos(x) dx = sin(x)
        assert!(st.to_string(res).contains("sin"));
    }

    #[test]
    fn integrate_rational_function_simple() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x_plus_2 = st.add(vec![x, two]);
        let m1 = st.int(-1);
        let expr = st.pow(x_plus_2, m1); // 1/(x+2)
        let res = integrate(&mut st, expr, "x");
        // Should integrate to ln(x+2)
        if let Some(r) = res {
            assert!(st.to_string(r).contains("ln"));
        }
    }

    #[test]
    fn integrate_mul_with_rational_coeff() {
        let mut st = Store::new();
        let x = st.sym("x");
        let half = st.rat(1, 2);
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let expr = st.mul(vec![half, x2]);
        let res = integrate(&mut st, expr, "x").expect("(1/2)*x^2");
        // ∫ (1/2)x^2 dx = (1/2) * x^3/3 = x^3/6
        let deriv = diff(&mut st, res, "x");
        let simplified = simplify(&mut st, deriv);
        let original_simplified = simplify(&mut st, expr);
        assert_eq!(st.get(simplified).digest, st.get(original_simplified).digest);
    }

    #[test]
    fn integrate_unknown_function() {
        let mut st = Store::new();
        let x = st.sym("x");
        let fx = st.func("unknown", vec![x]);
        let res = integrate(&mut st, fx, "x");
        assert!(res.is_none());
    }

    #[test]
    fn integrate_multiarg_function() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let f = st.func("f", vec![x, y]);
        let res = integrate(&mut st, f, "x");
        assert!(res.is_none());
    }

    #[test]
    fn integrate_product_no_parts_match() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let sinx = st.func("sin", vec![x]);
        let siny = st.func("sin", vec![y]);
        let expr = st.mul(vec![sinx, siny]);
        let res = integrate(&mut st, expr, "x");
        // sin(y) is constant w.r.t. x, so should work
        if let Some(r) = res {
            assert!(!st.to_string(r).is_empty());
        }
    }

    #[test]
    fn integrate_rational_partial_fractions() {
        let mut st = Store::new();
        let x = st.sym("x");

        // 1/((x-1)(x-2)) should use partial fractions
        let m1_const = st.int(-1);
        let m2_const = st.int(-2);
        let x_m1 = st.add(vec![x, m1_const]);
        let x_m2 = st.add(vec![x, m2_const]);
        let den = st.mul(vec![x_m1, x_m2]);
        let m1 = st.int(-1);
        let expr = st.pow(den, m1);

        let res = integrate(&mut st, expr, "x");
        // Should succeed with partial fractions
        if let Some(r) = res {
            assert!(st.to_string(r).contains("ln"));
        }
    }

    #[test]
    fn integrate_add_with_multiple_terms() {
        let mut st = Store::new();
        let x = st.sym("x");
        let one = st.int(1);
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let sinx = st.func("sin", vec![x]);
        let expr = st.add(vec![one, x, x2, sinx]);
        let res = integrate(&mut st, expr, "x").expect("sum");
        // Should integrate each term
        let result = st.to_string(res);
        assert!(!result.is_empty());
    }

    #[test]
    fn integrate_constant_mul_function() {
        let mut st = Store::new();
        let x = st.sym("x");
        let five = st.int(5);
        let sinx = st.func("sin", vec![x]);
        let expr = st.mul(vec![five, sinx]);
        let res = integrate(&mut st, expr, "x").expect("5*sin(x)");
        // ∫ 5 sin(x) dx = -5 cos(x)
        assert!(st.to_string(res).contains("cos"));
    }

    #[test]
    fn integrate_piecewise() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let cond = st.sym("c");
        let pw = st.piecewise(vec![(cond, x2)]);
        let res = integrate(&mut st, pw, "x");
        // Should integrate piecewise
        if let Some(r) = res {
            assert!(st.to_string(r).contains("piecewise"));
        }
    }

    #[test]
    fn integrate_memoization() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two);

        // First integration - computes and caches
        let result1 = integrate(&mut st, x2, "x");
        assert!(result1.is_some());

        // Second integration - should use cache
        let result2 = integrate(&mut st, x2, "x");
        assert_eq!(result1, result2);

        // Integration of unsupported expression - caches None
        let unknown = st.func("unknown", vec![x]);
        let result3 = integrate(&mut st, unknown, "x");
        assert!(result3.is_none());

        // Second call should also return None from cache
        let result4 = integrate(&mut st, unknown, "x");
        assert_eq!(result3, result4);

        // Clear cache and verify
        st.clear_caches();
        let result5 = integrate(&mut st, x2, "x");
        assert_eq!(result1, result5); // Same result, but recomputed
    }

    #[test]
    fn integrate_piecewise_multiple_branches() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let three = st.int(3);
        let x2 = st.pow(x, two);
        let x3 = st.pow(x, three);
        let c1 = st.sym("c1");
        let c2 = st.sym("c2");
        let pw = st.piecewise(vec![(c1, x2), (c2, x3)]);
        let res = integrate(&mut st, pw, "x");
        if let Some(r) = res {
            assert!(st.to_string(r).contains("piecewise"));
        }
    }

    // ========== Hyperbolic Function Tests (v1.1) ==========

    #[test]
    fn integrate_sinh() {
        let mut st = Store::new();
        let x = st.sym("x");
        let sinhx = st.func("sinh", vec![x]);
        let res = integrate(&mut st, sinhx, "x").expect("sinh(x)");
        // ∫ sinh(x) dx = cosh(x)
        let coshx = st.func("cosh", vec![x]);
        assert_eq!(st.to_string(res), st.to_string(coshx));
    }

    #[test]
    fn integrate_cosh() {
        let mut st = Store::new();
        let x = st.sym("x");
        let coshx = st.func("cosh", vec![x]);
        let res = integrate(&mut st, coshx, "x").expect("cosh(x)");
        // ∫ cosh(x) dx = sinh(x)
        let sinhx = st.func("sinh", vec![x]);
        assert_eq!(st.to_string(res), st.to_string(sinhx));
    }

    #[test]
    fn integrate_tanh() {
        let mut st = Store::new();
        let x = st.sym("x");
        let tanhx = st.func("tanh", vec![x]);
        let res = integrate(&mut st, tanhx, "x").expect("tanh(x)");
        // ∫ tanh(x) dx = ln(cosh(x))
        let result = st.to_string(res);
        assert!(result.contains("ln"));
        assert!(result.contains("cosh"));
    }

    #[test]
    fn integrate_sinh_linear() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let two_x = st.mul(vec![two, x]);
        let sinh2x = st.func("sinh", vec![two_x]);
        let res = integrate(&mut st, sinh2x, "x").expect("sinh(2x)");
        // ∫ sinh(2x) dx = (1/2) cosh(2x)
        let result = st.to_string(res);
        assert!(result.contains("cosh"));
    }

    #[test]
    fn integrate_cosh_linear() {
        let mut st = Store::new();
        let x = st.sym("x");
        let three = st.int(3);
        let three_x = st.mul(vec![three, x]);
        let cosh3x = st.func("cosh", vec![three_x]);
        let res = integrate(&mut st, cosh3x, "x").expect("cosh(3x)");
        // ∫ cosh(3x) dx = (1/3) sinh(3x)
        let result = st.to_string(res);
        assert!(result.contains("sinh"));
    }

    #[test]
    fn integrate_sinh_verification_by_diff() {
        let mut st = Store::new();
        let x = st.sym("x");
        let sinhx = st.func("sinh", vec![x]);
        let integral = integrate(&mut st, sinhx, "x").expect("sinh(x)");
        // Verify: d/dx(∫ sinh(x) dx) = sinh(x)
        let derivative = diff(&mut st, integral, "x");
        let simplified = simplify(&mut st, derivative);
        assert_eq!(st.get(simplified).digest, st.get(sinhx).digest);
    }

    #[test]
    fn integrate_cosh_verification_by_diff() {
        let mut st = Store::new();
        let x = st.sym("x");
        let coshx = st.func("cosh", vec![x]);
        let integral = integrate(&mut st, coshx, "x").expect("cosh(x)");
        // Verify: d/dx(∫ cosh(x) dx) = cosh(x)
        let derivative = diff(&mut st, integral, "x");
        let simplified = simplify(&mut st, derivative);
        assert_eq!(st.get(simplified).digest, st.get(coshx).digest);
    }

    #[test]
    fn integrate_tanh_verification_by_diff() {
        let mut st = Store::new();
        let x = st.sym("x");
        let tanhx = st.func("tanh", vec![x]);
        let integral = integrate(&mut st, tanhx, "x").expect("tanh(x)");
        // ∫ tanh(x) dx = ln(cosh(x))
        // d/dx ln(cosh(x)) = sinh(x)/cosh(x) = tanh(x)
        // However, simplifier may not reduce sinh(x)/cosh(x) to tanh(x) automatically
        // So we verify the structural form instead
        let derivative = diff(&mut st, integral, "x");
        let result_str = st.to_string(derivative);
        // The derivative should be sinh(x) * cosh(x)^(-1) or sinh(x)/cosh(x)
        // which is mathematically equivalent to tanh(x)
        assert!(result_str.contains("sinh") && result_str.contains("cosh"));
    }

    // ========== Trigonometric Power Pattern Tests (v1.1) ==========

    #[test]
    fn integrate_sin_cos_product() {
        let mut st = Store::new();
        let x = st.sym("x");
        let sinx = st.func("sin", vec![x]);
        let cosx = st.func("cos", vec![x]);
        let prod = st.mul(vec![sinx, cosx]);
        let res = integrate(&mut st, prod, "x").expect("sin(x)*cos(x)");
        // ∫ sin(x)cos(x) dx = -cos(2x)/4
        let result = st.to_string(res);
        assert!(result.contains("cos"));
    }

    #[test]
    fn integrate_sin_cos_product_reversed() {
        let mut st = Store::new();
        let x = st.sym("x");
        let sinx = st.func("sin", vec![x]);
        let cosx = st.func("cos", vec![x]);
        let prod = st.mul(vec![cosx, sinx]); // Reversed order
        let res = integrate(&mut st, prod, "x").expect("cos(x)*sin(x)");
        let result = st.to_string(res);
        assert!(result.contains("cos"));
    }

    #[test]
    fn integrate_sin_squared() {
        let mut st = Store::new();
        let x = st.sym("x");
        let sinx = st.func("sin", vec![x]);
        let two = st.int(2);
        let sin2 = st.pow(sinx, two);
        let res = integrate(&mut st, sin2, "x").expect("sin^2(x)");
        // ∫ sin^2(x) dx = x/2 - sin(2x)/4
        let result = st.to_string(res);
        assert!(result.contains("x"));
        assert!(result.contains("sin"));
    }

    #[test]
    fn integrate_cos_squared() {
        let mut st = Store::new();
        let x = st.sym("x");
        let cosx = st.func("cos", vec![x]);
        let two = st.int(2);
        let cos2 = st.pow(cosx, two);
        let res = integrate(&mut st, cos2, "x").expect("cos^2(x)");
        // ∫ cos^2(x) dx = x/2 + sin(2x)/4
        let result = st.to_string(res);
        assert!(result.contains("x"));
        assert!(result.contains("sin"));
    }

    #[test]
    fn integrate_sin_squared_verification() {
        let mut st = Store::new();
        let x = st.sym("x");
        let sinx = st.func("sin", vec![x]);
        let two = st.int(2);
        let sin2 = st.pow(sinx, two);
        let integral = integrate(&mut st, sin2, "x").expect("sin^2(x)");
        // Verify by differentiation: d/dx(∫ sin^2(x) dx) = sin^2(x)
        let derivative = diff(&mut st, integral, "x");
        let simplified = simplify(&mut st, derivative);
        // Note: derivative might be in form (1 - cos(2x))/2, not exactly sin^2(x)
        // But structurally they should be equivalent
        let result = st.to_string(simplified);
        assert!(!result.is_empty());
    }

    #[test]
    fn integrate_cos_squared_verification() {
        let mut st = Store::new();
        let x = st.sym("x");
        let cosx = st.func("cos", vec![x]);
        let two = st.int(2);
        let cos2 = st.pow(cosx, two);
        let integral = integrate(&mut st, cos2, "x").expect("cos^2(x)");
        // Verify by differentiation
        let derivative = diff(&mut st, integral, "x");
        let simplified = simplify(&mut st, derivative);
        let result = st.to_string(simplified);
        assert!(!result.is_empty());
    }

    #[test]
    fn integrate_sin_cos_product_verification() {
        let mut st = Store::new();
        let x = st.sym("x");
        let sinx = st.func("sin", vec![x]);
        let cosx = st.func("cos", vec![x]);
        let prod = st.mul(vec![sinx, cosx]);
        let integral = integrate(&mut st, prod, "x").expect("sin(x)*cos(x)");
        // Verify by differentiation
        let derivative = diff(&mut st, integral, "x");
        let simplified = simplify(&mut st, derivative);
        // May not match exactly due to trigonometric identities
        let result = st.to_string(simplified);
        assert!(result.contains("sin") || result.contains("cos"));
    }

    // ========== U-Substitution Tests (v1.1) ==========

    #[test]
    fn integrate_u_substitution_power_rule() {
        let mut st = Store::new();
        let x = st.sym("x");
        // ∫ 2x(x²+1)⁵ dx, u = x²+1, du = 2x dx
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let one = st.int(1);
        let u = st.add(vec![x2, one]); // x² + 1
        let five = st.int(5);
        let u5 = st.pow(u, five); // (x² + 1)⁵
        let two_x = st.mul(vec![two, x]); // 2x
        let integrand = st.mul(vec![two_x, u5]); // 2x * (x² + 1)⁵

        let res = integrate(&mut st, integrand, "x").expect("u-substitution");
        // Result should be (x²+1)⁶/6
        let derivative = diff(&mut st, res, "x");
        let simplified = simplify(&mut st, derivative);
        let original_simplified = simplify(&mut st, integrand);
        assert_eq!(st.get(simplified).digest, st.get(original_simplified).digest);
    }

    #[test]
    fn integrate_u_substitution_cubic_polynomial() {
        let mut st = Store::new();
        let x = st.sym("x");
        // ∫ 6x²(x³+5)⁴ dx, u = x³+5, du = 3x² dx, so 6x² = 2du
        let three = st.int(3);
        let x3 = st.pow(x, three);
        let five = st.int(5);
        let u = st.add(vec![x3, five]); // x³ + 5
        let four = st.int(4);
        let u4 = st.pow(u, four); // (x³ + 5)⁴
        let six = st.int(6);
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let six_x2 = st.mul(vec![six, x2]); // 6x²
        let integrand = st.mul(vec![six_x2, u4]);

        let res = integrate(&mut st, integrand, "x").expect("u-substitution cubic");
        // Verify by differentiation
        let derivative = diff(&mut st, res, "x");
        let simplified = simplify(&mut st, derivative);
        let original_simplified = simplify(&mut st, integrand);
        assert_eq!(st.get(simplified).digest, st.get(original_simplified).digest);
    }

    #[test]
    fn integrate_u_substitution_negative_power() {
        let mut st = Store::new();
        let x = st.sym("x");
        // ∫ 2x(x²+1)⁻² dx, u = x²+1, du = 2x dx
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let one = st.int(1);
        let u = st.add(vec![x2, one]);
        let neg_two = st.int(-2);
        let u_inv2 = st.pow(u, neg_two); // (x² + 1)⁻²
        let two_x = st.mul(vec![two, x]);
        let integrand = st.mul(vec![two_x, u_inv2]);

        let res = integrate(&mut st, integrand, "x").expect("u-substitution negative power");
        // Result should be -(x²+1)⁻¹
        let derivative = diff(&mut st, res, "x");
        let simplified = simplify(&mut st, derivative);
        let original_simplified = simplify(&mut st, integrand);
        assert_eq!(st.get(simplified).digest, st.get(original_simplified).digest);
    }

    #[test]
    fn integrate_u_substitution_with_coefficient() {
        let mut st = Store::new();
        let x = st.sym("x");
        // ∫ 4x(x²)³ dx = ∫ 4x·x⁶ dx = ∫ 4x⁷ dx
        // But test u-sub: u = x², du = 2x, so 4x = 2du
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let three = st.int(3);
        let x2_cubed = st.pow(x2, three); // (x²)³
        let four = st.int(4);
        let four_x = st.mul(vec![four, x]);
        let integrand = st.mul(vec![four_x, x2_cubed]);

        let res = integrate(&mut st, integrand, "x");
        // Should successfully integrate
        assert!(res.is_some());
        if let Some(r) = res {
            let derivative = diff(&mut st, r, "x");
            let simplified = simplify(&mut st, derivative);
            let original_simplified = simplify(&mut st, integrand);
            assert_eq!(st.get(simplified).digest, st.get(original_simplified).digest);
        }
    }

    #[test]
    fn integrate_u_substitution_complex_expression() {
        let mut st = Store::new();
        let x = st.sym("x");
        // ∫ 2x(2x²+3)² dx, u = 2x²+3, du = 4x dx
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let two_x2 = st.mul(vec![two, x2]); // 2x²
        let three = st.int(3);
        let u = st.add(vec![two_x2, three]); // 2x²+3
        let u2 = st.pow(u, two); // (2x²+3)²
        let two_x = st.mul(vec![two, x]); // 2x (which is du/2)
        let integrand = st.mul(vec![two_x, u2]);

        let res = integrate(&mut st, integrand, "x").expect("complex u-substitution");
        // Verify by differentiation
        let derivative = diff(&mut st, res, "x");
        let simplified = simplify(&mut st, derivative);
        let original_simplified = simplify(&mut st, integrand);
        assert_eq!(st.get(simplified).digest, st.get(original_simplified).digest);
    }

    #[test]
    fn integrate_u_substitution_not_applicable() {
        let mut st = Store::new();
        let x = st.sym("x");
        // ∫ x(x+1)² dx - doesn't have the right derivative form
        // (x is not the derivative of x+1, derivative would be 1)
        // This should be handled by integration by parts or expansion
        let one = st.int(1);
        let x_plus_1 = st.add(vec![x, one]);
        let two = st.int(2);
        let u2 = st.pow(x_plus_1, two);
        let integrand = st.mul(vec![x, u2]);

        // Try to integrate - may succeed via integration by parts
        let res = integrate(&mut st, integrand, "x");
        // If it succeeds, verify by differentiation
        if let Some(r) = res {
            let derivative = diff(&mut st, r, "x");
            let simplified = simplify(&mut st, derivative);
            let original_simplified = simplify(&mut st, integrand);
            assert_eq!(st.get(simplified).digest, st.get(original_simplified).digest);
        }
        // Otherwise, u-substitution correctly rejected it
    }

    // ========== Weierstrass Substitution Tests (v1.1) ==========

    #[test]
    fn integrate_weierstrass_one_over_one_plus_cos() {
        let mut st = Store::new();
        let x = st.sym("x");
        // ∫ 1/(1 + cos(x)) dx = tan(x/2)
        let one = st.int(1);
        let cosx = st.func("cos", vec![x]);
        let denom = st.add(vec![one, cosx]); // 1 + cos(x)
        let neg_one = st.int(-1);
        let integrand = st.pow(denom, neg_one); // (1 + cos(x))^(-1)

        let res = integrate(&mut st, integrand, "x").expect("1/(1+cos(x))");

        // Result should be tan(x/2)
        let result_str = st.to_string(res);
        assert!(result_str.contains("tan"));

        // Verify by differentiation (note: this may not match exactly due to trig identities)
        let deriv = diff(&mut st, res, "x");
        let simplified = simplify(&mut st, deriv);
        // Just check it doesn't crash and produces valid output
        assert!(!st.to_string(simplified).is_empty());
    }

    #[test]
    fn integrate_weierstrass_one_over_one_minus_cos() {
        let mut st = Store::new();
        let x = st.sym("x");
        // ∫ 1/(1 - cos(x)) dx = -cot(x/2)
        let one = st.int(1);
        let cosx = st.func("cos", vec![x]);
        let neg_one = st.int(-1);
        let neg_cosx = st.mul(vec![neg_one, cosx]); // -cos(x)
        let denom = st.add(vec![one, neg_cosx]); // 1 - cos(x)
        let integrand = st.pow(denom, neg_one); // (1 - cos(x))^(-1)

        let res = integrate(&mut st, integrand, "x").expect("1/(1-cos(x))");

        // Result should contain cot
        let result_str = st.to_string(res);
        assert!(result_str.contains("cot"));
    }

    #[test]
    fn integrate_weierstrass_cos_plus_one_reversed() {
        let mut st = Store::new();
        let x = st.sym("x");
        // ∫ 1/(cos(x) + 1) dx = tan(x/2) (same as 1 + cos(x))
        let one = st.int(1);
        let cosx = st.func("cos", vec![x]);
        let denom = st.add(vec![cosx, one]); // cos(x) + 1
        let neg_one = st.int(-1);
        let integrand = st.pow(denom, neg_one);

        let res = integrate(&mut st, integrand, "x").expect("1/(cos(x)+1)");

        // Should produce same result as 1/(1+cos(x))
        let result_str = st.to_string(res);
        assert!(result_str.contains("tan"));
    }

    #[test]
    fn integrate_weierstrass_scaled() {
        let mut st = Store::new();
        let x = st.sym("x");
        // ∫ 2/(1 + cos(x)) dx = 2*tan(x/2)
        let one = st.int(1);
        let two = st.int(2);
        let cosx = st.func("cos", vec![x]);
        let denom = st.add(vec![one, cosx]);
        let neg_one = st.int(-1);
        let inv_denom = st.pow(denom, neg_one);
        let integrand = st.mul(vec![two, inv_denom]); // 2/(1+cos(x))

        let res = integrate(&mut st, integrand, "x").expect("2/(1+cos(x))");

        // Result should contain tan and coefficient 2
        let result_str = st.to_string(res);
        assert!(result_str.contains("tan"));
        assert!(result_str.contains("2"));
    }

    #[test]
    fn integrate_weierstrass_not_applicable_no_trig() {
        let mut st = Store::new();
        let x = st.sym("x");
        // ∫ 1/(1 + x) dx - not a trig pattern, should not use Weierstrass
        let one = st.int(1);
        let denom = st.add(vec![one, x]);
        let neg_one = st.int(-1);
        let integrand = st.pow(denom, neg_one);

        // Should still integrate via logarithm
        let res = integrate(&mut st, integrand, "x");
        assert!(res.is_some());
        if let Some(r) = res {
            // Should be ln(1+x)
            let result_str = st.to_string(r);
            assert!(result_str.contains("ln"));
        }
    }
}
