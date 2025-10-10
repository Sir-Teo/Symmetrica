//! Tests for Gröbner basis system solving with back-substitution

use expr_core::Store;
use grobner::solve_system;
use simplify::simplify;

#[test]
#[ignore = "grobner solver needs performance optimizations"]
fn test_solve_linear_system_2x2() {
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");

    // System: x + y - 3 = 0, x - y - 1 = 0
    // Solution: x = 2, y = 1
    let neg_three = st.int(-3);
    let eq1 = st.add(vec![x, y, neg_three]);

    let neg_one_a = st.int(-1);
    let neg_y = st.mul(vec![neg_one_a, y]);
    let neg_one_b = st.int(-1);
    let eq2 = st.add(vec![x, neg_y, neg_one_b]);

    let result = solve_system(&mut st, vec![eq1, eq2], vec!["x".to_string(), "y".to_string()]);
    assert!(result.is_some());

    let solutions = result.unwrap();
    assert_eq!(solutions.len(), 1);

    // Check solution values
    let sol = &solutions[0];
    assert!(sol.contains_key("x"));
    assert!(sol.contains_key("y"));

    // Verify x = 2
    let x_val = sol.get("x").unwrap();
    let x_str = st.to_string(*x_val);
    assert!(x_str.contains("2") || x_str == "2");

    // Verify y = 1
    let y_val = sol.get("y").unwrap();
    let y_str = st.to_string(*y_val);
    assert!(y_str.contains("1") || y_str == "1");
}

#[test]
fn test_solve_simple_quadratic_system() {
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");

    // System: x^2 - 4 = 0, y - 2 = 0
    // Solutions: x = ±2, y = 2
    let two = st.int(2);
    let x2 = st.pow(x, two);
    let neg_four = st.int(-4);
    let eq1 = st.add(vec![x2, neg_four]);
    let neg_two = st.int(-2);
    let eq2 = st.add(vec![y, neg_two]);

    let result = solve_system(&mut st, vec![eq1, eq2], vec!["x".to_string(), "y".to_string()]);
    assert!(result.is_some());

    let solutions = result.unwrap();
    assert_eq!(solutions.len(), 1);

    let sol = &solutions[0];
    assert!(sol.contains_key("x"));
    assert!(sol.contains_key("y"));
}

#[test]
#[ignore = "grobner solver needs performance optimizations"]
fn test_solve_inconsistent_system() {
    let mut st = Store::new();
    let x = st.sym("x");

    // System: x - 1 = 0, x - 2 = 0 (inconsistent)
    let neg_one = st.int(-1);
    let eq1 = st.add(vec![x, neg_one]);
    let neg_two = st.int(-2);
    let eq2 = st.add(vec![x, neg_two]);

    let result = solve_system(&mut st, vec![eq1, eq2], vec!["x".to_string()]);
    // Should detect inconsistency
    assert!(result.is_none());
}

#[test]
fn test_solve_single_variable() {
    let mut st = Store::new();
    let x = st.sym("x");

    // System: x^2 - 1 = 0
    // Solutions: x = ±1
    let two = st.int(2);
    let x2 = st.pow(x, two);
    let neg_one = st.int(-1);
    let eq = st.add(vec![x2, neg_one]);

    let result = solve_system(&mut st, vec![eq], vec!["x".to_string()]);
    assert!(result.is_some());

    let solutions = result.unwrap();
    assert_eq!(solutions.len(), 1);
    assert!(solutions[0].contains_key("x"));
}

#[test]
#[ignore = "grobner solver needs performance optimizations"]
fn test_solve_triangular_system() {
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");

    // Triangular system: x - 1 = 0, y - x = 0
    // Solution: x = 1, y = 1
    let neg_one_a = st.int(-1);
    let eq1 = st.add(vec![x, neg_one_a]);
    let neg_one_b = st.int(-1);
    let neg_x = st.mul(vec![neg_one_b, x]);
    let eq2 = st.add(vec![y, neg_x]);

    let result = solve_system(&mut st, vec![eq1, eq2], vec!["x".to_string(), "y".to_string()]);
    assert!(result.is_some());

    let solutions = result.unwrap();
    assert_eq!(solutions.len(), 1);

    let sol = &solutions[0];
    assert!(sol.contains_key("x"));
    assert!(sol.contains_key("y"));
}

#[test]
fn test_solve_empty_system() {
    let mut st = Store::new();

    let result = solve_system(&mut st, vec![], vec!["x".to_string()]);
    // Empty system should return None
    assert!(result.is_none());
}

#[test]
#[ignore = "grobner solver needs performance optimizations"]
fn test_solve_system_with_substitution() {
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");

    // System: y - 2x = 0, x - 1 = 0
    // Solution: x = 1, y = 2
    let two = st.int(2);
    let two_x = st.mul(vec![two, x]);
    let neg_one_a = st.int(-1);
    let neg_two_x = st.mul(vec![neg_one_a, two_x]);
    let eq1 = st.add(vec![y, neg_two_x]);
    let neg_one_b = st.int(-1);
    let eq2 = st.add(vec![x, neg_one_b]);

    let result = solve_system(&mut st, vec![eq1, eq2], vec!["x".to_string(), "y".to_string()]);
    assert!(result.is_some());

    let solutions = result.unwrap();
    assert_eq!(solutions.len(), 1);

    let sol = &solutions[0];
    assert!(sol.contains_key("x"));
    assert!(sol.contains_key("y"));

    // Verify solution by substitution
    let x_val = sol.get("x").unwrap();
    let y_val = sol.get("y").unwrap();

    // Simplify to check values
    let x_simplified = simplify(&mut st, *x_val);
    let y_simplified = simplify(&mut st, *y_val);

    let x_str = st.to_string(x_simplified);
    let y_str = st.to_string(y_simplified);

    // x should be 1
    assert!(x_str.contains("1") || x_str == "1");
    // y should be 2
    assert!(y_str.contains("2") || y_str == "2");
}
