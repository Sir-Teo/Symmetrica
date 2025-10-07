//! evalf: Numeric evaluation of symbolic expressions
//!
//! This module provides arbitrary-precision floating-point evaluation of symbolic expressions.
//! For now, uses f64 for simplicity. Future versions can add MPFR via feature flags.

#![deny(warnings)]

use expr_core::{ExprId, Op, Payload, Store};
use std::collections::HashMap;

/// Evaluation context holding variable bindings
#[derive(Default, Clone, Debug)]
pub struct EvalContext {
    bindings: HashMap<String, f64>,
}

impl EvalContext {
    /// Create a new empty evaluation context
    pub fn new() -> Self {
        Self::default()
    }

    /// Bind a variable to a numeric value
    pub fn bind<S: Into<String>>(&mut self, name: S, value: f64) -> &mut Self {
        self.bindings.insert(name.into(), value);
        self
    }

    /// Get the value of a bound variable
    pub fn get(&self, name: &str) -> Option<f64> {
        self.bindings.get(name).copied()
    }

    /// Clear all bindings
    pub fn clear(&mut self) {
        self.bindings.clear();
    }
}

/// Error type for evaluation failures
#[derive(Debug, Clone, PartialEq)]
pub enum EvalError {
    /// Unbound variable
    UnboundVariable(String),
    /// Unknown function
    UnknownFunction(String),
    /// Domain error (e.g., ln of negative number)
    DomainError(String),
    /// Result is non-finite (infinity or NaN)
    NonFinite,
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::UnboundVariable(name) => write!(f, "unbound variable: {}", name),
            EvalError::UnknownFunction(name) => write!(f, "unknown function: {}", name),
            EvalError::DomainError(msg) => write!(f, "domain error: {}", msg),
            EvalError::NonFinite => write!(f, "result is non-finite"),
        }
    }
}

impl std::error::Error for EvalError {}

/// Evaluate an expression to a floating-point number
pub fn eval(store: &Store, id: ExprId, ctx: &EvalContext) -> Result<f64, EvalError> {
    let result = eval_recursive(store, id, ctx)?;

    if result.is_finite() {
        Ok(result)
    } else {
        Err(EvalError::NonFinite)
    }
}

/// Evaluate with a single variable binding (convenience function)
pub fn eval_at(store: &Store, id: ExprId, var: &str, value: f64) -> Result<f64, EvalError> {
    let mut ctx = EvalContext::new();
    ctx.bind(var, value);
    eval(store, id, &ctx)
}

fn eval_recursive(store: &Store, id: ExprId, ctx: &EvalContext) -> Result<f64, EvalError> {
    let node = store.get(id);

    match (&node.op, &node.payload) {
        // Constants
        (Op::Integer, Payload::Int(k)) => Ok(*k as f64),

        (Op::Rational, Payload::Rat(num, den)) => Ok((*num as f64) / (*den as f64)),

        // Symbols
        (Op::Symbol, Payload::Sym(name)) => {
            ctx.get(name).ok_or_else(|| EvalError::UnboundVariable(name.clone()))
        }

        // Addition
        (Op::Add, _) => {
            let mut sum = 0.0;
            for &child in &node.children {
                sum += eval_recursive(store, child, ctx)?;
            }
            Ok(sum)
        }

        // Multiplication
        (Op::Mul, _) => {
            let mut product = 1.0;
            for &child in &node.children {
                product *= eval_recursive(store, child, ctx)?;
            }
            Ok(product)
        }

        // Power
        (Op::Pow, _) => {
            let base = eval_recursive(store, node.children[0], ctx)?;
            let exponent = eval_recursive(store, node.children[1], ctx)?;
            Ok(base.powf(exponent))
        }

        // Functions
        (Op::Function, Payload::Func(name)) => eval_function(store, name, &node.children, ctx),

        _ => Err(EvalError::DomainError(format!("cannot evaluate {:?}", node.op))),
    }
}

