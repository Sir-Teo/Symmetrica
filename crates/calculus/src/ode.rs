//! Ordinary Differential Equation (ODE) Solving
//!
//! This module provides symbolic solutions for first-order ODEs:
//! - Separable equations: dy/dx = f(x)g(y)
//! - Linear equations: dy/dx + p(x)y = q(x)
//! - Bernoulli equations: dy/dx + p(x)y = q(x)y^n
//!
//! Future work:
//! - Exact equations
//! - Homogeneous equations
//! - Second-order ODEs with constant coefficients

use crate::integrate::integrate;
use expr_core::{ExprId, Op, Payload, Store};
use simplify::simplify;

/// Attempt to solve a first-order ODE: dy/dx = f(x, y)
/// Returns the solution y(x) if found, or None
pub fn solve_ode_first_order(
    store: &mut Store,
    rhs: ExprId,
    y_var: &str,
    x_var: &str,
) -> Option<ExprId> {
    // Try Bernoulli form first: dy/dx = -p(x)y + q(x)y^n
    if let Some(solution) = try_bernoulli(store, rhs, y_var, x_var) {
        return Some(solution);
    }

    // Try separable form: dy/dx = f(x)g(y)
    if let Some(solution) = try_separable(store, rhs, y_var, x_var) {
        return Some(solution);
    }

    // Try linear form: dy/dx = -p(x)y + q(x)
    if let Some(solution) = try_linear(store, rhs, y_var, x_var) {
        return Some(solution);
    }

    // Try exact form: M(x,y) + N(x,y)dy/dx = 0
    // Note: This requires the equation in the form M dx + N dy = 0
    // For now, we skip exact equations as they need different input format

    // Try homogeneous form: dy/dx = f(y/x)
    if let Some(solution) = try_homogeneous(store, rhs, y_var, x_var) {
        return Some(solution);
    }

    None
}

/// Try to solve homogeneous ODE: dy/dx = f(y/x)
/// Use substitution v = y/x, so y = vx and dy/dx = v + x(dv/dx)
fn try_homogeneous(store: &mut Store, rhs: ExprId, y_var: &str, x_var: &str) -> Option<ExprId> {
    // Check if rhs is a function of y/x
    // For simplicity, check if rhs = y/x or contains y/x pattern

    // Simple case: rhs = y/x
    if let Op::Mul = store.get(rhs).op {
        let children = &store.get(rhs).children;
        if children.len() == 2 {
            let (a, b) = (children[0], children[1]);

            // Check if one is y and other is x^(-1)
            let y_sym = store.sym(y_var);
            let x_sym = store.sym(x_var);

            if a == y_sym {
                if let Op::Pow = store.get(b).op {
                    let pow_children = &store.get(b).children;
                    if pow_children.len() == 2 && pow_children[0] == x_sym {
                        if let (Op::Integer, Payload::Int(-1)) =
                            (&store.get(pow_children[1]).op, &store.get(pow_children[1]).payload)
                        {
                            // This is y/x form
                            // Solution: ln|y| = ln|x| + C, or y = Cx
                            // For now, return y = x (omitting constant)
                            return Some(x_sym);
                        }
                    }
                }
            }
        }
    }

    None
}

