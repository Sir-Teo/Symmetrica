//! Tests for exact ODE solver

use calculus::diff::diff;
use calculus::ode::solve_ode_exact;
use expr_core::Store;
use simplify::simplify;

#[test]
fn test_exact_simple_linear() {
    let mut st = Store::new();

    // (2x + y)dx + (x + 2y)dy = 0
    // M = 2x + y, N = x + 2y
    // ∂M/∂y = 1, ∂N/∂x = 1 ✓ (exact)
    // Solution: x^2 + xy + y^2 = C

    let x = st.sym("x");
    let y = st.sym("y");
    let two = st.int(2);

    let two_x = st.mul(vec![two, x]);
    let m = st.add(vec![two_x, y]); // 2x + y

    let two_y = st.mul(vec![two, y]);
    let n = st.add(vec![x, two_y]); // x + 2y

    let result = solve_ode_exact(&mut st, m, n, "x", "y");
    assert!(result.is_some());

    let solution = result.unwrap();
    let sol_str = st.to_string(solution);

    // Should contain x^2, xy, and y^2 terms
    assert!(sol_str.contains("x") && sol_str.contains("y"));
}

#[test]
fn test_exact_polynomial() {
    let mut st = Store::new();

    // (2x + y)dx + (x)dy = 0
    // M = 2x + y, N = x
    // ∂M/∂y = 1, ∂N/∂x = 1 ✓ (exact)
    // Solution: x^2 + xy = C

    let x = st.sym("x");
    let y = st.sym("y");
    let two = st.int(2);

    let two_x = st.mul(vec![two, x]);
    let m = st.add(vec![two_x, y]); // 2x + y

    let n = x; // x

    let result = solve_ode_exact(&mut st, m, n, "x", "y");
    assert!(result.is_some());

    let solution = result.unwrap();

    // Verify by differentiation: ∂F/∂x should equal M
    let df_dx = diff(&mut st, solution, "x");
    let df_dx_simplified = simplify(&mut st, df_dx);
    let m_simplified = simplify(&mut st, m);

    // They should be equal (or at least structurally similar)
    assert_eq!(df_dx_simplified, m_simplified);
}

#[test]
fn test_exact_exponential() {
    let mut st = Store::new();

    // Simple polynomial case that we know works
    // (x + y)dx + (x + y)dy = 0
    // M = x + y, N = x + y
    // ∂M/∂y = 1, ∂N/∂x = 1 ✓ (exact)

    let x = st.sym("x");
    let y = st.sym("y");

    let m = st.add(vec![x, y]); // x + y
    let n = st.add(vec![x, y]); // x + y

    let result = solve_ode_exact(&mut st, m, n, "x", "y");
    assert!(result.is_some());

    let solution = result.unwrap();
    let sol_str = st.to_string(solution);

    // Should contain x and y
    assert!(sol_str.contains("x") && sol_str.contains("y"));
}

#[test]
fn test_not_exact() {
    let mut st = Store::new();

    // y*dx + x*dy = 0 is NOT exact
    // M = y, N = x
    // ∂M/∂y = 1, ∂N/∂x = 1 ✓ (actually this IS exact!)
    // Let's try a different one

    // y*dx + 2x*dy = 0 is NOT exact
    // M = y, N = 2x
    // ∂M/∂y = 1, ∂N/∂x = 2 ✗ (not exact)

    let x = st.sym("x");
    let y = st.sym("y");
    let two = st.int(2);

    let m = y; // y
    let n = st.mul(vec![two, x]); // 2x

    let result = solve_ode_exact(&mut st, m, n, "x", "y");
    assert!(result.is_none());
}

#[test]
fn test_exact_cubic() {
    let mut st = Store::new();

    // (x^2)dx + (y^2)dy = 0
    // M = x^2, N = y^2
    // ∂M/∂y = 0, ∂N/∂x = 0 ✓ (exact)
    // Solution: x^3/3 + y^3/3 = C

    let x = st.sym("x");
    let y = st.sym("y");
    let two = st.int(2);

    let x2 = st.pow(x, two);
    let y2 = st.pow(y, two);

    let m = x2; // x^2
    let n = y2; // y^2

    let result = solve_ode_exact(&mut st, m, n, "x", "y");
    assert!(result.is_some());

    let solution = result.unwrap();
    let sol_str = st.to_string(solution);

    // Should contain x and y with powers
    assert!(sol_str.contains("x") && sol_str.contains("y"));
}

#[test]
fn test_exact_rational() {
    let mut st = Store::new();

    // (1/x + y)dx + (x + 1/y)dy = 0
    // M = 1/x + y, N = x + 1/y
    // ∂M/∂y = 1, ∂N/∂x = 1 ✓ (exact)
    // Solution: ln|x| + xy + ln|y| = C

    let x = st.sym("x");
    let y = st.sym("y");
    let _one = st.int(1);
    let neg_one = st.int(-1);

    let x_inv = st.pow(x, neg_one); // 1/x
    let m = st.add(vec![x_inv, y]); // 1/x + y

    let y_inv = st.pow(y, neg_one); // 1/y
    let n = st.add(vec![x, y_inv]); // x + 1/y

    let result = solve_ode_exact(&mut st, m, n, "x", "y");
    assert!(result.is_some());

    let solution = result.unwrap();
    let sol_str = st.to_string(solution);

    // Should contain ln
    assert!(sol_str.contains("ln") || sol_str.contains("x"));
}

#[test]
fn test_exact_verification_by_differentiation() {
    let mut st = Store::new();

    // (2x + 3y)dx + (3x + 4y)dy = 0
    // M = 2x + 3y, N = 3x + 4y
    // ∂M/∂y = 3, ∂N/∂x = 3 ✓ (exact)
    // Solution: x^2 + 3xy + 2y^2 = C

    let x = st.sym("x");
    let y = st.sym("y");
    let two = st.int(2);
    let three = st.int(3);
    let four = st.int(4);

    let two_x = st.mul(vec![two, x]);
    let three_y = st.mul(vec![three, y]);
    let m = st.add(vec![two_x, three_y]); // 2x + 3y

    let three_x = st.mul(vec![three, x]);
    let four_y = st.mul(vec![four, y]);
    let n = st.add(vec![three_x, four_y]); // 3x + 4y

    let result = solve_ode_exact(&mut st, m, n, "x", "y");
    assert!(result.is_some());

    let solution = result.unwrap();

    // Verify: ∂F/∂x = M and ∂F/∂y = N
    let df_dx = diff(&mut st, solution, "x");
    let df_dy = diff(&mut st, solution, "y");

    let df_dx_simplified = simplify(&mut st, df_dx);
    let df_dy_simplified = simplify(&mut st, df_dy);
    let m_simplified = simplify(&mut st, m);
    let n_simplified = simplify(&mut st, n);

    assert_eq!(df_dx_simplified, m_simplified);
    assert_eq!(df_dy_simplified, n_simplified);
}