fn eval_function(
    store: &Store,
    name: &str,
    args: &[ExprId],
    ctx: &EvalContext,
) -> Result<f64, EvalError> {
    match name {
        // Trigonometric functions
        "sin" => {
            check_arity(name, args, 1)?;
            let x = eval_recursive(store, args[0], ctx)?;
            Ok(x.sin())
        }

        "cos" => {
            check_arity(name, args, 1)?;
            let x = eval_recursive(store, args[0], ctx)?;
            Ok(x.cos())
        }

        "tan" => {
            check_arity(name, args, 1)?;
            let x = eval_recursive(store, args[0], ctx)?;
            Ok(x.tan())
        }

        // Inverse trigonometric
        "asin" | "arcsin" => {
            check_arity(name, args, 1)?;
            let x = eval_recursive(store, args[0], ctx)?;
            if !(-1.0..=1.0).contains(&x) {
                return Err(EvalError::DomainError(format!(
                    "asin requires -1 <= x <= 1, got {}",
                    x
                )));
            }
            Ok(x.asin())
        }

        "acos" | "arccos" => {
            check_arity(name, args, 1)?;
            let x = eval_recursive(store, args[0], ctx)?;
            if !(-1.0..=1.0).contains(&x) {
                return Err(EvalError::DomainError(format!(
                    "acos requires -1 <= x <= 1, got {}",
                    x
                )));
            }
            Ok(x.acos())
        }

        "atan" | "arctan" => {
            check_arity(name, args, 1)?;
            let x = eval_recursive(store, args[0], ctx)?;
            Ok(x.atan())
        }

        // Hyperbolic functions
        "sinh" => {
            check_arity(name, args, 1)?;
            let x = eval_recursive(store, args[0], ctx)?;
            Ok(x.sinh())
        }

        "cosh" => {
            check_arity(name, args, 1)?;
            let x = eval_recursive(store, args[0], ctx)?;
            Ok(x.cosh())
        }

        "tanh" => {
            check_arity(name, args, 1)?;
            let x = eval_recursive(store, args[0], ctx)?;
            Ok(x.tanh())
        }

        // Exponential and logarithmic
        "exp" => {
            check_arity(name, args, 1)?;
            let x = eval_recursive(store, args[0], ctx)?;
            Ok(x.exp())
        }

        "ln" | "log" => {
            check_arity(name, args, 1)?;
            let x = eval_recursive(store, args[0], ctx)?;
            if x <= 0.0 {
                return Err(EvalError::DomainError(format!("ln requires x > 0, got {}", x)));
            }
            Ok(x.ln())
        }

        "log10" => {
            check_arity(name, args, 1)?;
            let x = eval_recursive(store, args[0], ctx)?;
            if x <= 0.0 {
                return Err(EvalError::DomainError(format!("log10 requires x > 0, got {}", x)));
            }
            Ok(x.log10())
        }

        "log2" => {
            check_arity(name, args, 1)?;
            let x = eval_recursive(store, args[0], ctx)?;
            if x <= 0.0 {
                return Err(EvalError::DomainError(format!("log2 requires x > 0, got {}", x)));
            }
            Ok(x.log2())
        }

        // Other functions
        "sqrt" => {
            check_arity(name, args, 1)?;
            let x = eval_recursive(store, args[0], ctx)?;
            if x < 0.0 {
                return Err(EvalError::DomainError(format!("sqrt requires x >= 0, got {}", x)));
            }
            Ok(x.sqrt())
        }

        "abs" => {
            check_arity(name, args, 1)?;
            let x = eval_recursive(store, args[0], ctx)?;
            Ok(x.abs())
        }

        "floor" => {
            check_arity(name, args, 1)?;
            let x = eval_recursive(store, args[0], ctx)?;
            Ok(x.floor())
        }

        "ceil" => {
            check_arity(name, args, 1)?;
            let x = eval_recursive(store, args[0], ctx)?;
            Ok(x.ceil())
        }

        "round" => {
            check_arity(name, args, 1)?;
            let x = eval_recursive(store, args[0], ctx)?;
            Ok(x.round())
        }

        // Two-argument functions
        "atan2" | "arctan2" => {
            check_arity(name, args, 2)?;
            let y = eval_recursive(store, args[0], ctx)?;
            let x = eval_recursive(store, args[1], ctx)?;
            Ok(y.atan2(x))
        }

        "min" => {
            if args.len() < 2 {
                return Err(EvalError::DomainError(format!(
                    "min requires at least 2 arguments, got {}",
                    args.len()
                )));
            }
            let mut min_val = eval_recursive(store, args[0], ctx)?;
            for &arg in &args[1..] {
                let val = eval_recursive(store, arg, ctx)?;
                if val < min_val {
                    min_val = val;
                }
            }
            Ok(min_val)
        }

        "max" => {
            if args.len() < 2 {
                return Err(EvalError::DomainError(format!(
                    "max requires at least 2 arguments, got {}",
                    args.len()
                )));
            }
            let mut max_val = eval_recursive(store, args[0], ctx)?;
            for &arg in &args[1..] {
                let val = eval_recursive(store, arg, ctx)?;
                if val > max_val {
                    max_val = val;
                }
            }
            Ok(max_val)
        }

        // Special functions (Phase 3)
        "Gamma" => {
            check_arity(name, args, 1)?;
            let x = eval_recursive(store, args[0], ctx)?;
            let gamma_func = special::gamma::GammaFunction;
            special::SpecialFunction::eval(&gamma_func, &[x])
                .ok_or_else(|| EvalError::DomainError(format!("Gamma({}) not computable", x)))
        }

        "erf" => {
            check_arity(name, args, 1)?;
            let x = eval_recursive(store, args[0], ctx)?;
            let erf_func = special::erf::ErfFunction;
            special::SpecialFunction::eval(&erf_func, &[x])
                .ok_or_else(|| EvalError::DomainError(format!("erf({}) not computable", x)))
        }

        "Ei" => {
            check_arity(name, args, 1)?;
            let x = eval_recursive(store, args[0], ctx)?;
            let ei_func = special::expint::EiFunction;
            special::SpecialFunction::eval(&ei_func, &[x])
                .ok_or_else(|| EvalError::DomainError(format!("Ei({}) not computable", x)))
        }

        _ => Err(EvalError::UnknownFunction(name.to_string())),
    }
}

