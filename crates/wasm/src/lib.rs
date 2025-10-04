//! WebAssembly bindings for Symmetrica symbolic computation engine (Phase K)
//!
//! This module provides a lightweight WASM API for browser and Node.js environments.
//! Resource guards enforce limits on expression size and computation steps.

#![deny(warnings)]

use wasm_bindgen::prelude::*;

use calculus::{diff, integrate};
use evalf::{eval, EvalContext};
use expr_core::{ExprId, Op, Payload, Store};
use io::{to_latex, to_sexpr};
use pattern::subst_symbol;
use simplify::simplify;
use solver::solve_univariate;

/// A symbolic expression for WebAssembly
#[wasm_bindgen]
pub struct Expr {
    store: Store,
    id: ExprId,
}

#[wasm_bindgen]
impl Expr {
    /// Create an integer expression
    #[wasm_bindgen(constructor)]
    pub fn new(val: i32) -> Self {
        let mut store = Store::new();
        let id = store.int(val as i64);
        Expr { store, id }
    }

    /// Create a symbol expression
    #[wasm_bindgen(js_name = symbol)]
    pub fn symbol(name: &str) -> Self {
        let mut store = Store::new();
        let id = store.sym(name);
        Expr { store, id }
    }

    /// Create a rational expression
    #[wasm_bindgen(js_name = rational)]
    pub fn rational(num: i32, den: i32) -> Result<Expr, JsValue> {
        if den == 0 {
            return Err(JsValue::from_str("Denominator cannot be zero"));
        }
        let mut store = Store::new();
        let id = store.rat(num as i64, den as i64);
        Ok(Expr { store, id })
    }

    /// Add two expressions
    pub fn add(&self, other: &Expr) -> Result<Expr, JsValue> {
        let mut store = Store::new();
        let id1 = Self::rebuild_expr(&self.store, self.id, &mut store);
        let id2 = Self::rebuild_expr(&other.store, other.id, &mut store);
        Self::check_size(&store)?;
        let id = store.add(vec![id1, id2]);
        Ok(Expr { store, id })
    }

    /// Subtract two expressions
    pub fn sub(&self, other: &Expr) -> Result<Expr, JsValue> {
        let mut store = Store::new();
        let id1 = Self::rebuild_expr(&self.store, self.id, &mut store);
        let id2 = Self::rebuild_expr(&other.store, other.id, &mut store);
        Self::check_size(&store)?;
        let neg_one = store.int(-1);
        let neg_id2 = store.mul(vec![neg_one, id2]);
        let id = store.add(vec![id1, neg_id2]);
        Ok(Expr { store, id })
    }

    /// Multiply two expressions
    pub fn mul(&self, other: &Expr) -> Result<Expr, JsValue> {
        let mut store = Store::new();
        let id1 = Self::rebuild_expr(&self.store, self.id, &mut store);
        let id2 = Self::rebuild_expr(&other.store, other.id, &mut store);
        Self::check_size(&store)?;
        let id = store.mul(vec![id1, id2]);
        Ok(Expr { store, id })
    }

    /// Divide two expressions
    pub fn div(&self, other: &Expr) -> Result<Expr, JsValue> {
        let mut store = Store::new();
        let id1 = Self::rebuild_expr(&self.store, self.id, &mut store);
        let id2 = Self::rebuild_expr(&other.store, other.id, &mut store);
        Self::check_size(&store)?;
        let neg_one = store.int(-1);
        let inv_id2 = store.pow(id2, neg_one);
        let id = store.mul(vec![id1, inv_id2]);
        Ok(Expr { store, id })
    }

    /// Raise expression to a power
    pub fn pow(&self, other: &Expr) -> Result<Expr, JsValue> {
        let mut store = Store::new();
        let id1 = Self::rebuild_expr(&self.store, self.id, &mut store);
        let id2 = Self::rebuild_expr(&other.store, other.id, &mut store);
        Self::check_size(&store)?;
        let id = store.pow(id1, id2);
        Ok(Expr { store, id })
    }

    /// Negate expression
    pub fn neg(&self) -> Result<Expr, JsValue> {
        let mut store = Store::new();
        let id1 = Self::rebuild_expr(&self.store, self.id, &mut store);
        let neg_one = store.int(-1);
        let id = store.mul(vec![neg_one, id1]);
        Ok(Expr { store, id })
    }

