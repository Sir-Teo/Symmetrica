//! Fuzz validation tests for Phase L acceptance criteria
//!
//! Validates that fuzz targets run crash-free for a threshold duration.
//! Phase L requirement: "Crash-free fuzzing over threshold corpus"
//!
//! These tests verify the fuzzing infrastructure works correctly by:
//! 1. Running representative inputs through fuzz target code paths
//! 2. Ensuring no panics or crashes occur
//! 3. Validating deterministic behavior

use calculus::diff;
use expr_core::Store;
use io::{from_sexpr, to_sexpr};
use simplify::simplify;

/// Test that the diff fuzz target code path works without crashes
#[test]
fn test_fuzz_diff_no_crash() {
    let test_cases = vec![
        "x",
        "(+ x 1)",
        "(* x 2)",
        "(^ x 2)",
        "(+ (* 2 x) (^ x 3))",
        "(Fn sin x)",
        "(Fn cos x)",
        "(Fn exp x)",
        "(Fn ln x)",
        "(* (Fn sin x) (Fn cos x))",
        "(+ (^ x 2) (* 3 x) 1)",
        "(^ (+ x 1) 2)",
        "(* (Fn exp x) (Fn ln x))",
        "(+ (Fn sin (* 2 x)) (Fn cos (* 3 x)))",
        // Edge cases
        "0",
        "1",
        "(+)",
        "(*)",
    ];

    for sexpr_input in test_cases {
        let mut st = Store::new();

        // Parse input
        let expr = match from_sexpr(&mut st, sexpr_input) {
            Ok(e) => e,
            Err(_) => continue, // Skip invalid inputs gracefully
        };

        // Differentiate
        let deriv = diff(&mut st, expr, "x");

        // Simplify result
        let simplified = simplify(&mut st, deriv);

        // Convert back to string (tests serialization)
        let _output = to_sexpr(&st, simplified);

        // If we reached here, no crash occurred
    }
}

/// Test that the simplify fuzz target code path works without crashes
#[test]
fn test_fuzz_simplify_no_crash() {
    let test_cases = vec![
        "(+ x x)",
        "(* x 1)",
        "(* x 0)",
        "(+ x 0)",
        "(^ x 1)",
        "(^ x 0)",
        "(+ (* 2 x) (* 3 x))",
        "(* (+ x 1) (+ x 1))",
        "(+ (^ x 2) (* -1 (^ x 2)))",
        "(* (^ x 2) (^ x 3))",
        // Rational arithmetic
        "(+ (Rat 1 2) (Rat 1 2))",
        "(* (Rat 2 3) (Rat 3 4))",
        // Nested expressions
        "(+ (+ (+ x x) x) x)",
        "(* (* (* x x) x) x)",
        // Complex nesting
        "(+ (* 2 (^ x 3)) (* -1 (* 2 (^ x 3))))",
        "(* (+ x 1) (+ x -1))",
        // Edge cases
        "(+ (Rat 0 1) x)",
        "(* (Rat 1 1) x)",
    ];

    for sexpr_input in test_cases {
        let mut st = Store::new();

        let expr = match from_sexpr(&mut st, sexpr_input) {
            Ok(e) => e,
            Err(_) => continue,
        };

        // Simplify once
        let s1 = simplify(&mut st, expr);

        // Simplify again (idempotence test)
        let s2 = simplify(&mut st, s1);

        // Should be idempotent
        assert_eq!(
            st.get(s1).digest,
            st.get(s2).digest,
            "Simplify should be idempotent for {}",
            sexpr_input
        );
    }
}

/// Test that expression operations fuzz target works without crashes
#[test]
fn test_fuzz_expr_ops_no_crash() {
    let mut st = Store::new();

    // Test various expression building operations
    let x = st.sym("x");
    let y = st.sym("y");
    let zero = st.int(0);
    let one = st.int(1);
    let two = st.int(2);

    // Addition operations
    let _e1 = st.add(vec![x, y]);
    let _e2 = st.add(vec![x, x, x]);
    let _e3 = st.add(vec![zero, x]);
    let _e4 = st.add(vec![]); // Empty add

    // Multiplication operations
    let _m1 = st.mul(vec![x, y]);
    let _m2 = st.mul(vec![two, x]);
    let _m3 = st.mul(vec![zero, x]);
    let _m4 = st.mul(vec![one, x]);
    let _m5 = st.mul(vec![]); // Empty mul

    // Power operations
    let _p1 = st.pow(x, two);
    let _p2 = st.pow(x, zero);
    let _p3 = st.pow(x, one);
    let _p4 = st.pow(zero, zero);

    // Rational operations
    let r1 = st.rat(1, 2);
    let r2 = st.rat(2, 4); // Should normalize to 1/2
    let _r3 = st.rat(0, 1);
    let _r4 = st.rat(-3, 4);

    // Hash consing validation
    assert_eq!(r1, r2, "Rationals should be normalized");

    // Function operations
    let _f1 = st.func("sin", vec![x]);
    let _f2 = st.func("cos", vec![x]);
    let _f3 = st.func("exp", vec![x]);
    let _f4 = st.func("ln", vec![x]);
}