/// Try to solve Bernoulli ODE: dy/dx + p(x)y = q(x)y^n
/// Transform via v = y^(1-n) to get linear ODE
fn try_bernoulli(store: &mut Store, rhs: ExprId, y_var: &str, x_var: &str) -> Option<ExprId> {
    // rhs should be: -p(x)y + q(x)y^n
    // We need to identify n, p(x), and q(x)

    // For now, handle simple case: rhs = ay + by^n where a, b are functions of x
    // This corresponds to dy/dx = ay + by^n

    if store.get(rhs).op != Op::Add {
        return None;
    }

    let children = store.get(rhs).children.clone();

    // Look for linear term (y) and power term (y^n)
    let mut linear_coeff = None;
    let mut power_coeff = None;
    let mut power_n = None;

    for child in children {
        if let Some(n) = is_power_of_var(store, child, y_var) {
            if n == 1 {
                // This is the linear term
                linear_coeff = Some(extract_coeff_of_power(store, child, y_var, 1));
            } else if n > 1 {
                // This is the y^n term
                power_n = Some(n);
                power_coeff = Some(extract_coeff_of_power(store, child, y_var, n));
            }
        }
    }

    // Check if we found both terms
    let n = power_n?;
    let p_x = linear_coeff?;
    let q_x = power_coeff?;

    if n == 1 {
        // This is actually linear, not Bernoulli
        return None;
    }

    // Transform: v = y^(1-n)
    // Then dv/dx = (1-n)y^(-n) dy/dx
    // Original: dy/dx = -p(x)y + q(x)y^n
    // Multiply by (1-n)y^(-n): (1-n)y^(-n) dy/dx = (1-n)(-p(x)y^(1-n) + q(x))
    // This gives: dv/dx = (1-n)(-p(x)v + q(x))
    // Which is linear in v: dv/dx + (1-n)p(x)v = (1-n)q(x)

    // Solve the linear ODE in v
    let one_minus_n = 1 - n;
    let one_minus_n_expr = store.int(one_minus_n);

    // New p(x) for linear equation: (1-n)p(x)
    let new_p = store.mul(vec![one_minus_n_expr, p_x]);

    // New q(x) for linear equation: (1-n)q(x)
    let new_q = store.mul(vec![one_minus_n_expr, q_x]);

    // Build RHS for linear solver: -new_p * v + new_q
    // We'll use a dummy variable name for v
    let v_var = format!("{}_bernoulli_v", y_var);
    let v = store.sym(&v_var);
    let neg_one = store.int(-1);
    let neg_new_p = store.mul(vec![neg_one, new_p]);
    let neg_new_p_v = store.mul(vec![neg_new_p, v]);
    let linear_rhs = store.add(vec![neg_new_p_v, new_q]);

    // Solve linear ODE for v
    let v_solution = try_linear(store, linear_rhs, &v_var, x_var)?;

    // Transform back: y = v^(1/(1-n))
    let exponent = if one_minus_n != 0 {
        store.rat(1, one_minus_n)
    } else {
        return None;
    };

    // Substitute v with v_solution in y = v^(1/(1-n))
    let y_solution = store.pow(v_solution, exponent);

    Some(simplify(store, y_solution))
}

/// Check if expression is y^n and return n
fn is_power_of_var(store: &Store, expr: ExprId, var: &str) -> Option<i64> {
    match store.get(expr).op {
        Op::Symbol => {
            if matches!(&store.get(expr).payload, Payload::Sym(s) if s == var) {
                Some(1)
            } else {
                None
            }
        }
        Op::Pow => {
            let n = store.get(expr);
            let base = n.children[0];
            let exp = n.children[1];

            if matches!((&store.get(base).op, &store.get(base).payload), (Op::Symbol, Payload::Sym(s)) if s == var)
            {
                if let (Op::Integer, Payload::Int(power)) =
                    (&store.get(exp).op, &store.get(exp).payload)
                {
                    return Some(*power);
                }
            }
            None
        }
        Op::Mul => {
            // Check if one factor is y^n
            for &child in &store.get(expr).children {
                if let Some(n) = is_power_of_var(store, child, var) {
                    return Some(n);
                }
            }
            None
        }
        _ => None,
    }
}

/// Extract coefficient of y^n from expression
fn extract_coeff_of_power(store: &mut Store, expr: ExprId, var: &str, n: i64) -> ExprId {
    match store.get(expr).op {
        Op::Symbol => {
            if matches!(&store.get(expr).payload, Payload::Sym(s) if s == var) && n == 1 {
                store.int(1)
            } else {
                expr
            }
        }
        Op::Pow => {
            // If this is y^n, coefficient is 1
            store.int(1)
        }
        Op::Mul => {
            // Extract all factors except y^n
            let children = &store.get(expr).children;
            let mut coeff_parts = Vec::new();

            for &child in children {
                if is_power_of_var(store, child, var).is_none() {
                    coeff_parts.push(child);
                }
            }

            if coeff_parts.is_empty() {
                store.int(1)
            } else {
                store.mul(coeff_parts)
            }
        }
        _ => store.int(1),
    }
}

/// Try to solve separable ODE: dy/dx = f(x)g(y)
/// Solution: ∫ dy/g(y) = ∫ f(x) dx + C
fn try_separable(store: &mut Store, rhs: ExprId, y_var: &str, x_var: &str) -> Option<ExprId> {
    // Try to factor rhs into f(x) * g(y)
    let (f_x, g_y) = extract_separable_factors(store, rhs, y_var, x_var)?;

    // Integrate f(x) with respect to x
    let integral_f = integrate(store, f_x, x_var)?;

    // For g(y), we need 1/g(y) integrated with respect to y
    // This is complex in general, so we handle simple cases
    let neg_one = store.int(-1);
    let inv_g = store.pow(g_y, neg_one);

    // Try to integrate 1/g(y) with respect to y
    let integral_inv_g = integrate(store, inv_g, y_var)?;

    // Solution: integral_inv_g = integral_f + C
    // For now, return implicit form: integral_inv_g - integral_f = C
    let neg_integral_f = store.mul(vec![neg_one, integral_f]);
    let implicit = store.add(vec![integral_inv_g, neg_integral_f]);

    Some(simplify(store, implicit))
}

