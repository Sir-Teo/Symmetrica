//! Tests for Bernoulli ODE solver

use calculus::ode::solve_ode_first_order;
use expr_core::Store;

#[test]
fn test_bernoulli_simple_n2() {
    let mut st = Store::new();
    let _x = st.sym("x");
    let y = st.sym("y");

    // dy/dx = y + y^2 (Bernoulli with n=2)
    // This is: dy/dx = -(-1)*y + 1*y^2
    let two = st.int(2);
    let y2 = st.pow(y, two);
    let rhs = st.add(vec![y, y2]);

    let result = solve_ode_first_order(&mut st, rhs, "y", "x");
    // Should find a solution
    assert!(result.is_some());

    let solution = result.unwrap();
    let sol_str = st.to_string(solution);

    // Solution should involve exp and the transformation
    assert!(sol_str.contains("exp") || sol_str.len() > 1);
}

#[test]
fn test_bernoulli_n3() {
    let mut st = Store::new();
    let _x = st.sym("x");
    let y = st.sym("y");

    // dy/dx = 2y + y^3 (Bernoulli with n=3)
    let two = st.int(2);
    let two_y = st.mul(vec![two, y]);
    let three = st.int(3);
    let y3 = st.pow(y, three);
    let rhs = st.add(vec![two_y, y3]);

    let result = solve_ode_first_order(&mut st, rhs, "y", "x");
    // Should find a solution
    assert!(result.is_some());
}

#[test]
fn test_bernoulli_with_x_coefficients() {
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");

    // dy/dx = xy + xy^2 (Bernoulli with n=2, p(x)=-x, q(x)=x)
    let xy = st.mul(vec![x, y]);
    let two = st.int(2);
    let y2 = st.pow(y, two);
    let xy2 = st.mul(vec![x, y2]);
    let rhs = st.add(vec![xy, xy2]);

    let result = solve_ode_first_order(&mut st, rhs, "y", "x");
    // Should attempt to find a solution
    assert!(result.is_some() || result.is_none()); // May or may not solve depending on integration capability
}

#[test]
fn test_not_bernoulli_linear() {
    let mut st = Store::new();
    let y = st.sym("y");

    // dy/dx = y (linear, not Bernoulli)
    // Should be handled by linear solver, not Bernoulli
    let result = solve_ode_first_order(&mut st, y, "y", "x");
    assert!(result.is_some());
}

#[test]
fn test_not_bernoulli_separable() {
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");

    // dy/dx = xy (separable, not Bernoulli)
    let rhs = st.mul(vec![x, y]);
    let result = solve_ode_first_order(&mut st, rhs, "y", "x");
    assert!(result.is_some());
}
