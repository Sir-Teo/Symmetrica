//! Python bindings for Symmetrica symbolic computation engine (Phase K)
#![deny(warnings)]
#![allow(non_local_definitions)]

#[cfg(feature = "python")]
mod python_bindings {
    use pyo3::exceptions::PyValueError;
    use pyo3::prelude::*;

    use calculus::{diff, integrate};
    use evalf::{eval, EvalContext};
    use expr_core::{ExprId, Op, Payload, Store};
    use io::{to_latex, to_sexpr};
    use pattern::subst_symbol;
    use plot::{plot_svg, PlotConfig};
    use simplify::simplify;
    use solver::solve_univariate;

    /// A symbolic expression wrapper for Python
    #[pyclass]
    pub struct Expr {
        store: Store,
        id: ExprId,
    }

#[pymethods]
impl Expr {
    /// Create a new expression from an integer
    #[staticmethod]
    fn int(val: i64) -> Self {
        let mut store = Store::new();
        let id = store.int(val);
        Expr { store, id }
    }

    /// Create a new expression from a rational number
    #[staticmethod]
    fn rat(num: i64, den: i64) -> PyResult<Self> {
        if den == 0 {
            return Err(PyValueError::new_err("Denominator cannot be zero"));
        }
        let mut store = Store::new();
        let id = store.rat(num, den);
        Ok(Expr { store, id })
    }

    /// Create a new symbol expression
    #[staticmethod]
    fn sym(name: String) -> Self {
        let mut store = Store::new();
        let id = store.sym(&name);
        Expr { store, id }
    }

    /// Add two expressions
    fn __add__(&self, other: &Expr) -> Self {
        let mut store = Store::new();
        let id1 = self.rebuild_in(&mut store);
        let id2 = other.rebuild_in(&mut store);
        let id = store.add(vec![id1, id2]);
        Expr { store, id }
    }

    /// Subtract two expressions
    fn __sub__(&self, other: &Expr) -> Self {
        let mut store = Store::new();
        let id1 = self.rebuild_in(&mut store);
        let id2 = other.rebuild_in(&mut store);
        let neg_one = store.int(-1);
        let neg_id2 = store.mul(vec![neg_one, id2]);
        let id = store.add(vec![id1, neg_id2]);
        Expr { store, id }
    }

    /// Multiply two expressions
    fn __mul__(&self, other: &Expr) -> Self {
        let mut store = Store::new();
        let id1 = self.rebuild_in(&mut store);
        let id2 = other.rebuild_in(&mut store);
        let id = store.mul(vec![id1, id2]);
        Expr { store, id }
    }

    /// Divide two expressions (returns rational expression)
    fn __truediv__(&self, other: &Expr) -> Self {
        let mut store = Store::new();
        let id1 = self.rebuild_in(&mut store);
        let id2 = other.rebuild_in(&mut store);
        let neg_one = store.int(-1);
        let inv_id2 = store.pow(id2, neg_one);
        let id = store.mul(vec![id1, inv_id2]);
        Expr { store, id }
    }

    /// Raise expression to a power
    fn __pow__(&self, other: &Expr, _mod: Option<&PyAny>) -> Self {
        let mut store = Store::new();
        let id1 = self.rebuild_in(&mut store);
        let id2 = other.rebuild_in(&mut store);
        let id = store.pow(id1, id2);
        Expr { store, id }
    }

    /// Negate expression
    fn __neg__(&self) -> Self {
        let mut store = Store::new();
        let id1 = self.rebuild_in(&mut store);
        let neg_one = store.int(-1);
        let id = store.mul(vec![neg_one, id1]);
        Expr { store, id }
    }

    /// String representation
    fn __str__(&self) -> String {
        self.store.to_string(self.id)
    }

    /// Repr
    fn __repr__(&self) -> String {
        format!("Expr('{}')", self.store.to_string(self.id))
    }

    /// Simplify the expression
    fn simplify(&self) -> Self {
        let mut store = Store::new();
        let id = self.rebuild_in(&mut store);
        let simplified = simplify(&mut store, id);
        Expr { store, id: simplified }
    }

    /// Differentiate with respect to a variable
    fn diff(&self, var: String) -> Self {
        let mut store = Store::new();
        let id = self.rebuild_in(&mut store);
        let deriv = diff(&mut store, id, &var);
        let simplified = simplify(&mut store, deriv);
        Expr { store, id: simplified }
    }

