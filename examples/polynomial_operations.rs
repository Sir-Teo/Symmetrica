//! Polynomial operations: division, GCD, partial fractions, and conversions.

use arith::Q;
use expr_core::Store;
use polys::{expr_to_unipoly, partial_fractions_simple, unipoly_to_expr, UniPoly};

fn main() {
    println!("=== Polynomial Operations ===\n");

    // Create polynomials: p(x) = x^2 + 3x + 2 and q(x) = x + 1
    let p = UniPoly::new("x", vec![Q::new(2, 1), Q::new(3, 1), Q::new(1, 1)]);
    let q = UniPoly::new("x", vec![Q::new(1, 1), Q::new(1, 1)]);

    println!("p(x) = x^2 + 3x + 2");
    println!("q(x) = x + 1");

    // Division with remainder
    let (quotient, remainder) = p.div_rem(&q).expect("division");
    println!("\nDivision: p(x) / q(x)");
    println!("  Quotient: degree {:?}, coeffs: {:?}", quotient.degree(), quotient.coeffs);
    println!("  Remainder: {:?}", remainder.coeffs);

    // GCD
    let p1 = UniPoly::new("x", vec![Q::new(-1, 1), Q::new(0, 1), Q::new(1, 1)]); // x^2 - 1
    let p2 = UniPoly::new("x", vec![Q::new(0, 1), Q::new(-1, 1), Q::new(1, 1)]); // x^2 - x
    let g = UniPoly::gcd(p1.clone(), p2.clone());
    println!("\nGCD of (x^2 - 1) and (x^2 - x):");
    println!("  GCD degree: {:?}, coeffs: {:?}", g.degree(), g.coeffs);

    // Partial fractions
    let num = UniPoly::new("x", vec![Q::new(3, 1), Q::new(2, 1)]); // 2x + 3
    let den = UniPoly::new("x", vec![Q::new(2, 1), Q::new(3, 1), Q::new(1, 1)]); // x^2 + 3x + 2
    if let Some((poly_part, terms)) = partial_fractions_simple(&num, &den) {
        println!("\nPartial fractions of (2x + 3)/(x^2 + 3x + 2):");
        println!("  Polynomial part: {:?}", poly_part.coeffs);
        println!("  Fraction terms:");
        for (a, r) in terms {
            println!("    {}/{} / (x - {}/{})", a.0, a.1, r.0, r.1);
        }
    }

    // Expr <-> UniPoly conversion
    let mut st = Store::new();
    let x = st.sym("x");
    let two = st.int(2);
    let x2 = st.pow(x, two);
    let three = st.int(3);
    let three_x = st.mul(vec![three, x]);
    let five = st.int(5);
    let expr = st.add(vec![x2, three_x, five]);

    println!("\nExpr to Polynomial conversion:");
    println!("  Expression: {}", st.to_string(expr));

    let poly = expr_to_unipoly(&st, expr, "x").expect("convertible");
    println!("  Polynomial coeffs: {:?}", poly.coeffs);

    let back = unipoly_to_expr(&mut st, &poly);
    println!("  Back to expr: {}", st.to_string(back));

    // Discriminant
    let quadratic = UniPoly::new("x", vec![Q::new(1, 1), Q::new(-2, 1), Q::new(1, 1)]); // (x-1)^2
    if let Some(disc) = quadratic.discriminant() {
        println!("\nDiscriminant of (x-1)^2 = x^2 - 2x + 1:");
        println!("  Discriminant: {}/{} (should be 0 for repeated root)", disc.0, disc.1);
    }

    println!("\n=== Done ===");
}