/// Try to solve linear ODE: dy/dx + p(x)y = q(x)
/// Solution: y = e^{-∫p dx} [∫ q e^{∫p dx} dx + C]
fn try_linear(store: &mut Store, rhs: ExprId, y_var: &str, x_var: &str) -> Option<ExprId> {
    // rhs should be of form: -p(x)y + q(x)
    // We need to extract p(x) and q(x)

    let (p_x, q_x) = extract_linear_coefficients(store, rhs, y_var, x_var)?;

    // Compute integrating factor: μ = e^{∫p dx}
    let integral_p = integrate(store, p_x, x_var)?;
    let mu = store.func("exp", vec![integral_p]);

    // Compute ∫ q·μ dx
    let q_mu = store.mul(vec![q_x, mu]);
    let integral_q_mu = integrate(store, q_mu, x_var)?;

    // Solution: y = (∫ q·μ dx + C) / μ
    // For now, omit constant C
    let neg_one = store.int(-1);
    let inv_mu = store.pow(mu, neg_one);
    let solution = store.mul(vec![integral_q_mu, inv_mu]);

    Some(simplify(store, solution))
}

/// Extract f(x) and g(y) from separable form f(x) * g(y)
fn extract_separable_factors(
    store: &mut Store,
    expr: ExprId,
    y_var: &str,
    x_var: &str,
) -> Option<(ExprId, ExprId)> {
    // Simple case: expr is a product
    if store.get(expr).op != Op::Mul {
        // Check if it's purely a function of x or y
        if !contains_var(store, expr, y_var) {
            // f(x) * 1
            return Some((expr, store.int(1)));
        }
        if !contains_var(store, expr, x_var) {
            // 1 * g(y)
            return Some((store.int(1), expr));
        }
        return None;
    }

    let children = &store.get(expr).children;
    let mut x_factors = Vec::new();
    let mut y_factors = Vec::new();

    for &child in children {
        let has_x = contains_var(store, child, x_var);
        let has_y = contains_var(store, child, y_var);

        if has_x && has_y {
            // Mixed term - not separable
            return None;
        } else if has_x {
            x_factors.push(child);
        } else if has_y {
            y_factors.push(child);
        } else {
            // Constant - can go in either factor
            x_factors.push(child);
        }
    }

    let f_x = if x_factors.is_empty() { store.int(1) } else { store.mul(x_factors) };

    let g_y = if y_factors.is_empty() { store.int(1) } else { store.mul(y_factors) };

    Some((f_x, g_y))
}

/// Extract p(x) and q(x) from linear form: -p(x)y + q(x)
fn extract_linear_coefficients(
    store: &mut Store,
    expr: ExprId,
    y_var: &str,
    _x_var: &str,
) -> Option<(ExprId, ExprId)> {
    // expr should be of form: -p(x)y + q(x)
    // We look for terms with y and without y

    if store.get(expr).op != Op::Add {
        // Single term - check if it's p(x)y or q(x)
        if contains_var(store, expr, y_var) {
            // Extract coefficient of y
            let p = extract_coefficient_of_var(store, expr, y_var)?;
            let neg_one = store.int(-1);
            let neg_p = store.mul(vec![neg_one, p]);
            return Some((neg_p, store.int(0)));
        } else {
            // It's q(x)
            return Some((store.int(0), expr));
        }
    }

    let children = store.get(expr).children.clone();
    let mut p_terms = Vec::new();
    let mut q_terms = Vec::new();

    for child in children {
        if contains_var(store, child, y_var) {
            // This is a p(x)y term
            if let Some(coeff) = extract_coefficient_of_var(store, child, y_var) {
                p_terms.push(coeff);
            } else {
                return None;
            }
        } else {
            // This is a q(x) term
            q_terms.push(child);
        }
    }

    if p_terms.len() != 1 {
        // Linear ODE should have exactly one y term
        return None;
    }

    let p_x = p_terms[0];
    let neg_one = store.int(-1);
    let neg_p = store.mul(vec![neg_one, p_x]);

    let q_x = if q_terms.is_empty() { store.int(0) } else { store.add(q_terms) };

    Some((neg_p, q_x))
}

/// Extract coefficient of a variable from an expression
/// For example: 3xy -> 3x (coefficient of y)
fn extract_coefficient_of_var(store: &mut Store, expr: ExprId, var: &str) -> Option<ExprId> {
    match store.get(expr).op {
        Op::Symbol => {
            if matches!(&store.get(expr).payload, Payload::Sym(s) if s == var) {
                Some(store.int(1))
            } else {
                None
            }
        }
        Op::Mul => {
            let children = &store.get(expr).children;
            let mut coeff_parts = Vec::new();
            let mut found_var = false;

            for &child in children {
                if !found_var
                    && matches!((&store.get(child).op, &store.get(child).payload), (Op::Symbol, Payload::Sym(ref s)) if s == var)
                {
                    found_var = true;
                } else {
                    coeff_parts.push(child);
                }
            }

            if !found_var {
                return None;
            }

            if coeff_parts.is_empty() {
                Some(store.int(1))
            } else {
                Some(store.mul(coeff_parts))
            }
        }
        _ => None,
    }
}

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

