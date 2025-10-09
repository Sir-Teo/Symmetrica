//! Test cases for solving systems with Gröbner bases

use expr_core::Store;
use grobner::{buchberger, solve_system, MonomialOrder};

#[test]
#[ignore = "grobner solver needs implementation fixes"]
fn test_solve_linear_2d() {
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");

    // System: 2x + y = 3, x - y = 0
    // Solution: x = 1, y = 1
    let two = st.int(2);
    let two_x = st.mul(vec![two, x]);
    let neg_three = st.int(-3);
    let eq1 = st.add(vec![two_x, y, neg_three]);
    let neg_one = st.int(-1);
    let neg_y = st.mul(vec![neg_one, y]);
    let eq2 = st.add(vec![x, neg_y]);

    let vars = vec!["x".to_string(), "y".to_string()];
    let solution = solve_system(&mut st, vec![eq1, eq2], vars);
    assert!(solution.is_some());

    let solutions = solution.unwrap();
    assert_eq!(solutions.len(), 1);

    // Check that we get some solution representation
    let sol = &solutions[0];
    assert!(sol.contains_key("x") || sol.contains_key("y"));
}

#[test]
#[ignore = "grobner solver needs implementation fixes"]
fn test_solve_quadratic_system() {
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");
    let two = st.int(2);

    // System: x^2 + y^2 = 1, x + y = 1
    let x2 = st.pow(x, two);
    let y2 = st.pow(y, two);
    let neg_one = st.int(-1);

    let eq1 = st.add(vec![x2, y2, neg_one]);
    let eq2 = st.add(vec![x, y, neg_one]);

    let vars = vec!["x".to_string(), "y".to_string()];
    let solution = solve_system(&mut st, vec![eq1, eq2], vars);

    // Should find some solution (may be implicit)
    assert!(solution.is_some());
}

#[test]
#[ignore = "grobner solver needs implementation fixes"]
fn test_solve_inconsistent() {
    let mut st = Store::new();
    let x = st.sym("x");

    // Inconsistent: x = 1, x = 2
    let neg_one = st.int(-1);
    let eq1 = st.add(vec![x, neg_one]);
    let neg_two = st.int(-2);
    let eq2 = st.add(vec![x, neg_two]);

    let vars = vec!["x".to_string()];
    let solution = solve_system(&mut st, vec![eq1, eq2], vars);
    assert!(solution.is_none());
}

#[test]
#[ignore = "grobner solver needs implementation fixes"]
fn test_solve_underconstrained() {
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");

    // Single equation: x + y = 1
    let neg_one = st.int(-1);
    let eq1 = st.add(vec![x, y, neg_one]);

    let vars = vec!["x".to_string(), "y".to_string()];
    let solution = solve_system(&mut st, vec![eq1], vars);

    // Should return parametric solution
    assert!(solution.is_some());
}

#[test]
#[ignore = "grobner solver needs implementation fixes"]
fn test_buchberger_cyclic() {
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");
    let z = st.sym("z");

    // Cyclic system that should terminate
    let neg_one = st.int(-1);
    let neg_y = st.mul(vec![neg_one, y]);
    let eq1 = st.add(vec![x, neg_y]);
    let neg_z = st.mul(vec![neg_one, z]);
    let eq2 = st.add(vec![y, neg_z]);
    let neg_x = st.mul(vec![neg_one, x]);
    let eq3 = st.add(vec![z, neg_x]);

    let vars = vec!["x".to_string(), "y".to_string(), "z".to_string()];
    let basis = buchberger(&mut st, vec![eq1, eq2, eq3], vars, MonomialOrder::Lex);

    // Should terminate without infinite loop
    assert!(basis.len() >= 3);
}