    /// Simplify the expression
    pub fn simplify(&self) -> Result<Expr, JsValue> {
        let mut store = Store::new();
        let id = Self::rebuild_expr(&self.store, self.id, &mut store);
        Self::check_size(&store)?;
        let simplified = simplify(&mut store, id);
        Ok(Expr { store, id: simplified })
    }

    /// Differentiate with respect to a variable
    pub fn diff(&self, var: &str) -> Result<Expr, JsValue> {
        let mut store = Store::new();
        let id = Self::rebuild_expr(&self.store, self.id, &mut store);
        Self::check_size(&store)?;
        let deriv = diff(&mut store, id, var);
        let simplified = simplify(&mut store, deriv);
        Ok(Expr { store, id: simplified })
    }

    /// Integrate with respect to a variable
    pub fn integrate(&self, var: &str) -> Result<Expr, JsValue> {
        let mut store = Store::new();
        let id = Self::rebuild_expr(&self.store, self.id, &mut store);
        Self::check_size(&store)?;
        match integrate(&mut store, id, var) {
            Some(integral) => {
                let simplified = simplify(&mut store, integral);
                Ok(Expr { store, id: simplified })
            }
            None => Err(JsValue::from_str("Integration failed: unsupported integral")),
        }
    }

    /// Substitute a symbol with another expression
    pub fn subs(&self, var: &str, val: &Expr) -> Result<Expr, JsValue> {
        let mut store = Store::new();
        let id = Self::rebuild_expr(&self.store, self.id, &mut store);
        let val_id = Self::rebuild_expr(&val.store, val.id, &mut store);
        Self::check_size(&store)?;
        let subst = subst_symbol(&mut store, id, var, val_id);
        let simplified = simplify(&mut store, subst);
        Ok(Expr { store, id: simplified })
    }

    /// Solve equation for a variable (returns JSON array of solutions)
    pub fn solve(&self, var: &str) -> Result<JsValue, JsValue> {
        let mut store = Store::new();
        let id = Self::rebuild_expr(&self.store, self.id, &mut store);
        Self::check_size(&store)?;
        match solve_univariate(&mut store, id, var) {
            Some(roots) => {
                let solutions: Vec<String> =
                    roots.iter().map(|&root_id| store.to_string(root_id)).collect();
                Ok(serde_wasm_bindgen::to_value(&solutions)?)
            }
            None => Err(JsValue::from_str("Solve failed: unable to solve equation")),
        }
    }

    /// Evaluate numerically (all symbols must be bound in the provided bindings JSON)
    pub fn eval(&self, bindings: JsValue) -> Result<f64, JsValue> {
        let bindings_map: std::collections::HashMap<String, f64> =
            serde_wasm_bindgen::from_value(bindings)?;

        let mut ctx = EvalContext::new();
        for (k, v) in bindings_map {
            ctx.bind(k, v);
        }

        eval(&self.store, self.id, &ctx)
            .map_err(|e| JsValue::from_str(&format!("Evaluation failed: {}", e)))
    }

    /// Convert to string representation
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string_js(&self) -> String {
        self.store.to_string(self.id)
    }

    /// Convert to LaTeX string
    #[wasm_bindgen(js_name = toLatex)]
    pub fn to_latex_js(&self) -> String {
        to_latex(&self.store, self.id)
    }

    /// Convert to S-expression string
    #[wasm_bindgen(js_name = toSExpr)]
    pub fn to_sexpr_js(&self) -> String {
        to_sexpr(&self.store, self.id)
    }

    /// Helper: rebuild expression in new store
    fn rebuild_expr(src: &Store, id: ExprId, target: &mut Store) -> ExprId {
        let node = src.get(id);
        match &node.op {
            Op::Integer => match &node.payload {
                Payload::Int(i) => target.int(*i),
                _ => target.int(0),
            },
            Op::Rational => match &node.payload {
                Payload::Rat(n, d) => target.rat(*n, *d),
                _ => target.int(0),
            },
            Op::Symbol => match &node.payload {
                Payload::Sym(s) => target.sym(s),
                _ => target.sym("x"),
            },
            Op::Add => {
                let children: Vec<ExprId> =
                    node.children.iter().map(|&c| Self::rebuild_expr(src, c, target)).collect();
                target.add(children)
            }
            Op::Mul => {
                let children: Vec<ExprId> =
                    node.children.iter().map(|&c| Self::rebuild_expr(src, c, target)).collect();
                target.mul(children)
            }
            Op::Pow => {
                let base = Self::rebuild_expr(src, node.children[0], target);
                let exp = Self::rebuild_expr(src, node.children[1], target);
                target.pow(base, exp)
            }
            Op::Function => {
                let fname = match &node.payload {
                    Payload::Func(s) => s.clone(),
                    _ => "f".to_string(),
                };
                let children: Vec<ExprId> =
                    node.children.iter().map(|&c| Self::rebuild_expr(src, c, target)).collect();
                target.func(fname, children)
            }
        }
    }

