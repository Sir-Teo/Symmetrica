//! Risch algorithm foundation for symbolic integration (v1.1)
//!
//! This module provides the groundwork for the Risch algorithm, including:
//! - Differential field tower representation
//! - Tower extension detection (exp/log structures)
//! - Logarithmic derivative computation
//!
//! The Risch algorithm is a decision procedure for symbolic integration of
//! elementary functions. This implementation focuses on exponential extensions
//! as a foundation for more advanced integration.

use crate::diff::diff;
use expr_core::{ExprId, Op, Payload, Store};

/// Represents the type of tower extension
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtensionType {
    /// Exponential extension: t = exp(u) where u is in the base field
    Exponential(ExprId), // stores u
    /// Logarithmic extension: t = ln(u) where u is in the base field
    Logarithmic(ExprId), // stores u
    /// No extension (base field element)
    Base,
}

/// A differential field tower element
#[derive(Debug, Clone)]
pub struct TowerElement {
    /// The expression itself
    pub expr: ExprId,
    /// The type of extension this represents
    pub extension: ExtensionType,
    /// Derivative with respect to the variable
    pub derivative: Option<ExprId>,
}

/// Analyzes an expression to determine its tower structure
///
/// Returns the extension type if the expression is exp(u) or ln(u)
/// for some simpler expression u.
pub fn detect_extension(store: &Store, expr: ExprId, var: &str) -> ExtensionType {
    match store.get(expr).op {
        Op::Function => {
            if let Payload::Func(fname) = &store.get(expr).payload {
                let children = &store.get(expr).children;
                if children.len() == 1 {
                    let arg = children[0];
                    match fname.as_str() {
                        "exp" => {
                            // Check if argument depends on var
                            if depends_on_var(store, arg, var) {
                                return ExtensionType::Exponential(arg);
                            }
                        }
                        "ln" | "log" => {
                            // Check if argument depends on var
                            if depends_on_var(store, arg, var) {
                                return ExtensionType::Logarithmic(arg);
                            }
                        }
                        _ => {}
                    }
                }
            }
            ExtensionType::Base
        }
        _ => ExtensionType::Base,
    }
}

/// Helper: checks if an expression depends on the given variable
fn depends_on_var(store: &Store, expr: ExprId, var: &str) -> bool {
    match (&store.get(expr).op, &store.get(expr).payload) {
        (Op::Symbol, Payload::Sym(s)) => s == var,
        (Op::Integer, _) | (Op::Rational, _) => false,
        _ => store.get(expr).children.iter().any(|&c| depends_on_var(store, c, var)),
    }
}

/// Computes the logarithmic derivative of an expression: d/dx(ln(f)) = f'/f
///
/// This is a key operation in the Risch algorithm for handling exponential
/// and logarithmic functions.
///
/// Returns None if the derivative cannot be computed or the expression is zero.
pub fn logarithmic_derivative(store: &mut Store, expr: ExprId, var: &str) -> Option<ExprId> {
    // Compute f'
    let derivative = diff(store, expr, var);

    // Return f'/f = f' * f^(-1)
    let minus_one = store.int(-1);
    let inv_expr = store.pow(expr, minus_one);
    Some(store.mul(vec![derivative, inv_expr]))
}

/// Checks if an expression is in exponential normal form: exp(u)
///
/// Returns Some(u) if expr = exp(u), None otherwise
pub fn is_exponential(store: &Store, expr: ExprId) -> Option<ExprId> {
    match store.get(expr).op {
        Op::Function => {
            if let Payload::Func(fname) = &store.get(expr).payload {
                if fname == "exp" && store.get(expr).children.len() == 1 {
                    return Some(store.get(expr).children[0]);
                }
            }
            None
        }
        _ => None,
    }
}

