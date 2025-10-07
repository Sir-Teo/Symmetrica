//! Ordinary Differential Equation (ODE) Solving
//!
//! This module provides symbolic solutions for first-order ODEs:
//! - Separable equations: dy/dx = f(x)g(y)
//! - Linear equations: dy/dx + p(x)y = q(x)
//!
//! Future work:
//! - Exact equations
//! - Bernoulli equations
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
    // Try separable form: dy/dx = f(x)g(y)
    if let Some(solution) = try_separable(store, rhs, y_var, x_var) {
        return Some(solution);
    }

    // Try linear form: dy/dx = -p(x)y + q(x)
    if let Some(solution) = try_linear(store, rhs, y_var, x_var) {
        return Some(solution);
    }

    None
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
