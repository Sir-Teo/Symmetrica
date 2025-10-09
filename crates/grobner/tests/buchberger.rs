//! Test cases for Buchberger algorithm

use expr_core::Store;
use grobner::{buchberger, solve_system, MonomialOrder};

#[test]
#[ignore = "grobner solver needs implementation fixes"]
fn test_buchberger_simple() {
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");
    let _one = st.int(1);
    let neg_one = st.int(-1);

    // Test system: x + y - 1 = 0, x - y = 0
    // Solution: x = 1/2, y = 1/2
    let eq1 = st.add(vec![x, y, neg_one]);
    let neg_y = st.mul(vec![neg_one, y]);
    let eq2 = st.add(vec![x, neg_y]);

    let vars = vec!["x".to_string(), "y".to_string()];
    let basis = buchberger(&mut st, vec![eq1, eq2], vars, MonomialOrder::Lex);

    // Should find Gröbner basis
    assert!(basis.len() >= 2);

    // Test solving
    let solution = solve_system(&mut st, vec![eq1, eq2], vec!["x".to_string(), "y".to_string()]);
    assert!(solution.is_some());
}

#[test]
#[ignore = "grobner solver needs implementation fixes"]
fn test_buchberger_consistent() {
    let mut st = Store::new();
    let x = st.sym("x");
    let _y = st.sym("y");
    let _two = st.int(2);

    // Inconsistent system: x = 1, x = 2
    let neg_one = st.int(-1);
    let eq1 = st.add(vec![x, neg_one]);
    let neg_two = st.int(-2);
    let eq2 = st.add(vec![x, neg_two]);

    let vars = vec!["x".to_string(), "y".to_string()];
    let solution = solve_system(&mut st, vec![eq1, eq2], vars);
    assert!(solution.is_none()); // Should detect inconsistency
}

#[test]
#[ignore = "grobner solver needs implementation fixes"]
fn test_buchberger_3_vars() {
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");
    let z = st.sym("z");
    let _one = st.int(1);
    let neg_one = st.int(-1);

    // System: x + y + z = 1, x + y = 0, x + z = 0
    // Solution: x = 1/3, y = -1/3, z = -1/3
    let eq1 = st.add(vec![x, y, z, neg_one]);
    let eq2 = st.add(vec![x, y]);
    let eq3 = st.add(vec![x, z]);

    let vars = vec!["x".to_string(), "y".to_string(), "z".to_string()];
    let basis = buchberger(&mut st, vec![eq1, eq2, eq3], vars.clone(), MonomialOrder::Lex);

    let solution = solve_system(&mut st, vec![eq1, eq2, eq3], vars);
    assert!(solution.is_some());
    assert!(basis.len() >= 3);
}
