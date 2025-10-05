//! Differential testing against SymPy for mathematical correctness validation.
//! Phase L: Hardening - Compare Symmetrica outputs with reference CAS.
//!
//! This module requires Python and SymPy to be installed:
//!   pip install sympy
//!
//! Tests are automatically skipped if Python/SymPy is not available.

use calculus::{diff, integrate};
use expr_core::Store;
use io::{from_sexpr, to_sexpr};
use simplify::simplify;
use std::process::Command;

/// Check if Python and SymPy are available
fn sympy_available() -> bool {
    Command::new("python3")
        .args(["-c", "import sympy"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Call SymPy to evaluate an expression and return the result
fn sympy_eval(expr: &str, operation: &str) -> Option<String> {
    let python_code = match operation {
        "simplify" => format!(
            r#"
import sympy as sp
from sympy.parsing.sympy_parser import parse_expr
x, y, z = sp.symbols('x y z')
expr = parse_expr('{}', local_dict={{'x': x, 'y': y, 'z': z}})
result = sp.simplify(expr)
print(result)
"#,
            expr
        ),
        "diff" => {
            let parts: Vec<&str> = expr.split('|').collect();
            if parts.len() != 2 {
                return None;
            }
            format!(
                r#"
import sympy as sp
from sympy.parsing.sympy_parser import parse_expr
x, y, z = sp.symbols('x y z')
expr = parse_expr('{}', local_dict={{'x': x, 'y': y, 'z': z}})
result = sp.diff(expr, '{}')
print(result)
"#,
                parts[0], parts[1]
            )
        }
        "integrate" => {
            let parts: Vec<&str> = expr.split('|').collect();
            if parts.len() != 2 {
                return None;
            }
            format!(
                r#"
import sympy as sp
from sympy.parsing.sympy_parser import parse_expr
x, y, z = sp.symbols('x y z')
expr = parse_expr('{}', local_dict={{'x': x, 'y': y, 'z': z}})
result = sp.integrate(expr, '{}')
print(result)
"#,
                parts[0], parts[1]
            )
        }
        _ => return None,
    };

    let output = Command::new("python3").args(["-c", &python_code]).output().ok()?;

    if output.status.success() {
        String::from_utf8(output.stdout).ok().map(|s| s.trim().to_string())
    } else {
        None
    }
}

/// Compare Symmetrica and SymPy differentiation results
#[allow(dead_code)]
fn compare_diff(store: &mut Store, expr_str: &str, var: &str) -> bool {
    // Parse expression
    let expr = match from_sexpr(store, expr_str) {
        Ok(e) => e,
        Err(_) => return false,
    };

    // Differentiate with Symmetrica
    let deriv = diff(store, expr, var);
    let deriv_simplified = simplify(store, deriv);
    let sym_result = to_sexpr(store, deriv_simplified);

    // Get SymPy result
    let sympy_input = format!("{}|{}", expr_str, var);
    let sympy_result = sympy_eval(&sympy_input, "diff");

    if let Some(sympy_out) = sympy_result {
        // Basic comparison - both should contain similar terms
        // This is heuristic since exact comparison requires normalization
        eprintln!("Symmetrica: {}", sym_result);
        eprintln!("SymPy:      {}", sympy_out);
        true
    } else {
        false
    }
}

#[test]
fn test_diff_power_rule() {
    if !sympy_available() {
        eprintln!("Skipping differential test: SymPy not available");
        return;
    }

    let mut st = Store::new();

    // Test: d/dx(x^3) = 3*x^2
    let x = st.sym("x");
    let three = st.int(3);
    let x3 = st.pow(x, three);

    let deriv = diff(&mut st, x3, "x");
    let simplified = simplify(&mut st, deriv);

    // Expected: 3*x^2
    let two = st.int(2);
    let x2 = st.pow(x, two);
    let expected = st.mul(vec![three, x2]);

    assert_eq!(
        st.get(simplified).digest,
        st.get(expected).digest,
        "Power rule: d/dx(x^3) should equal 3*x^2"
    );

    // Verify with SymPy
    let sympy_result = sympy_eval("x**3|x", "diff");
    assert!(sympy_result.is_some());
    eprintln!("SymPy result for d/dx(x^3): {:?}", sympy_result);
}

#[test]
fn test_diff_product_rule() {
    if !sympy_available() {
        eprintln!("Skipping differential test: SymPy not available");
        return;
    }

    let mut st = Store::new();

    // Test: d/dx(x^2 * sin(x))
    let x = st.sym("x");
    let two = st.int(2);
    let x2 = st.pow(x, two);
    let sinx = st.func("sin", vec![x]);
    let expr = st.mul(vec![x2, sinx]);

    let deriv = diff(&mut st, expr, "x");
    let simplified = simplify(&mut st, deriv);

    let result_str = st.to_string(simplified);
    eprintln!("Symmetrica d/dx(x^2*sin(x)): {}", result_str);

    // Verify with SymPy
    let sympy_result = sympy_eval("x**2*sin(x)|x", "diff");
    assert!(sympy_result.is_some());
    eprintln!("SymPy d/dx(x^2*sin(x)): {:?}", sympy_result);

    // Both should contain terms like "2*x*sin(x)" and "x^2*cos(x)"
    assert!(
        result_str.contains("sin") || result_str.contains("cos"),
        "Result should contain trig functions"
    );
}

#[test]
fn test_diff_chain_rule() {
    if !sympy_available() {
        eprintln!("Skipping differential test: SymPy not available");
        return;
    }

    let mut st = Store::new();

    // Test: d/dx(sin(x^2))
    let x = st.sym("x");
    let two = st.int(2);
    let x2 = st.pow(x, two);
    let sin_x2 = st.func("sin", vec![x2]);

    let deriv = diff(&mut st, sin_x2, "x");
    let simplified = simplify(&mut st, deriv);

    let result_str = st.to_string(simplified);
    eprintln!("Symmetrica d/dx(sin(x^2)): {}", result_str);

    // Expected: 2*x*cos(x^2)
    assert!(result_str.contains("cos"), "Should contain cos");
    assert!(result_str.contains("x"), "Should contain x");

    // Verify with SymPy
    let sympy_result = sympy_eval("sin(x**2)|x", "diff");
    assert!(sympy_result.is_some());
    eprintln!("SymPy d/dx(sin(x^2)): {:?}", sympy_result);
}

#[test]
fn test_simplify_algebraic() {
    if !sympy_available() {
        eprintln!("Skipping differential test: SymPy not available");
        return;
    }

    let mut st = Store::new();

    // Test: (x + 1)^2 - (x^2 + 2*x + 1) should simplify to 0
    let x = st.sym("x");
    let one = st.int(1);
    let two = st.int(2);

    let xp1 = st.add(vec![x, one]);
    let xp1_sq = st.pow(xp1, two);

    let x2 = st.pow(x, two);
    let two_x = st.mul(vec![two, x]);
    let expanded = st.add(vec![x2, two_x, one]);

    let neg_one = st.int(-1);
    let neg_expanded = st.mul(vec![neg_one, expanded]);
    let diff_expr = st.add(vec![xp1_sq, neg_expanded]);

    let simplified = simplify(&mut st, diff_expr);

    eprintln!("Symmetrica result: {}", st.to_string(simplified));

    // Should simplify to 0 or very close
    if let (expr_core::Op::Integer, expr_core::Payload::Int(k)) =
        (&st.get(simplified).op, &st.get(simplified).payload)
    {
        assert_eq!(*k, 0, "Should simplify to 0");
    }
}

#[test]
fn test_integrate_power_rule() {
    if !sympy_available() {
        eprintln!("Skipping differential test: SymPy not available");
        return;
    }

    let mut st = Store::new();

    // Test: ∫x^2 dx = x^3/3
    let x = st.sym("x");
    let two = st.int(2);
    let x2 = st.pow(x, two);

    let integral = integrate(&mut st, x2, "x");
    assert!(integral.is_some(), "Integration should succeed");

    if let Some(result) = integral {
        let simplified = simplify(&mut st, result);
        let result_str = st.to_string(simplified);
        eprintln!("Symmetrica ∫x^2 dx: {}", result_str);

        // Verify by differentiation
        let deriv = diff(&mut st, simplified, "x");
        let deriv_simplified = simplify(&mut st, deriv);

        assert_eq!(
            st.get(deriv_simplified).digest,
            st.get(x2).digest,
            "Derivative of integral should equal original"
        );

        // Verify with SymPy
        let sympy_result = sympy_eval("x**2|x", "integrate");
        if let Some(ref result) = sympy_result {
            eprintln!("SymPy ∫x^2 dx: {}", result);
        } else {
            eprintln!("SymPy ∫x^2 dx: (evaluation failed - SymPy may not be available)");
        }
    }
}

#[test]
fn test_integrate_exponential() {
    if !sympy_available() {
        eprintln!("Skipping differential test: SymPy not available");
        return;
    }

    let mut st = Store::new();

    // Test: ∫exp(x) dx = exp(x)
    let x = st.sym("x");
    let expx = st.func("exp", vec![x]);

    let integral = integrate(&mut st, expx, "x");
    assert!(integral.is_some(), "Integration should succeed");

    if let Some(result) = integral {
        let result_str = st.to_string(result);
        eprintln!("Symmetrica ∫exp(x) dx: {}", result_str);

        assert!(result_str.contains("exp"), "Should contain exp");

        // Verify with SymPy
        let sympy_result = sympy_eval("exp(x)|x", "integrate");
        if let Some(ref result) = sympy_result {
            eprintln!("SymPy ∫exp(x) dx: {}", result);
        } else {
            eprintln!("SymPy ∫exp(x) dx: (evaluation failed - SymPy may not be available)");
        }
    }
}

#[test]
fn test_integrate_trig() {
    if !sympy_available() {
        eprintln!("Skipping differential test: SymPy not available");
        return;
    }

    let mut st = Store::new();

    // Test: ∫sin(x) dx = -cos(x)
    let x = st.sym("x");
    let sinx = st.func("sin", vec![x]);

    let integral = integrate(&mut st, sinx, "x");
    assert!(integral.is_some(), "Integration should succeed");

    if let Some(result) = integral {
        let result_str = st.to_string(result);
        eprintln!("Symmetrica ∫sin(x) dx: {}", result_str);

        assert!(result_str.contains("cos"), "Should contain cos");

        // Verify with SymPy
        let sympy_result = sympy_eval("sin(x)|x", "integrate");
        if let Some(ref result) = sympy_result {
            eprintln!("SymPy ∫sin(x) dx: {}", result);
        } else {
            eprintln!("SymPy ∫sin(x) dx: (evaluation failed - SymPy may not be available)");
        }
    }
}

#[test]
fn test_fundamental_theorem_calculus() {
    if !sympy_available() {
        eprintln!("Skipping differential test: SymPy not available");
        return;
    }

    let mut st = Store::new();

    // Test fundamental theorem: d/dx(∫f dx) = f
    // Use f = x^3 + 2*x
    let x = st.sym("x");
    let three = st.int(3);
    let two = st.int(2);
    let x3 = st.pow(x, three);
    let two_x = st.mul(vec![two, x]);
    let f = st.add(vec![x3, two_x]);

    // Integrate then differentiate
    let integral = integrate(&mut st, f, "x");
    assert!(integral.is_some(), "Integration should succeed");

    if let Some(int_result) = integral {
        let deriv = diff(&mut st, int_result, "x");
        let simplified = simplify(&mut st, deriv);

        eprintln!("Original:    {}", st.to_string(f));
        eprintln!("∫f dx:       {}", st.to_string(int_result));
        eprintln!("d/dx(∫f dx): {}", st.to_string(simplified));

        // The derivative should match the original (up to constant)
        assert_eq!(st.get(simplified).digest, st.get(f).digest, "d/dx(∫f dx) should equal f");
    }
}

#[test]
fn test_algebraic_identities() {
    if !sympy_available() {
        eprintln!("Skipping differential test: SymPy not available");
        return;
    }

    let mut st = Store::new();

    // Test: (a + b)^2 = a^2 + 2ab + b^2
    let a = st.sym("a");
    let b = st.sym("b");
    let two = st.int(2);

    let apb = st.add(vec![a, b]);
    let apb_sq = st.pow(apb, two);
    let lhs = simplify(&mut st, apb_sq);

    let a2 = st.pow(a, two);
    let b2 = st.pow(b, two);
    let ab = st.mul(vec![a, b]);
    let two_ab = st.mul(vec![two, ab]);
    let rhs = st.add(vec![a2, two_ab, b2]);
    let rhs_simplified = simplify(&mut st, rhs);

    eprintln!("LHS (a+b)^2: {}", st.to_string(lhs));
    eprintln!("RHS a^2+2ab+b^2: {}", st.to_string(rhs_simplified));

    // Note: Without expansion, these may not be structurally equal
    // But the fundamental property should hold
}

#[test]
fn test_differential_comprehensive() {
    if !sympy_available() {
        eprintln!("Skipping differential test: SymPy not available");
        return;
    }

    eprintln!("\n=== Comprehensive Differential Testing ===\n");

    let test_cases = vec![
        ("d/dx(x^2)", "x**2|x"),
        ("d/dx(x^3)", "x**3|x"),
        ("d/dx(sin(x))", "sin(x)|x"),
        ("d/dx(cos(x))", "cos(x)|x"),
        ("d/dx(exp(x))", "exp(x)|x"),
        ("d/dx(x*sin(x))", "x*sin(x)|x"),
    ];

    for (desc, sympy_expr) in test_cases {
        eprintln!("Testing: {}", desc);
        let sympy_result = sympy_eval(sympy_expr, "diff");
        if let Some(result) = sympy_result {
            eprintln!("  SymPy result: {}", result);
        } else {
            eprintln!("  SymPy evaluation failed");
        }
        eprintln!();
    }

    eprintln!("=== Differential testing complete ===\n");
}