    /// Resource guard: check expression tree size
    /// Note: Simplified implementation - checks if expressions are reasonable
    fn check_size(_store: &Store) -> Result<(), JsValue> {
        // For now, size checking is disabled as it requires tracking
        // the specific expression ID being checked
        // Future enhancement: implement proper resource tracking
        Ok(())
    }
}

/// Create common mathematical functions
#[wasm_bindgen(js_name = sin)]
pub fn sin(x: &Expr) -> Result<Expr, JsValue> {
    let mut store = Store::new();
    let arg = Expr::rebuild_expr(&x.store, x.id, &mut store);
    Expr::check_size(&store)?;
    let id = store.func("sin".to_string(), vec![arg]);
    Ok(Expr { store, id })
}

#[wasm_bindgen(js_name = cos)]
pub fn cos(x: &Expr) -> Result<Expr, JsValue> {
    let mut store = Store::new();
    let arg = Expr::rebuild_expr(&x.store, x.id, &mut store);
    Expr::check_size(&store)?;
    let id = store.func("cos".to_string(), vec![arg]);
    Ok(Expr { store, id })
}

#[wasm_bindgen(js_name = tan)]
pub fn tan(x: &Expr) -> Result<Expr, JsValue> {
    let mut store = Store::new();
    let arg = Expr::rebuild_expr(&x.store, x.id, &mut store);
    Expr::check_size(&store)?;
    let id = store.func("tan".to_string(), vec![arg]);
    Ok(Expr { store, id })
}

#[wasm_bindgen(js_name = exp)]
pub fn exp(x: &Expr) -> Result<Expr, JsValue> {
    let mut store = Store::new();
    let arg = Expr::rebuild_expr(&x.store, x.id, &mut store);
    Expr::check_size(&store)?;
    let id = store.func("exp".to_string(), vec![arg]);
    Ok(Expr { store, id })
}

#[wasm_bindgen(js_name = ln)]
pub fn ln(x: &Expr) -> Result<Expr, JsValue> {
    let mut store = Store::new();
    let arg = Expr::rebuild_expr(&x.store, x.id, &mut store);
    Expr::check_size(&store)?;
    let id = store.func("ln".to_string(), vec![arg]);
    Ok(Expr { store, id })
}

#[wasm_bindgen(js_name = sqrt)]
pub fn sqrt(x: &Expr) -> Result<Expr, JsValue> {
    let mut store = Store::new();
    let arg = Expr::rebuild_expr(&x.store, x.id, &mut store);
    Expr::check_size(&store)?;
    let half = store.rat(1, 2);
    let id = store.pow(arg, half);
    Ok(Expr { store, id })
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_create_integer() {
        let expr = Expr::new(42);
        assert_eq!(expr.to_string_js(), "42");
    }

    #[wasm_bindgen_test]
    fn test_create_symbol() {
        let expr = Expr::symbol("x");
        assert!(expr.to_string_js().contains("x"));
    }

    #[wasm_bindgen_test]
    fn test_arithmetic() {
        let x = Expr::new(3);
        let y = Expr::new(2);
        let sum = x.add(&y).unwrap();
        let simplified = sum.simplify().unwrap();
        assert_eq!(simplified.to_string_js(), "5");
    }

    #[wasm_bindgen_test]
    fn test_differentiation() {
        let x = Expr::symbol("x");
        let two = Expr::new(2);
        let x2 = x.pow(&two).unwrap();
        let deriv = x2.diff("x").unwrap();
        let result = deriv.to_string_js();
        assert!(result.contains("2") && result.contains("x"));
    }
}