fn check_arity(name: &str, args: &[ExprId], expected: usize) -> Result<(), EvalError> {
    if args.len() != expected {
        Err(EvalError::DomainError(format!(
            "{} requires {} argument(s), got {}",
            name,
            expected,
            args.len()
        )))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expr_core::Store;

    #[test]
    fn eval_integer() {
        let mut st = Store::new();
        let five = st.int(5);
        let ctx = EvalContext::new();
        assert_eq!(eval(&st, five, &ctx).unwrap(), 5.0);
    }

    #[test]
    fn eval_rational() {
        let mut st = Store::new();
        let half = st.rat(1, 2);
        let ctx = EvalContext::new();
        assert_eq!(eval(&st, half, &ctx).unwrap(), 0.5);
    }

    #[test]
    fn eval_symbol_bound() {
        let mut st = Store::new();
        let x = st.sym("x");
        let mut ctx = EvalContext::new();
        ctx.bind("x", 3.0);
        assert_eq!(eval(&st, x, &ctx).unwrap(), 3.0);
    }

    #[test]
    fn eval_symbol_unbound() {
        let mut st = Store::new();
        let x = st.sym("x");
        let ctx = EvalContext::new();
        assert!(matches!(eval(&st, x, &ctx), Err(EvalError::UnboundVariable(_))));
    }

    #[test]
    fn eval_addition() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let three = st.int(3);
        let expr = st.add(vec![x, two, three]);
        let mut ctx = EvalContext::new();
        ctx.bind("x", 5.0);
        assert_eq!(eval(&st, expr, &ctx).unwrap(), 10.0);
    }

    #[test]
    fn eval_multiplication() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let three = st.int(3);
        let expr = st.mul(vec![two, x, three]);
        let mut ctx = EvalContext::new();
        ctx.bind("x", 4.0);
        assert_eq!(eval(&st, expr, &ctx).unwrap(), 24.0);
    }

    #[test]
    fn eval_power() {
        let mut st = Store::new();
        let x = st.sym("x");
        let three = st.int(3);
        let expr = st.pow(x, three);
        let mut ctx = EvalContext::new();
        ctx.bind("x", 2.0);
        assert_eq!(eval(&st, expr, &ctx).unwrap(), 8.0);
    }

    #[test]
    fn eval_sin() {
        let mut st = Store::new();
        let x = st.sym("x");
        let expr = st.func("sin", vec![x]);
        let mut ctx = EvalContext::new();
        ctx.bind("x", std::f64::consts::PI / 2.0);
        let result = eval(&st, expr, &ctx).unwrap();
        assert!((result - 1.0).abs() < 1e-10);
    }

    #[test]
    fn eval_cos() {
        let mut st = Store::new();
        let x = st.sym("x");
        let expr = st.func("cos", vec![x]);
        let mut ctx = EvalContext::new();
        ctx.bind("x", 0.0);
        assert_eq!(eval(&st, expr, &ctx).unwrap(), 1.0);
    }

    #[test]
    fn eval_exp() {
        let mut st = Store::new();
        let x = st.sym("x");
        let expr = st.func("exp", vec![x]);
        let mut ctx = EvalContext::new();
        ctx.bind("x", 1.0);
        let result = eval(&st, expr, &ctx).unwrap();
        assert!((result - std::f64::consts::E).abs() < 1e-10);
    }

    #[test]
    fn eval_ln() {
        let mut st = Store::new();
        let x = st.sym("x");
        let expr = st.func("ln", vec![x]);
        let mut ctx = EvalContext::new();
        ctx.bind("x", std::f64::consts::E);
        let result = eval(&st, expr, &ctx).unwrap();
        assert!((result - 1.0).abs() < 1e-10);
    }

    #[test]
    fn eval_ln_negative() {
        let mut st = Store::new();
        let neg_one = st.int(-1);
        let expr = st.func("ln", vec![neg_one]);
        let ctx = EvalContext::new();
        assert!(matches!(eval(&st, expr, &ctx), Err(EvalError::DomainError(_))));
    }

    #[test]
    fn eval_sqrt() {
        let mut st = Store::new();
        let four = st.int(4);
        let expr = st.func("sqrt", vec![four]);
        let ctx = EvalContext::new();
        assert_eq!(eval(&st, expr, &ctx).unwrap(), 2.0);
    }

    #[test]
    fn eval_sqrt_negative() {
        let mut st = Store::new();
        let neg_one = st.int(-1);
        let expr = st.func("sqrt", vec![neg_one]);
        let ctx = EvalContext::new();
        assert!(matches!(eval(&st, expr, &ctx), Err(EvalError::DomainError(_))));
    }

    #[test]
    fn eval_abs() {
        let mut st = Store::new();
        let neg_five = st.int(-5);
        let expr = st.func("abs", vec![neg_five]);
        let ctx = EvalContext::new();
        assert_eq!(eval(&st, expr, &ctx).unwrap(), 5.0);
    }

    #[test]
    fn eval_complex_expression() {
        let mut st = Store::new();
        let x = st.sym("x");
        // (x^2 + 2*x + 1) / (x + 1) at x=3
        let two = st.int(2);
        let one = st.int(1);
        let neg_one = st.int(-1);
        let x2 = st.pow(x, two);
        let two_x = st.mul(vec![two, x]);
        let num = st.add(vec![x2, two_x, one]);
        let xp1 = st.add(vec![x, one]);
        let inv = st.pow(xp1, neg_one);
        let expr = st.mul(vec![num, inv]);

        let mut ctx = EvalContext::new();
        ctx.bind("x", 3.0);
        let result = eval(&st, expr, &ctx).unwrap();
        // (9 + 6 + 1) / 4 = 16 / 4 = 4
        assert_eq!(result, 4.0);
    }

    #[test]
    fn eval_at_convenience() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let expr = st.pow(x, two);
        assert_eq!(eval_at(&st, expr, "x", 5.0).unwrap(), 25.0);
    }

    #[test]
    fn eval_min_max() {
        let mut st = Store::new();
        let three = st.int(3);
        let one = st.int(1);
        let two = st.int(2);
        let min_expr = st.func("min", vec![three, one, two]);
        let max_expr = st.func("max", vec![three, one, two]);
        let ctx = EvalContext::new();

        assert_eq!(eval(&st, min_expr, &ctx).unwrap(), 1.0);
        assert_eq!(eval(&st, max_expr, &ctx).unwrap(), 3.0);
    }

    #[test]
    fn eval_trig_functions() {
        let mut st = Store::new();
        let x = st.sym("x");

        let tan_expr = st.func("tan", vec![x]);
        let mut ctx = EvalContext::new();
        ctx.bind("x", std::f64::consts::PI / 4.0);
        let result = eval(&st, tan_expr, &ctx).unwrap();
        assert!((result - 1.0).abs() < 1e-10);
    }

    #[test]
    fn eval_hyperbolic() {
        let mut st = Store::new();
        let zero = st.int(0);
        let sinh_expr = st.func("sinh", vec![zero]);
        let cosh_expr = st.func("cosh", vec![zero]);
        let ctx = EvalContext::new();

        assert_eq!(eval(&st, sinh_expr, &ctx).unwrap(), 0.0);
        assert_eq!(eval(&st, cosh_expr, &ctx).unwrap(), 1.0);
    }

    #[test]
    fn eval_floor_ceil_round() {
        let mut st = Store::new();
        let val = st.rat(7, 2); // 3.5

        let floor_expr = st.func("floor", vec![val]);
        let ceil_expr = st.func("ceil", vec![val]);
        let round_expr = st.func("round", vec![val]);

        let ctx = EvalContext::new();
        assert_eq!(eval(&st, floor_expr, &ctx).unwrap(), 3.0);
        assert_eq!(eval(&st, ceil_expr, &ctx).unwrap(), 4.0);
        assert_eq!(eval(&st, round_expr, &ctx).unwrap(), 4.0);
    }

    #[test]
    fn eval_unknown_function() {
        let mut st = Store::new();
        let one = st.int(1);
        let expr = st.func("unknown_func", vec![one]);
        let ctx = EvalContext::new();
        assert!(matches!(eval(&st, expr, &ctx), Err(EvalError::UnknownFunction(_))));
    }

    #[test]
    fn eval_asin() {
        let mut st = Store::new();
        let half = st.rat(1, 2);
        let expr = st.func("asin", vec![half]);
        let ctx = EvalContext::new();
        let result = eval(&st, expr, &ctx).unwrap();
        assert!((result - std::f64::consts::FRAC_PI_6).abs() < 1e-10);
    }

    #[test]
    fn eval_asin_domain_error() {
        let mut st = Store::new();
        let two = st.int(2);
        let expr = st.func("asin", vec![two]);
        let ctx = EvalContext::new();
        assert!(matches!(eval(&st, expr, &ctx), Err(EvalError::DomainError(_))));
    }

    #[test]
    fn eval_arcsin_alias() {
        let mut st = Store::new();
        let zero = st.int(0);
        let expr = st.func("arcsin", vec![zero]);
        let ctx = EvalContext::new();
        assert_eq!(eval(&st, expr, &ctx).unwrap(), 0.0);
    }

    #[test]
    fn eval_acos() {
        let mut st = Store::new();
        let zero = st.int(0);
        let expr = st.func("acos", vec![zero]);
        let ctx = EvalContext::new();
        let result = eval(&st, expr, &ctx).unwrap();
        assert!((result - std::f64::consts::FRAC_PI_2).abs() < 1e-10);
    }

    #[test]
    fn eval_acos_domain_error() {
        let mut st = Store::new();
        let neg_two = st.int(-2);
        let expr = st.func("acos", vec![neg_two]);
        let ctx = EvalContext::new();
        assert!(matches!(eval(&st, expr, &ctx), Err(EvalError::DomainError(_))));
    }

    #[test]
    fn eval_arccos_alias() {
        let mut st = Store::new();
        let one = st.int(1);
        let expr = st.func("arccos", vec![one]);
        let ctx = EvalContext::new();
        assert_eq!(eval(&st, expr, &ctx).unwrap(), 0.0);
    }

    #[test]
    fn eval_atan() {
        let mut st = Store::new();
        let one = st.int(1);
        let expr = st.func("atan", vec![one]);
        let ctx = EvalContext::new();
        let result = eval(&st, expr, &ctx).unwrap();
        assert!((result - std::f64::consts::FRAC_PI_4).abs() < 1e-10);
    }

    #[test]
    fn eval_arctan_alias() {
        let mut st = Store::new();
        let zero = st.int(0);
        let expr = st.func("arctan", vec![zero]);
        let ctx = EvalContext::new();
        assert_eq!(eval(&st, expr, &ctx).unwrap(), 0.0);
    }

    #[test]
    fn eval_tanh() {
        let mut st = Store::new();
        let zero = st.int(0);
        let expr = st.func("tanh", vec![zero]);
        let ctx = EvalContext::new();
        assert_eq!(eval(&st, expr, &ctx).unwrap(), 0.0);
    }

    #[test]
    fn eval_log_alias() {
        let mut st = Store::new();
        let e = st.rat(271828, 100000); // Approx e
        let expr = st.func("log", vec![e]);
        let ctx = EvalContext::new();
        let result = eval(&st, expr, &ctx).unwrap();
        assert!((result - 1.0).abs() < 0.01);
    }

    #[test]
    fn eval_log10() {
        let mut st = Store::new();
        let hundred = st.int(100);
        let expr = st.func("log10", vec![hundred]);
        let ctx = EvalContext::new();
        assert_eq!(eval(&st, expr, &ctx).unwrap(), 2.0);
    }

    #[test]
    fn eval_log10_domain_error() {
        let mut st = Store::new();
        let neg_one = st.int(-1);
        let expr = st.func("log10", vec![neg_one]);
        let ctx = EvalContext::new();
        assert!(matches!(eval(&st, expr, &ctx), Err(EvalError::DomainError(_))));
    }

    #[test]
    fn eval_log2() {
        let mut st = Store::new();
        let eight = st.int(8);
        let expr = st.func("log2", vec![eight]);
        let ctx = EvalContext::new();
        assert_eq!(eval(&st, expr, &ctx).unwrap(), 3.0);
    }

    #[test]
    fn eval_log2_domain_error() {
        let mut st = Store::new();
        let zero = st.int(0);
        let expr = st.func("log2", vec![zero]);
        let ctx = EvalContext::new();
        assert!(matches!(eval(&st, expr, &ctx), Err(EvalError::DomainError(_))));
    }

    #[test]
    fn eval_atan2() {
        let mut st = Store::new();
        let one = st.int(1);
        let zero = st.int(0);
        let expr = st.func("atan2", vec![one, zero]);
        let ctx = EvalContext::new();
        let result = eval(&st, expr, &ctx).unwrap();
        assert!((result - std::f64::consts::FRAC_PI_2).abs() < 1e-10);
    }

    #[test]
    fn eval_arctan2_alias() {
        let mut st = Store::new();
        let one = st.int(1);
        let expr = st.func("arctan2", vec![one, one]);
        let ctx = EvalContext::new();
        let result = eval(&st, expr, &ctx).unwrap();
        assert!((result - std::f64::consts::FRAC_PI_4).abs() < 1e-10);
    }

    #[test]
    fn eval_min_insufficient_args() {
        let mut st = Store::new();
        let one = st.int(1);
        let expr = st.func("min", vec![one]);
        let ctx = EvalContext::new();
        assert!(matches!(eval(&st, expr, &ctx), Err(EvalError::DomainError(_))));
    }

    #[test]
    fn eval_max_insufficient_args() {
        let mut st = Store::new();
        let one = st.int(1);
        let expr = st.func("max", vec![one]);
        let ctx = EvalContext::new();
        assert!(matches!(eval(&st, expr, &ctx), Err(EvalError::DomainError(_))));
    }

    #[test]
    fn eval_context_get_none() {
        let ctx = EvalContext::new();
        assert_eq!(ctx.get("nonexistent"), None);
    }

    #[test]
    fn eval_context_clear() {
        let mut ctx = EvalContext::new();
        ctx.bind("x", 5.0);
        assert_eq!(ctx.get("x"), Some(5.0));
        ctx.clear();
        assert_eq!(ctx.get("x"), None);
    }

    #[test]
    fn eval_error_display() {
        let err1 = EvalError::UnboundVariable("x".to_string());
        assert_eq!(err1.to_string(), "unbound variable: x");

        let err2 = EvalError::UnknownFunction("foo".to_string());
        assert_eq!(err2.to_string(), "unknown function: foo");

        let err3 = EvalError::DomainError("test message".to_string());
        assert_eq!(err3.to_string(), "domain error: test message");

        let err4 = EvalError::NonFinite;
        assert_eq!(err4.to_string(), "result is non-finite");
    }

    #[test]
    fn eval_non_finite_infinity() {
        let mut st = Store::new();
        let x = st.sym("x");
        let expr = st.func("exp", vec![x]);
        let mut ctx = EvalContext::new();
        ctx.bind("x", 1000.0); // Very large number
                               // exp(1000) results in infinity
        assert!(matches!(eval(&st, expr, &ctx), Err(EvalError::NonFinite)));
    }

    #[test]
    fn eval_arity_check_sin() {
        let mut st = Store::new();
        let one = st.int(1);
        let two = st.int(2);
        let expr = st.func("sin", vec![one, two]);
        let ctx = EvalContext::new();
        assert!(matches!(eval(&st, expr, &ctx), Err(EvalError::DomainError(_))));
    }

    #[test]
    fn eval_arity_check_exp() {
        let mut st = Store::new();
        let expr = st.func("exp", vec![]);
        let ctx = EvalContext::new();
        assert!(matches!(eval(&st, expr, &ctx), Err(EvalError::DomainError(_))));
    }

    #[test]
    fn eval_arity_check_atan2() {
        let mut st = Store::new();
        let one = st.int(1);
        let expr = st.func("atan2", vec![one]);
        let ctx = EvalContext::new();
        assert!(matches!(eval(&st, expr, &ctx), Err(EvalError::DomainError(_))));
    }

    // Special function tests (Phase 3)
    #[test]
    fn eval_gamma_at_one() {
        let mut st = Store::new();
        let one = st.int(1);
        let gamma_1 = st.func("Gamma", vec![one]);
        let ctx = EvalContext::new();
        let result = eval(&st, gamma_1, &ctx).unwrap();
        assert_eq!(result, 1.0);
    }

    #[test]
    fn eval_gamma_at_half() {
        let mut st = Store::new();
        let half = st.rat(1, 2);
        let gamma_half = st.func("Gamma", vec![half]);
        let ctx = EvalContext::new();
        let result = eval(&st, gamma_half, &ctx).unwrap();
        // Γ(1/2) = √π
        assert!((result - std::f64::consts::PI.sqrt()).abs() < 1e-10);
    }

    #[test]
    fn eval_erf_at_zero() {
        let mut st = Store::new();
        let zero = st.int(0);
        let erf_0 = st.func("erf", vec![zero]);
        let ctx = EvalContext::new();
        let result = eval(&st, erf_0, &ctx).unwrap();
        assert_eq!(result, 0.0);
    }

    #[test]
    fn eval_erf_small_value() {
        let mut st = Store::new();
        let half = st.rat(1, 2);
        let erf_half = st.func("erf", vec![half]);
        let ctx = EvalContext::new();
        let result = eval(&st, erf_half, &ctx).unwrap();
        // erf(0.5) ≈ 0.5205
        assert!((result - 0.5205).abs() < 0.001);
    }
}
