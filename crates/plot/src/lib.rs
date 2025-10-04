//! plot: simple SVG plotter for expressions using f64 evaluation.
//! - Evaluates expressions w.r.t. a single variable (e.g., "x")
//! - Renders a polyline SVG with fixed-precision coordinates for determinism
//! - No external deps; minimal feature set (Add/Mul/Pow/sin/cos/exp/ln)

#![deny(warnings)]
use expr_core::{ExprId, Op, Payload, Store};

const MARGIN: f64 = 10.0;
const PREC: usize = 6; // digits after decimal for stable output

/// Plot configuration.
pub struct PlotConfig {
    pub var: String,
    pub x_min: f64,
    pub x_max: f64,
    pub samples: usize,
    pub width: u32,
    pub height: u32,
}

impl PlotConfig {
    pub fn new(var: &str, x_min: f64, x_max: f64, samples: usize, width: u32, height: u32) -> Self {
        Self { var: var.into(), x_min, x_max, samples, width, height }
    }
}

/// Evaluate `id` to f64 by substituting `var = x`. Returns None if not evaluable
/// or if the result is non-finite.
pub fn eval_f64(st: &Store, id: ExprId, var: &str, x: f64) -> Option<f64> {
    fn go(st: &Store, id: ExprId, var: &str, x: f64) -> Option<f64> {
        let n = st.get(id);
        match (&n.op, &n.payload) {
            (Op::Integer, Payload::Int(k)) => Some(*k as f64),
            (Op::Rational, Payload::Rat(a, b)) => Some((*a as f64) / (*b as f64)),
            (Op::Symbol, Payload::Sym(name)) => {
                if name == var {
                    Some(x)
                } else {
                    None
                }
            }
            (Op::Add, _) => {
                let mut acc = 0.0;
                for &c in &n.children {
                    acc += go(st, c, var, x)?;
                }
                Some(acc)
            }
            (Op::Mul, _) => {
                let mut acc = 1.0;
                for &c in &n.children {
                    acc *= go(st, c, var, x)?;
                }
                Some(acc)
            }
            (Op::Pow, _) => {
                let b = go(st, n.children[0], var, x)?;
                let e = go(st, n.children[1], var, x)?;
                Some(b.powf(e))
            }
            (Op::Function, Payload::Func(name)) => {
                if n.children.len() != 1 {
                    return None;
                }
                let u = go(st, n.children[0], var, x)?;
                let y = match name.as_str() {
                    "sin" => u.sin(),
                    "cos" => u.cos(),
                    "exp" => u.exp(),
                    "ln" => u.ln(),
                    _ => return None,
                };
                Some(y)
            }
            _ => None,
        }
    }
    let y = go(st, id, var, x)?;
    if y.is_finite() {
        Some(y)
    } else {
        None
    }
}

/// Render an SVG polyline for `id` over [x_min, x_max], sampling `samples` points.
/// Returns a full `<svg>` document string of fixed size `width` x `height`.
pub fn plot_svg(st: &Store, id: ExprId, cfg: &PlotConfig) -> String {
    let width_f = cfg.width as f64;
    let height_f = cfg.height as f64;
    let plot_w = (width_f - 2.0 * MARGIN).max(1.0);
    let plot_h = (height_f - 2.0 * MARGIN).max(1.0);
    let n = cfg.samples.max(2);

    // Sample x uniformly and collect (x, y)
    let dx = if n > 1 { (cfg.x_max - cfg.x_min) / (n as f64 - 1.0) } else { 0.0 };
    let mut xs: Vec<f64> = Vec::with_capacity(n);
    let mut ys: Vec<Option<f64>> = Vec::with_capacity(n);
    for i in 0..n {
        let x = cfg.x_min + (i as f64) * dx;
        let y = eval_f64(st, id, &cfg.var, x);
        xs.push(x);
        ys.push(y);
    }

    // Determine y-range from finite values
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;
    for y_opt in &ys {
        if let Some(y) = *y_opt {
            if y < y_min {
                y_min = y;
            }
            if y > y_max {
                y_max = y;
            }
        }
    }
    if !y_min.is_finite() || !y_max.is_finite() {
        // No finite points; return empty polyline SVG
        return empty_svg(cfg.width, cfg.height);
    }
    // Avoid zero height range
    if (y_max - y_min).abs() < 1e-12 {
        y_min -= 1.0;
        y_max += 1.0;
    }

    // Build points string with fixed precision
    let mut points: Vec<String> = Vec::with_capacity(n);
    for (i, y_opt) in ys.into_iter().enumerate() {
        if let Some(y) = y_opt {
            let x_screen = MARGIN + (xs[i] - cfg.x_min) / (cfg.x_max - cfg.x_min) * plot_w;
            let y_norm = (y - y_min) / (y_max - y_min);
            let y_screen = height_f - MARGIN - y_norm * plot_h; // invert for SVG
            points.push(format!("{:.p$},{:.p$}", x_screen, y_screen, p = PREC));
        }
    }

    let polyline = format!(
        "<polyline fill=\"none\" stroke=\"#1f77b4\" stroke-width=\"1.5\" points=\"{}\" />",
        points.join(" ")
    );

    // Optional border for context
    let border = format!(
        "<rect x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"none\" stroke=\"#ccc\" stroke-width=\"1\" />",
        cfg.width,
        cfg.height
    );

    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\">{}{}{}{}{}\n{}</svg>",
        cfg.width, cfg.height, "\n", border, "\n", polyline, "\n", ""
    )
}

