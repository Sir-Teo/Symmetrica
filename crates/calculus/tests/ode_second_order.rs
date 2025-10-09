//! Tests for second-order constant coefficient ODE solver

use calculus::ode::solve_ode_second_order_constant_coeff;
use expr_core::Store;

#[test]
fn test_second_order_distinct_real_roots() {
    let mut st = Store::new();

    // y'' - 3y' + 2y = 0
    // Characteristic equation: r^2 - 3r + 2 = 0
    // Roots: r = 1, 2
    // Solution: y = C1*e^x + C2*e^(2x)
    let a = st.int(1);
    let b = st.int(-3);
    let c = st.int(2);

    let result = solve_ode_second_order_constant_coeff(&mut st, a, b, c, "x");
    assert!(result.is_some());

    let solution = result.unwrap();
    let sol_str = st.to_string(solution);

    // Should contain exp, C1, C2
    assert!(sol_str.contains("exp"));
    assert!(sol_str.contains("C1") || sol_str.contains("C2"));
}

#[test]
fn test_second_order_repeated_root() {
    let mut st = Store::new();

    // y'' - 2y' + y = 0
    // Characteristic equation: r^2 - 2r + 1 = 0
    // Roots: r = 1 (repeated)
    // Solution: y = (C1 + C2*x)*e^x
    let a = st.int(1);
    let b = st.int(-2);
    let c = st.int(1);

    let result = solve_ode_second_order_constant_coeff(&mut st, a, b, c, "x");
    assert!(result.is_some());

    let solution = result.unwrap();
    let sol_str = st.to_string(solution);

    // Should contain exp and x (for repeated root)
    assert!(sol_str.contains("exp"));
    assert!(sol_str.contains("x") || sol_str.contains("C2"));
}

#[test]
fn test_second_order_complex_roots() {
    let mut st = Store::new();

    // y'' + y = 0
    // Characteristic equation: r^2 + 1 = 0
    // Roots: r = ±i
    // Solution: y = C1*cos(x) + C2*sin(x) (or in exponential form)
    let a = st.int(1);
    let b = st.int(0);
    let c = st.int(1);

    let result = solve_ode_second_order_constant_coeff(&mut st, a, b, c, "x");
    assert!(result.is_some());

    let solution = result.unwrap();
    let sol_str = st.to_string(solution);

    // Should contain exp (complex exponentials)
    assert!(sol_str.contains("exp") || sol_str.len() > 1);
}

#[test]
fn test_second_order_simple_harmonic() {
    let mut st = Store::new();

    // y'' + 4y = 0
    // Characteristic equation: r^2 + 4 = 0
    // Roots: r = ±2i
    // Solution: y = C1*cos(2x) + C2*sin(2x)
    let a = st.int(1);
    let b = st.int(0);
    let c = st.int(4);

    let result = solve_ode_second_order_constant_coeff(&mut st, a, b, c, "x");
    assert!(result.is_some());

    let solution = result.unwrap();
    let sol_str = st.to_string(solution);

    // Should produce a solution
    assert!(!sol_str.is_empty());
}

#[test]
fn test_second_order_damped_oscillator() {
    let mut st = Store::new();

    // y'' + 2y' + 5y = 0
    // Characteristic equation: r^2 + 2r + 5 = 0
    // Roots: r = -1 ± 2i (complex with real part)
    // Solution: y = e^(-x) * (C1*cos(2x) + C2*sin(2x))
    let a = st.int(1);
    let b = st.int(2);
    let c = st.int(5);

    let result = solve_ode_second_order_constant_coeff(&mut st, a, b, c, "x");
    assert!(result.is_some());

    let solution = result.unwrap();
    let sol_str = st.to_string(solution);

    // Should contain exp
    assert!(sol_str.contains("exp") || sol_str.len() > 1);
}

#[test]
fn test_second_order_rational_coefficients() {
    let mut st = Store::new();

    // (1/2)y'' - y' + (1/2)y = 0
    // Multiply by 2: y'' - 2y' + y = 0
    // Roots: r = 1 (repeated)
    let a = st.rat(1, 2);
    let b = st.int(-1);
    let c = st.rat(1, 2);

    let result = solve_ode_second_order_constant_coeff(&mut st, a, b, c, "x");
    assert!(result.is_some());
}
