//! Regression tests from fuzzing findings

use expr_core::Store;
use simplify::simplify;

#[test]
fn test_idempotence_x_times_x_plus_x() {
    // Fuzzing found: x * (x + x) was not idempotent
    // Input bytes: [211, 1, 0, 2, 33, 0, 1, 96]
    let mut store = Store::new();

    let x = store.sym("x");
    let x_plus_x = store.add(vec![x, x]);
    let expr = store.mul(vec![x, x_plus_x]);

    eprintln!("Original: {}", store.to_string(expr));

    let s1 = simplify(&mut store, expr);
    eprintln!("After 1st simplify: {}", store.to_string(s1));

    let s2 = simplify(&mut store, s1);
    eprintln!("After 2nd simplify: {}", store.to_string(s2));

    // Simplification must be idempotent
    assert_eq!(
        store.get(s1).digest,
        store.get(s2).digest,
        "Simplify should be idempotent: simplify(simplify(e)) == simplify(e)"
    );
}