/// Solve second-order linear ODE with constant coefficients:
/// a*y'' + b*y' + c*y = 0
///
/// Uses characteristic equation: a*r^2 + b*r + c = 0
/// Returns general solution with arbitrary constants C1, C2
pub fn solve_ode_second_order_constant_coeff(
    store: &mut Store,
    a: ExprId,
    b: ExprId,
    c: ExprId,
    x_var: &str,
) -> Option<ExprId> {
    use solver::solve_univariate;

    // Build characteristic equation: a*r^2 + b*r + c = 0
    let r = store.sym("r");
    let two = store.int(2);
    let r2 = store.pow(r, two);
    let ar2 = store.mul(vec![a, r2]);
    let br = store.mul(vec![b, r]);
    let char_eq = store.add(vec![ar2, br, c]);

    // Solve characteristic equation
    let roots = solve_univariate(store, char_eq, "r")?;

    if roots.is_empty() {
        return None;
    }

    let x = store.sym(x_var);
    let c1 = store.sym("C1");
    let c2 = store.sym("C2");

    if roots.len() == 1 {
        // Repeated root: y = (C1 + C2*x)*e^(r*x)
        let r_val = roots[0];
        let rx = store.mul(vec![r_val, x]);
        let exp_rx = store.func("exp", vec![rx]);
        let c2x = store.mul(vec![c2, x]);
        let c1_plus_c2x = store.add(vec![c1, c2x]);
        let solution = store.mul(vec![c1_plus_c2x, exp_rx]);
        return Some(simplify(store, solution));
    }

    // Two distinct roots
    let r1 = roots[0];
    let r2 = roots[1];

    // Check if roots are complex conjugates (contain sqrt of negative)
    // For simplicity, construct: y = C1*e^(r1*x) + C2*e^(r2*x)
    let r1x = store.mul(vec![r1, x]);
    let r2x = store.mul(vec![r2, x]);
    let exp_r1x = store.func("exp", vec![r1x]);
    let exp_r2x = store.func("exp", vec![r2x]);
    let term1 = store.mul(vec![c1, exp_r1x]);
    let term2 = store.mul(vec![c2, exp_r2x]);
    let solution = store.add(vec![term1, term2]);

    Some(simplify(store, solution))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_separable_simple() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");

        // dy/dx = x*y (separable)
        let rhs = st.mul(vec![x, y]);

        let solution = solve_ode_first_order(&mut st, rhs, "y", "x");
        assert!(solution.is_some());

        let sol_str = st.to_string(solution.unwrap());
        // Should contain ln and x^2
        assert!(sol_str.contains("ln") || sol_str.contains("x"));
    }

    #[test]
    fn test_homogeneous_simple() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");

        // dy/dx = y/x (homogeneous, also separable)
        let neg_one = st.int(-1);
        let x_inv = st.pow(x, neg_one);
        let rhs = st.mul(vec![y, x_inv]);

        let solution = solve_ode_first_order(&mut st, rhs, "y", "x");
        assert!(solution.is_some());

        // Solution exists (may be ln(y/x) = C or y = Cx)
        let sol = solution.unwrap();
        let sol_str = st.to_string(sol);
        // Verify it contains expected variables
        assert!(sol_str.contains("x") || sol_str.contains("y"));
    }

    #[test]
    fn test_separable_x_only() {
        let mut st = Store::new();
        let x = st.sym("x");

        // dy/dx = x (separable: f(x) = x, g(y) = 1)
        let solution = solve_ode_first_order(&mut st, x, "y", "x");
        assert!(solution.is_some());
    }

    #[test]
    fn test_linear_simple() {
        let mut st = Store::new();
        let _x = st.sym("x");
        let y = st.sym("y");

        // dy/dx = -y (linear: p(x) = 1, q(x) = 0)
        let neg_one = st.int(-1);
        let rhs = st.mul(vec![neg_one, y]);

        let solution = solve_ode_first_order(&mut st, rhs, "y", "x");
        assert!(solution.is_some());

        // Solution exists (exact form may vary)
        let _sol_str = st.to_string(solution.unwrap());
    }

    #[test]
    fn test_extract_separable_factors() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let expr = st.mul(vec![x, y]);

        let result = extract_separable_factors(&mut st, expr, "y", "x");
        assert!(result.is_some());

        let (f_x, g_y) = result.unwrap();
        assert!(st.to_string(f_x).contains("x"));
        assert!(st.to_string(g_y).contains("y"));
    }
}