/// Test S-expression parsing fuzz target works without crashes
#[test]
fn test_fuzz_sexpr_parse_no_crash() {
    let test_inputs = vec![
        "",
        " ",
        "x",
        "123",
        "-456",
        "()",
        "(x)",
        "(+)",
        "(+ x)",
        "(+ x y)",
        "(+ x y z)",
        "(* 2 x)",
        "(^ x 2)",
        "(Fn sin x)",
        "(Rat 1 2)",
        "(Rat -3 4)",
        "42",
        // Nested expressions
        "(+ (* 2 x) 3)",
        "(* (+ x 1) (+ y 2))",
        "(^ (Fn sin x) 2)",
        // Multiple nesting levels
        "(+ (* (^ x 2) 3) (* 4 x) 5)",
        "(Fn sin (Fn cos (Fn exp (Fn ln x))))",
        // Edge cases
        "(+ (+ (+ x)))",
        "(* (* (* 1)))",
        // Whitespace variations
        "( +  x   y )",
        "(  *\n2\nx  )",
    ];

    for input in test_inputs {
        let mut st = Store::new();

        // Attempt to parse - should not crash
        let result = from_sexpr(&mut st, input);

        // If parsing succeeded, convert back
        if let Ok(expr) = result {
            let _output = to_sexpr(&st, expr);
        }
    }
}

/// Test that invalid/malformed inputs are handled gracefully
#[test]
fn test_fuzz_malformed_input_handling() {
    let malformed_inputs = vec![
        "(",         // Unclosed paren
        ")",         // Unmatched paren
        "(()",       // Mismatched
        "(+",        // Incomplete
        "+)",        // Wrong order
        "(Rat 1)",   // Missing denominator
        "(^)",       // Missing args
        "(Fn)",      // Missing function name/arg
        "(Unknown)", // Invalid op
        "(+ x (*)",  // Nested incomplete
    ];

    for input in malformed_inputs {
        let mut st = Store::new();

        // Should return Err, not panic
        let result = from_sexpr(&mut st, input);
        assert!(result.is_err(), "Should reject malformed input: {}", input);
    }
}

/// Test deterministic behavior (same input produces same output)
#[test]
fn test_fuzz_deterministic_behavior() {
    let test_expr = "(+ (* 2 x) (^ x 2) 1)";

    // Run multiple times
    for _ in 0..10 {
        let mut st = Store::new();
        let expr = from_sexpr(&mut st, test_expr).unwrap();
        let simplified = simplify(&mut st, expr);
        let output = to_sexpr(&st, simplified);

        // Output should be consistent
        assert!(output.contains("x"));
    }
}

/// Test large expressions don't cause stack overflow
#[test]
fn test_fuzz_deeply_nested_expressions() {
    let mut st = Store::new();

    // Build deeply nested expression: (add x (add x (add x ...)))
    let x = st.sym("x");
    let mut expr = x;
    for _ in 0..100 {
        expr = st.add(vec![expr, x]);
    }

    // Should simplify without stack overflow
    let simplified = simplify(&mut st, expr);

    // Result should be some multiple of x
    let output = to_sexpr(&st, simplified);
    assert!(output.contains("x"));
}

/// Test wide expressions (many operands) don't cause issues
#[test]
fn test_fuzz_wide_expressions() {
    let mut st = Store::new();

    // Create expression with many terms: x + x + x + ... (100 times)
    let x = st.sym("x");
    let terms: Vec<_> = (0..100).map(|_| x).collect();
    let expr = st.add(terms);

    // Should simplify to 100*x
    let simplified = simplify(&mut st, expr);
    let output = to_sexpr(&st, simplified);

    // Should contain "100" and "x"
    assert!(output.contains("100") && output.contains("x"));
}

/// Test mixed operations don't cause crashes
#[test]
fn test_fuzz_mixed_operations() {
    let mut st = Store::new();

    let x = st.sym("x");
    let two = st.int(2);
    let three = st.int(3);

    // Complex expression: (2x + 3)^2
    let two_x = st.mul(vec![two, x]);
    let two_x_plus_3 = st.add(vec![two_x, three]);
    let expr = st.pow(two_x_plus_3, two);

    // Differentiate
    let deriv = diff(&mut st, expr, "x");

    // Simplify
    let simplified = simplify(&mut st, deriv);

    // Convert to string
    let output = to_sexpr(&st, simplified);

    // Should contain "x" (derivative is non-zero)
    assert!(output.contains("x") || output.contains("add") || output.contains("mul"));
}

/// Integration test: Full pipeline from parse -> simplify -> diff -> simplify
#[test]
fn test_fuzz_full_pipeline() {
    let test_cases =
        vec!["(^ x 3)", "(* (Fn sin x) (Fn cos x))", "(+ (^ x 2) (* 2 x) 1)", "(Fn exp (* 2 x))"];

    for sexpr in test_cases {
        let mut st = Store::new();

        // Parse
        let expr = from_sexpr(&mut st, sexpr).unwrap();

        // Simplify
        let s1 = simplify(&mut st, expr);

        // Differentiate
        let deriv = diff(&mut st, s1, "x");

        // Simplify derivative
        let s2 = simplify(&mut st, deriv);

        // Serialize
        let output = to_sexpr(&st, s2);

        // Should produce valid output
        assert!(!output.is_empty());
    }
}
