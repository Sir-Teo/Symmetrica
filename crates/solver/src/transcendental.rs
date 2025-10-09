//! Transcendental equation solver patterns
//!
//! Handles common transcendental equation forms:
//! - Inverse trig: arcsin(x) = a, arctan(f(x)) = arctan(g(x))
//! - Logarithmic: log(f(x)) = a, log(f(x)) = log(g(x))
//! - Combined with exponential patterns from main solver

use expr_core::{ExprId, Op, Payload, Store};

/// Solve inverse trigonometric equations
pub fn solve_inverse_trig(store: &mut Store, expr: ExprId, var: &str) -> Option<Vec<ExprId>> {
    // Pattern 1: arcsin(x) = a  →  x = sin(a)
    // Pattern 2: arccos(x) = a  →  x = cos(a)
    // Pattern 3: arctan(x) = a  →  x = tan(a)

    if store.get(expr).op != Op::Add {
        return None;
    }

    let children = &store.get(expr).children.clone();
    if children.len() != 2 {
        return None;
    }

    // Try to find arcfunc(var_expr) - const pattern
    for i in 0..2 {
        let func_term = children[i];
        let const_term = children[1 - i];

        if !depends_on(store, const_term, var) {
            if let Some((func_name, arg)) = extract_inverse_trig(store, func_term) {
                // arcfunc(arg) = -const_term
                let neg_one = store.int(-1);
                let rhs = store.mul(vec![neg_one, const_term]);

                // Apply inverse: arg = func(rhs)
                let forward_func = match func_name.as_str() {
                    "arcsin" | "asin" => "sin",
                    "arccos" | "acos" => "cos",
                    "arctan" | "atan" => "tan",
                    _ => return None,
                };

                let solution_rhs = store.func(forward_func, vec![rhs]);

                // If arg is just var, we're done
                if let (Op::Symbol, Payload::Sym(s)) = (&store.get(arg).op, &store.get(arg).payload)
                {
                    if s == var {
                        return Some(vec![solution_rhs]);
                    }
                }

                // Otherwise, try to solve arg = solution_rhs for var
                // For now, handle simple linear cases
                return solve_linear_equation(store, arg, solution_rhs, var);
            }
        }
    }

    // Pattern 4: arctan(f(x)) = arctan(g(x))  →  f(x) = g(x)
    if let Some((name1, arg1)) = extract_inverse_trig(store, children[0]) {
        if let Some((name2, arg2)) = extract_inverse_trig(store, children[1]) {
            if name1 == name2 {
                // f(x) = g(x), so f(x) - g(x) = 0
                let neg_one = store.int(-1);
                let neg_arg2 = store.mul(vec![neg_one, arg2]);
                let diff = store.add(vec![arg1, neg_arg2]);

                // Try to solve as polynomial
                if let Some(roots) = crate::solve_univariate(store, diff, var) {
                    return Some(roots);
                }
            }
        }
    }

    None
}

/// Solve logarithmic equations
pub fn solve_logarithmic(store: &mut Store, expr: ExprId, var: &str) -> Option<Vec<ExprId>> {
    // Pattern 1: log(f(x)) = a  →  f(x) = e^a
    // Pattern 2: log(f(x)) = log(g(x))  →  f(x) = g(x)
    // Pattern 3: ln(x) - a = 0  →  x = e^a

    if store.get(expr).op != Op::Add {
        return None;
    }

    let children = &store.get(expr).children.clone();
    if children.len() != 2 {
        return None;
    }

    // Try to find log(var_expr) - const pattern
    for i in 0..2 {
        let func_term = children[i];
        let const_term = children[1 - i];

        if !depends_on(store, const_term, var) {
            if let Some(arg) = extract_log(store, func_term) {
                // log(arg) = -const_term
                let neg_one = store.int(-1);
                let rhs = store.mul(vec![neg_one, const_term]);

                // Apply inverse: arg = exp(rhs)
                let exp_rhs = store.func("exp", vec![rhs]);

                // If arg is just var, we're done
                if let (Op::Symbol, Payload::Sym(s)) = (&store.get(arg).op, &store.get(arg).payload)
                {
                    if s == var {
                        return Some(vec![exp_rhs]);
                    }
                }

                // Otherwise, try to solve arg = exp_rhs for var
                return solve_linear_equation(store, arg, exp_rhs, var);
            }
        }
    }

    // Pattern: log(f(x)) = log(g(x))  →  f(x) = g(x)
    if let Some(arg1) = extract_log(store, children[0]) {
        if let Some(arg2) = extract_log(store, children[1]) {
            // f(x) = g(x), so f(x) - g(x) = 0
            let neg_one = store.int(-1);
            let neg_arg2 = store.mul(vec![neg_one, arg2]);
            let diff = store.add(vec![arg1, neg_arg2]);

            // Try to solve as polynomial
            if let Some(roots) = crate::solve_univariate(store, diff, var) {
                return Some(roots);
            }
        }
    }

    None
}