fn empty_svg(width: u32, height: u32) -> String {
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\"></svg>",
        width, height
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plot_parabola_deterministic_points() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two); // f(x) = x^2
        let cfg = PlotConfig::new("x", -1.0, 1.0, 5, 100, 100);
        let svg = plot_svg(&st, x2, &cfg);
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("<polyline"));
        // Expect 5 points with fixed coordinates (see mapping in implementation)
        let expected = [(10.0, 10.0), (30.0, 70.0), (50.0, 90.0), (70.0, 70.0), (90.0, 10.0)];
        for (x, y) in expected {
            let needle = format!("{:.p$},{:.p$}", x, y, p = PREC);
            assert!(svg.contains(&needle), "missing point {}", needle);
        }
        // Deterministic: calling again yields identical output
        let svg2 = plot_svg(&st, x2, &cfg);
        assert_eq!(svg, svg2);
    }

    #[test]
    fn eval_basic_funcs() {
        let mut st = Store::new();
        let x = st.sym("x");
        let sinx = st.func("sin", vec![x]);
        let v = eval_f64(&st, sinx, "x", std::f64::consts::FRAC_PI_2).unwrap();
        assert!((v - 1.0).abs() < 1e-12);
        let lnx = st.func("ln", vec![x]);
        assert!(eval_f64(&st, lnx, "x", -1.0).is_none()); // domain error handled
    }

    #[test]
    fn empty_svg_when_variable_unbound() {
        let mut st = Store::new();
        let y = st.sym("y"); // variable is y
                             // Plot using var "x"; all evals will be None => empty svg
        let cfg = PlotConfig::new("x", -1.0, 1.0, 5, 100, 100);
        let svg = plot_svg(&st, y, &cfg);
        assert_eq!(
            svg,
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"100\"></svg>"
        );
    }

    #[test]
    fn eval_pow_rational_exponent() {
        let mut st = Store::new();
        let x = st.sym("x");
        let half = st.rat(1, 2);
        let sqrt_x = st.pow(x, half);
        let v = eval_f64(&st, sqrt_x, "x", 4.0).unwrap();
        assert!((v - 2.0).abs() < 1e-12);
    }

    #[test]
    fn eval_add() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let expr = st.add(vec![x, two]);
        let v = eval_f64(&st, expr, "x", 3.0).unwrap();
        assert!((v - 5.0).abs() < 1e-12);
    }

    #[test]
    fn eval_mul() {
        let mut st = Store::new();
        let x = st.sym("x");
        let three = st.int(3);
        let expr = st.mul(vec![three, x]);
        let v = eval_f64(&st, expr, "x", 2.0).unwrap();
        assert!((v - 6.0).abs() < 1e-12);
    }

    #[test]
    fn eval_cosx() {
        let mut st = Store::new();
        let x = st.sym("x");
        let cosx = st.func("cos", vec![x]);
        let v = eval_f64(&st, cosx, "x", 0.0).unwrap();
        assert!((v - 1.0).abs() < 1e-12);
    }

    #[test]
    fn eval_expx() {
        let mut st = Store::new();
        let x = st.sym("x");
        let expx = st.func("exp", vec![x]);
        let v = eval_f64(&st, expx, "x", 0.0).unwrap();
        assert!((v - 1.0).abs() < 1e-12);
    }

    #[test]
    fn eval_unknown_func() {
        let mut st = Store::new();
        let x = st.sym("x");
        let fx = st.func("unknown", vec![x]);
        assert!(eval_f64(&st, fx, "x", 1.0).is_none());
    }

    #[test]
    fn eval_multiarg_func() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let f = st.func("f", vec![x, y]);
        assert!(eval_f64(&st, f, "x", 1.0).is_none());
    }

    #[test]
    fn eval_unbound_symbol() {
        let mut st = Store::new();
        let y = st.sym("y");
        assert!(eval_f64(&st, y, "x", 1.0).is_none());
    }

    #[test]
    fn plot_single_sample() {
        let mut st = Store::new();
        let x = st.sym("x");
        let cfg = PlotConfig::new("x", 0.0, 1.0, 1, 100, 100);
        let svg = plot_svg(&st, x, &cfg);
        assert!(svg.contains("<svg"));
    }
}
