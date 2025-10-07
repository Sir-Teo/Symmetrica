//! I/O operations: JSON, S-expression, and LaTeX serialization.

use expr_core::Store;
use io::{from_json, from_sexpr, to_json, to_latex, to_sexpr};

fn main() {
    println!("=== I/O and Serialization ===\n");

    // Create a sample expression: (x + 1)^2 + sin(x)
    let mut st = Store::new();
    let x = st.sym("x");
    let one = st.int(1);
    let xp1 = st.add(vec![x, one]);
    let two = st.int(2);
    let squared = st.pow(xp1, two);
    let sinx = st.func("sin", vec![x]);
    let expr = st.add(vec![squared, sinx]);

    println!("Original expression: {}", st.to_string(expr));

    // S-expression serialization
    println!("\n--- S-expression Format ---");
    let sexpr = to_sexpr(&st, expr);
    println!("Serialized: {}", sexpr);

    let mut st2 = Store::new();
    let parsed_sexpr = from_sexpr(&mut st2, &sexpr).expect("parse s-expr");
    println!("Parsed back: {}", st2.to_string(parsed_sexpr));
    println!("Roundtrip matches: {}", st.to_string(expr) == st2.to_string(parsed_sexpr));

    // JSON serialization
    println!("\n--- JSON Format ---");
    let json = to_json(&st, expr);
    println!("Serialized: {}", json);

    let mut st3 = Store::new();
    let parsed_json = from_json(&mut st3, &json).expect("parse json");
    println!("Parsed back: {}", st3.to_string(parsed_json));
    println!("Roundtrip matches: {}", st.to_string(expr) == st3.to_string(parsed_json));

    // LaTeX export
    println!("\n--- LaTeX Format ---");
    let latex = to_latex(&st, expr);
    println!("LaTeX: {}", latex);

    // Example with rational numbers and more complex expression
    println!("\n--- Complex Expression with Rationals ---");
    let mut st4 = Store::new();
    let y = st4.sym("y");
    let half = st4.rat(1, 2);
    let three = st4.int(3);
    let y3 = st4.pow(y, three);
    let term1 = st4.mul(vec![half, y3]);
    let neg2 = st4.int(-2);
    let term2 = st4.mul(vec![neg2, y]);
    let five = st4.int(5);
    let complex_expr = st4.add(vec![term1, term2, five]);

    println!("Expression: {}", st4.to_string(complex_expr));

    // S-expr
    let sexpr2 = to_sexpr(&st4, complex_expr);
    println!("S-expr: {}", sexpr2);

    // JSON
    let json2 = to_json(&st4, complex_expr);
    println!("JSON: {}", json2);

    // LaTeX
    let latex2 = to_latex(&st4, complex_expr);
    println!("LaTeX: {}", latex2);

    // Verify roundtrip for complex expression
    let mut st5 = Store::new();
    let from_sexpr2 = from_sexpr(&mut st5, &sexpr2).expect("parse");
    println!(
        "\nS-expr roundtrip OK: {}",
        st4.to_string(complex_expr) == st5.to_string(from_sexpr2)
    );

    let mut st6 = Store::new();
    let from_json2 = from_json(&mut st6, &json2).expect("parse");
    println!("JSON roundtrip OK: {}", st4.to_string(complex_expr) == st6.to_string(from_json2));

    println!("\n=== Done ===");
}