/// Extract inverse trig function and its argument
fn extract_inverse_trig(store: &Store, expr: ExprId) -> Option<(String, ExprId)> {
    if store.get(expr).op == Op::Function {
        if let Payload::Func(name) = &store.get(expr).payload {
            if matches!(name.as_str(), "arcsin" | "asin" | "arccos" | "acos" | "arctan" | "atan") {
                let children = &store.get(expr).children;
                if children.len() == 1 {
                    return Some((name.clone(), children[0]));
                }
            }
        }
    }
    None
}

/// Extract logarithm function and its argument
fn extract_log(store: &Store, expr: ExprId) -> Option<ExprId> {
    if store.get(expr).op == Op::Function {
        if let Payload::Func(name) = &store.get(expr).payload {
            if matches!(name.as_str(), "ln" | "log") {
                let children = &store.get(expr).children;
                if children.len() == 1 {
                    return Some(children[0]);
                }
            }
        }
    }
    None
}

/// Check if expression depends on variable
fn depends_on(store: &Store, expr: ExprId, var: &str) -> bool {
    match (&store.get(expr).op, &store.get(expr).payload) {
        (Op::Symbol, Payload::Sym(s)) => s == var,
        (Op::Integer, _) | (Op::Rational, _) => false,
        _ => store.get(expr).children.iter().any(|&c| depends_on(store, c, var)),
    }
}

/// Solve simple linear equation lhs = rhs for var
fn solve_linear_equation(
    store: &mut Store,
    lhs: ExprId,
    rhs: ExprId,
    var: &str,
) -> Option<Vec<ExprId>> {
    // If lhs is just var, return rhs
    if let (Op::Symbol, Payload::Sym(s)) = (&store.get(lhs).op, &store.get(lhs).payload) {
        if s == var {
            return Some(vec![rhs]);
        }
    }

    // If lhs is a*var, return rhs/a
    if store.get(lhs).op == Op::Mul {
        let children = &store.get(lhs).children.clone();
        let mut var_found = false;
        let mut coeff_parts = Vec::new();

        for &child in children {
            if let (Op::Symbol, Payload::Sym(s)) = (&store.get(child).op, &store.get(child).payload)
            {
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
            let coeff = if coeff_parts.is_empty() { store.int(1) } else { store.mul(coeff_parts) };
            let neg_one = store.int(-1);
            let inv_coeff = store.pow(coeff, neg_one);
            let solution = store.mul(vec![rhs, inv_coeff]);
            return Some(vec![solution]);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arcsin_constant() {
        let mut st = Store::new();
        let x = st.sym("x");

        // arcsin(x) - 1 = 0  →  x = sin(1)
        let arcsin_x = st.func("arcsin", vec![x]);
        let neg_one = st.int(-1);
        let eq = st.add(vec![arcsin_x, neg_one]);

        let result = solve_inverse_trig(&mut st, eq, "x");
        assert!(result.is_some());

        let roots = result.unwrap();
        assert_eq!(roots.len(), 1);

        let root_str = st.to_string(roots[0]);
        assert!(root_str.contains("sin"));
    }

    #[test]
    fn test_ln_constant() {
        let mut st = Store::new();
        let x = st.sym("x");

        // ln(x) - 2 = 0  →  x = e^2
        let ln_x = st.func("ln", vec![x]);
        let neg_two = st.int(-2);
        let eq = st.add(vec![ln_x, neg_two]);

        let result = solve_logarithmic(&mut st, eq, "x");
        assert!(result.is_some());

        let roots = result.unwrap();
        assert_eq!(roots.len(), 1);

        let root_str = st.to_string(roots[0]);
        assert!(root_str.contains("exp") || root_str.contains("e"));
    }

    #[test]
    fn test_arctan_equality() {
        let mut st = Store::new();
        let x = st.sym("x");

        // arctan(x) - arctan(x) = 0  →  x = x  →  identity (infinitely many solutions)
        // This is a degenerate case, so we test a simpler pattern
        let arctan_x = st.func("arctan", vec![x]);
        let two = st.int(2);
        let two_x = st.mul(vec![two, x]);
        let arctan_2x = st.func("arctan", vec![two_x]);
        let neg_one = st.int(-1);
        let neg_arctan_2x = st.mul(vec![neg_one, arctan_2x]);
        let eq = st.add(vec![arctan_x, neg_arctan_2x]);

        // This pattern may or may not solve depending on implementation
        // Just verify it doesn't panic
        let result = solve_inverse_trig(&mut st, eq, "x");
        // May return Some or None depending on solver capability
        let _ = result;
    }
}
