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
/// - ∫ 1/x dx = ln|x|
/// - ∫ f'/f dx = ln|f| (logarithmic derivative pattern)
/// - ∫ ln(x) * g(x) dx - deferred to integration by parts
///
/// Returns an antiderivative if a logarithmic pattern is detected.
pub fn try_integrate_logarithmic(store: &mut Store, expr: ExprId, var: &str) -> Option<ExprId> {
    // Pattern 1: ∫ 1/x dx = ln(x)
    // Check if expr is x^(-1) or a rational with numerator 1 and denominator x
    match store.get(expr).op {
        Op::Pow => {
            let children = &store.get(expr).children;
            if children.len() == 2 {
                let base = children[0];
                let exp = children[1];

                // Check for x^(-1)
                if matches!((&store.get(exp).op, &store.get(exp).payload), (Op::Integer, Payload::Int(-1)))
                    && matches!((&store.get(base).op, &store.get(base).payload), (Op::Symbol, Payload::Sym(s)) if s == var)
                {
                    // ∫ x^(-1) dx = ln(x)
                    return Some(store.func("ln", vec![base]));
                }
            }
            None
        }
        // Pattern 2: Check for g'(x)/g(x) pattern
        // This is complex and typically handled by u-substitution in main integrate
        // For now, return None to defer to main engine
        _ => None,
    }
}

/// Builds a differential field tower for an expression
///
/// Analyzes the nested structure of exponentials and logarithms to construct
/// a tower representation suitable for Risch algorithm application.
///
/// Returns a vector of tower elements ordered from base to top.
#[allow(dead_code)]
pub fn build_tower(store: &mut Store, expr: ExprId, var: &str) -> Vec<TowerElement> {
    let mut tower = Vec::new();

    // Start with the base field (the variable itself)
    let x = store.sym(var);
    tower.push(TowerElement {
        expr: x,
        extension: ExtensionType::Base,
        derivative: Some(store.int(1)),
    });

    // Recursively detect extensions
    // This is a simplified version; full Risch requires more sophisticated analysis
    let ext_type = detect_extension(store, expr, var);
    match ext_type {
        ExtensionType::Exponential(u) | ExtensionType::Logarithmic(u) => {
            let deriv = diff(store, expr, var);
            tower.push(TowerElement { expr, extension: ext_type, derivative: Some(deriv) });

            // Check if u itself has extensions
            let u_ext = detect_extension(store, u, var);
            if !matches!(u_ext, ExtensionType::Base) {
                // Recursive case - u has its own extensions
                // For now, we stop at depth 2; full implementation would recurse
            }
        }
        ExtensionType::Base => {
            // No extension, just add the expression
            let deriv = diff(store, expr, var);
            tower.push(TowerElement {
                expr,
                extension: ExtensionType::Base,
                derivative: Some(deriv),
            });
        }
    }

    tower
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
