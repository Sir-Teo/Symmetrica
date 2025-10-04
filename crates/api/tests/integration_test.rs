//! Integration tests for Python bindings
//! 
//! Note: PyO3 methods are not directly accessible from Rust code.
//! Full Python API testing should be done through Python test suite.
//! These tests verify core functionality works.

#[test]
fn test_crate_compiles() {
    // If this test runs, the crate compiled successfully
    // No assertion needed - successful compilation is the test
}

#[test]
fn test_underlying_api() {
    // Test the underlying Rust API that the Python bindings use
    use expr_core::Store;
    use simplify::simplify;
    use calculus::diff;
    
    let mut st = Store::new();
    let x = st.sym("x");
    let two = st.int(2);
    let x2 = st.pow(x, two);
    
    // Test simplification
    let simplified = simplify(&mut st, x2);
    assert_eq!(st.get(simplified).digest, st.get(x2).digest);
    
    // Test differentiation
    let deriv = diff(&mut st, x2, "x");
    let result = st.to_string(deriv);
    assert!(result.contains("2") && result.contains("x"));
}

#[test]
fn test_integration_api() {
    use expr_core::Store;
    use calculus::integrate;
    use simplify::simplify;
    
    let mut st = Store::new();
    let x = st.sym("x");
    
    // Integrate x
    if let Some(integral) = integrate(&mut st, x, "x") {
        let simplified = simplify(&mut st, integral);
        let result = st.to_string(simplified);
        // Should contain x^2 and fraction
        assert!(result.contains("x") && result.contains("2"));
    }
}

#[test]
fn test_solver_api() {
    use expr_core::Store;
    use solver::solve_univariate;
    
    let mut st = Store::new();
    let x = st.sym("x");
    let two = st.int(2);
    let neg5 = st.int(-5);
    let six = st.int(6);
    
    // x^2 - 5x + 6 = 0
    let x2 = st.pow(x, two);
    let neg5x = st.mul(vec![neg5, x]);
    let expr = st.add(vec![x2, neg5x, six]);
    
    if let Some(roots) = solve_univariate(&mut st, expr, "x") {
        assert_eq!(roots.len(), 2);
    }
}

#[test]
fn test_evalf_api() {
    use expr_core::Store;
    use evalf::{eval, EvalContext};
    
    let mut st = Store::new();
    let five = st.int(5);
    let ctx = EvalContext::new();
    
    let result = eval(&st, five, &ctx).unwrap();
    assert!((result - 5.0).abs() < 1e-10);
}

#[test]
fn test_latex_export() {
    use expr_core::Store;
    use io::to_latex;
    
    let mut st = Store::new();
    let x = st.sym("x");
    let latex = to_latex(&st, x);
    assert!(!latex.is_empty());
}

#[test]
fn test_plot_api() {
    use expr_core::Store;
    use plot::{plot_svg, PlotConfig};
    
    let mut st = Store::new();
    let x = st.sym("x");
    let cfg = PlotConfig::new("x", -1.0, 1.0, 10, 800, 600);
    
    let svg = plot_svg(&st, x, &cfg);
    assert!(svg.contains("svg"));
}