    /// Integrate with respect to a variable
    fn integrate(&self, var: String) -> PyResult<Self> {
        let mut store = Store::new();
        let id = self.rebuild_in(&mut store);
        match integrate(&mut store, id, &var) {
            Some(integral) => {
                let simplified = simplify(&mut store, integral);
                Ok(Expr { store, id: simplified })
            }
            None => Err(PyValueError::new_err("Integration failed: unsupported integral")),
        }
    }

    /// Substitute a symbol with another expression
    fn subs(&self, var: String, val: &Expr) -> Self {
        let mut store = Store::new();
        let id = self.rebuild_in(&mut store);
        let val_id = val.rebuild_in(&mut store);
        let subst = subst_symbol(&mut store, id, &var, val_id);
        let simplified = simplify(&mut store, subst);
        Expr { store, id: simplified }
    }

    /// Evaluate numerically
    fn evalf(&self) -> PyResult<f64> {
        let ctx = EvalContext::new();
        eval(&self.store, self.id, &ctx)
            .map_err(|e| PyValueError::new_err(format!("Evaluation failed: {}", e)))
    }

    /// Convert to LaTeX string
    fn to_latex(&self) -> String {
        to_latex(&self.store, self.id)
    }

    /// Convert to S-expression string
    fn to_sexpr(&self) -> String {
        to_sexpr(&self.store, self.id)
    }

    /// Solve equation for a variable (returns list of solutions)
    fn solve(&self, var: String) -> PyResult<Vec<Expr>> {
        let mut store = Store::new();
        let id = self.rebuild_in(&mut store);
        match solve_univariate(&mut store, id, &var) {
            Some(roots) => Ok(roots
                .into_iter()
                .map(|root_id| {
                    let mut new_store = Store::new();
                    let new_id = Self::rebuild_expr(&store, root_id, &mut new_store);
                    Expr { store: new_store, id: new_id }
                })
                .collect()),
            None => Err(PyValueError::new_err("Solve failed: unable to solve equation")),
        }
    }

    /// Plot as SVG (returns SVG string)
    fn plot(
        &self,
        var: String,
        x_min: f64,
        x_max: f64,
        samples: Option<usize>,
    ) -> String {
        let n = samples.unwrap_or(200);
        let cfg = PlotConfig::new(&var, x_min, x_max, n, 800, 600);
        plot_svg(&self.store, self.id, &cfg)
    }
}

impl Expr {
    /// Helper to rebuild this expression in a new store
    fn rebuild_in(&self, target: &mut Store) -> ExprId {
        Self::rebuild_expr(&self.store, self.id, target)
    }

    /// Recursively rebuild an expression from one store into another
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
}

/// Helper functions module
#[pymodule]
fn symmetrica(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Expr>()?;

    /// Create common mathematical functions
    #[pyfn(m)]
    fn sin(x: &Expr) -> Expr {
        let mut store = Store::new();
        let arg = x.rebuild_in(&mut store);
        let id = store.func("sin".to_string(), vec![arg]);
        Expr { store, id }
    }

    #[pyfn(m)]
    fn cos(x: &Expr) -> Expr {
        let mut store = Store::new();
        let arg = x.rebuild_in(&mut store);
        let id = store.func("cos".to_string(), vec![arg]);
        Expr { store, id }
    }

    #[pyfn(m)]
    fn tan(x: &Expr) -> Expr {
        let mut store = Store::new();
        let arg = x.rebuild_in(&mut store);
        let id = store.func("tan".to_string(), vec![arg]);
        Expr { store, id }
    }

    #[pyfn(m)]
    fn exp(x: &Expr) -> Expr {
        let mut store = Store::new();
        let arg = x.rebuild_in(&mut store);
        let id = store.func("exp".to_string(), vec![arg]);
        Expr { store, id }
    }

    #[pyfn(m)]
    fn ln(x: &Expr) -> Expr {
        let mut store = Store::new();
        let arg = x.rebuild_in(&mut store);
        let id = store.func("ln".to_string(), vec![arg]);
        Expr { store, id }
    }

    #[pyfn(m)]
    fn log(x: &Expr) -> Expr {
        let mut store = Store::new();
        let arg = x.rebuild_in(&mut store);
        let id = store.func("log".to_string(), vec![arg]);
        Expr { store, id }
    }

    #[pyfn(m)]
    fn sqrt(x: &Expr) -> Expr {
        let mut store = Store::new();
        let arg = x.rebuild_in(&mut store);
        let half = store.rat(1, 2);
        let id = store.pow(arg, half);
        Expr { store, id }
    }

    Ok(())
    }
}

#[cfg(feature = "python")]
pub use python_bindings::symmetrica;