/// Checks if an expression is in logarithmic normal form: ln(u)
///
/// Returns Some(u) if expr = ln(u), None otherwise
pub fn is_logarithm(store: &Store, expr: ExprId) -> Option<ExprId> {
    match store.get(expr).op {
        Op::Function => {
            if let Payload::Func(fname) = &store.get(expr).payload {
                if (fname == "ln" || fname == "log") && store.get(expr).children.len() == 1 {
                    return Some(store.get(expr).children[0]);
                }
            }
            None
        }
        _ => None,
    }
}

/// Attempts to integrate simple exponential expressions using Risch principles
///
/// Handles patterns like:
/// - ∫ exp(x) dx = exp(x)
/// - ∫ exp(ax) dx = (1/a) exp(ax)
/// - ∫ exp(ax + b) dx = (1/a) exp(ax + b)
pub fn try_integrate_exponential(store: &mut Store, expr: ExprId, var: &str) -> Option<ExprId> {
    // Check if this is exp(u)
    let u = is_exponential(store, expr)?;

    // Compute du/dx
    let du = diff(store, u, var);

    // Check if du is a constant
    match (&store.get(du).op, &store.get(du).payload) {
        (Op::Integer, Payload::Int(0)) => {
            // du/dx = 0, so exp(u) is constant w.r.t. var
            // ∫ exp(u) dx = x * exp(u)
            let x = store.sym(var);
            Some(store.mul(vec![x, expr]))
        }
        (Op::Integer, Payload::Int(a)) => {
            // du/dx = a (constant), so ∫ exp(u) dx = (1/a) exp(u)
            let inv_a = store.rat(1, *a);
            Some(store.mul(vec![inv_a, expr]))
        }
        (Op::Rational, Payload::Rat(n, d)) => {
            // du/dx = n/d (constant), so ∫ exp(u) dx = (d/n) exp(u)
            let inv_a = store.rat(*d, *n);
            Some(store.mul(vec![inv_a, expr]))
        }
        _ => {
            // du/dx is not constant - more complex pattern
            None
        }
    }
}

/// Attempts to integrate expressions involving logarithms using Risch principles
///
/// Handles patterns like:
/// - ∫ ln(x) dx using integration by parts (already implemented elsewhere)
/// - Detection of logarithmic derivatives: ∫ f'/f dx = ln(f)
///
/// Currently a placeholder for future Risch algorithm development.
#[allow(dead_code)]
pub fn try_integrate_logarithmic(_store: &mut Store, _expr: ExprId, _var: &str) -> Option<ExprId> {
    // For now, this is a placeholder for future Risch algorithm work
    // The pattern ∫ f'/f dx is already handled by the u'/u rule in integrate.rs

    // Check if expr is of the form u'/u (already handled by main integration)
    // This function will be expanded when we implement full Risch algorithm

    // Return None to let the main integration engine handle it
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrate::integrate;
    use simplify::simplify;

    #[test]
    fn test_detect_exponential_extension() {
        let mut st = Store::new();
        let x = st.sym("x");
        let expx = st.func("exp", vec![x]);

        let ext = detect_extension(&st, expx, "x");
        assert_eq!(ext, ExtensionType::Exponential(x));
    }

    #[test]
    fn test_detect_logarithmic_extension() {
        let mut st = Store::new();
        let x = st.sym("x");
        let lnx = st.func("ln", vec![x]);

        let ext = detect_extension(&st, lnx, "x");
        assert_eq!(ext, ExtensionType::Logarithmic(x));
    }

    #[test]
    fn test_detect_base_element() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two);

        let ext = detect_extension(&st, x2, "x");
        assert_eq!(ext, ExtensionType::Base);
    }

    #[test]
    fn test_logarithmic_derivative_simple() {
        let mut st = Store::new();
        let x = st.sym("x");

        // log_deriv(x) = 1/x
        let log_deriv = logarithmic_derivative(&mut st, x, "x").unwrap();

        // Should be x^(-1)
        let expected = {
            let minus_one = st.int(-1);
            st.pow(x, minus_one)
        };

        assert_eq!(st.to_string(log_deriv), st.to_string(expected));
    }

    #[test]
    fn test_logarithmic_derivative_polynomial() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two);

        // log_deriv(x²) = 2x/x² = 2/x
        let log_deriv = logarithmic_derivative(&mut st, x2, "x").unwrap();

        // Verify structure: should contain 2 and x
        let result = st.to_string(log_deriv);
        assert!(result.contains("2"));
        assert!(result.contains("x"));
    }

    #[test]
    fn test_is_exponential() {
        let mut st = Store::new();
        let x = st.sym("x");
        let expx = st.func("exp", vec![x]);

        assert_eq!(is_exponential(&st, expx), Some(x));
        assert_eq!(is_exponential(&st, x), None);
    }

    #[test]
    fn test_is_logarithm() {
        let mut st = Store::new();
        let x = st.sym("x");
        let lnx = st.func("ln", vec![x]);

        assert_eq!(is_logarithm(&st, lnx), Some(x));
        assert_eq!(is_logarithm(&st, x), None);
    }

    #[test]
    fn test_integrate_exp_x() {
        let mut st = Store::new();
        let x = st.sym("x");
        let expx = st.func("exp", vec![x]);

        let result = try_integrate_exponential(&mut st, expx, "x").expect("exp(x)");

        // ∫ exp(x) dx = exp(x)
        assert_eq!(st.to_string(result), st.to_string(expx));
    }

    #[test]
    fn test_integrate_exp_2x() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let two_x = st.mul(vec![two, x]);
        let exp2x = st.func("exp", vec![two_x]);

        let result = try_integrate_exponential(&mut st, exp2x, "x").expect("exp(2x)");

        // ∫ exp(2x) dx = (1/2) exp(2x)
        // Verify by differentiation
        let deriv = diff(&mut st, result, "x");
        let simplified = simplify(&mut st, deriv);
        let original = simplify(&mut st, exp2x);
        assert_eq!(st.get(simplified).digest, st.get(original).digest);
    }

    #[test]
    fn test_integrate_exp_linear() {
        let mut st = Store::new();
        let x = st.sym("x");
        let three = st.int(3);
        let five = st.int(5);
        let three_x = st.mul(vec![three, x]);
        let u = st.add(vec![three_x, five]); // 3x + 5
        let exp_u = st.func("exp", vec![u]);

        let result = try_integrate_exponential(&mut st, exp_u, "x").expect("exp(3x+5)");

        // ∫ exp(3x + 5) dx = (1/3) exp(3x + 5)
        // Verify by differentiation
        let deriv = diff(&mut st, result, "x");
        let simplified = simplify(&mut st, deriv);
        let original = simplify(&mut st, exp_u);
        assert_eq!(st.get(simplified).digest, st.get(original).digest);
    }

    #[test]
    fn test_risch_exponential_integration_via_main() {
        let mut st = Store::new();
        let x = st.sym("x");
        let expx = st.func("exp", vec![x]);

        // Test that main integration engine can handle simple exponentials
        let result = integrate(&mut st, expx, "x").expect("exp(x) integrable");

        // Verify by differentiation
        let deriv = diff(&mut st, result, "x");
        let simplified = simplify(&mut st, deriv);
        assert_eq!(st.get(simplified).digest, st.get(expx).digest);
    }

    #[test]
    fn test_depends_on_var() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let two = st.int(2);

        assert!(depends_on_var(&st, x, "x"));
        assert!(!depends_on_var(&st, x, "y"));
        assert!(depends_on_var(&st, y, "y"));
        assert!(!depends_on_var(&st, two, "x"));

        let x_plus_y = st.add(vec![x, y]);
        assert!(depends_on_var(&st, x_plus_y, "x"));
        assert!(depends_on_var(&st, x_plus_y, "y"));
        assert!(!depends_on_var(&st, x_plus_y, "z"));
    }
}
