//! Comprehensive demo showing integration of multiple Symmetrica features
//! Demonstrates building, simplifying, differentiating, and plotting expressions

use calculus::diff;
use expr_core::Store;
use io::latex::to_latex;
use pattern::subst_symbol;
use plot::{plot_svg, PlotConfig};
use simplify::simplify;
use std::fs;

fn main() {
    println!("=== Comprehensive Symmetrica Plotting Demo ===\n");

    let mut st = Store::new();
    let x = st.sym("x");

    // Build a complex expression: (x+1)^2 * sin(x) / (x^2 + 1)
    println!("=== Step 1: Building a complex expression ===");
    let one = st.int(1);
    let two = st.int(2);
    let x_plus_1 = st.add(vec![x, one]);
    let x_plus_1_sq = st.pow(x_plus_1, two);
    let sinx = st.func("sin", vec![x]);
    let x2 = st.pow(x, two);
    let x2_plus_1 = st.add(vec![x2, one]);

    let neg_one = st.int(-1);
    let inv_x2_plus_1 = st.pow(x2_plus_1, neg_one);

    let numerator = st.mul(vec![x_plus_1_sq, sinx]);
    let expr = st.mul(vec![numerator, inv_x2_plus_1]);

    println!("Original expression:");
    println!("  ASCII: {}", st.to_string(expr));
    println!("  LaTeX: {}", to_latex(&st, expr));
    println!();

    // Simplify
    println!("=== Step 2: Simplifying ===");
    let simp = simplify(&mut st, expr);
    println!("Simplified:");
    println!("  ASCII: {}", st.to_string(simp));
    println!();

    // Plot original
    println!("=== Step 3: Plotting original expression ===");
    let cfg = PlotConfig::new("x", -5.0, 5.0, 200, 600, 400);
    let svg = plot_svg(&st, simp, &cfg);
    fs::write("comprehensive_original.svg", &svg)
        .expect("Failed to write comprehensive_original.svg");
    println!("Saved: comprehensive_original.svg\n");

    // Differentiate
    println!("=== Step 4: Computing derivative ===");
    let deriv = diff(&mut st, simp, "x");
    let deriv_simp = simplify(&mut st, deriv);
    println!("Derivative:");
    println!("  ASCII: {}", st.to_string(deriv_simp));
    println!();

    // Plot derivative
    println!("=== Step 5: Plotting derivative ===");
    let svg_deriv = plot_svg(&st, deriv_simp, &cfg);
    fs::write("comprehensive_derivative.svg", &svg_deriv)
        .expect("Failed to write comprehensive_derivative.svg");
    println!("Saved: comprehensive_derivative.svg\n");

    // Substitute x -> 2x
    println!("=== Step 6: Substitution (x -> 2x) ===");
    let two_x = st.mul(vec![two, x]);
    let subst_expr = subst_symbol(&mut st, simp, "x", two_x);
    let subst_simp = simplify(&mut st, subst_expr);
    println!("After substituting x -> 2x:");
    println!("  ASCII: {}", st.to_string(subst_simp));
    println!();

    // Plot substituted
    println!("=== Step 7: Plotting substituted expression ===");
    let svg_subst = plot_svg(&st, subst_simp, &cfg);
    fs::write("comprehensive_substituted.svg", &svg_subst)
        .expect("Failed to write comprehensive_substituted.svg");
    println!("Saved: comprehensive_substituted.svg\n");

    // Create a comparison plot with multiple related expressions
    println!("=== Step 8: Creating comparison expressions ===");

    // Simple polynomial for comparison
    let three = st.int(3);
    let x3 = st.pow(x, three);
    let neg_two = st.int(-2);
    let neg_two_x = st.mul(vec![neg_two, x]);
    let poly = st.add(vec![x3, neg_two_x]);
    println!("Comparison polynomial: {}", st.to_string(poly));

    let cfg_comp = PlotConfig::new("x", -3.0, 3.0, 150, 600, 400);
    let svg_poly = plot_svg(&st, poly, &cfg_comp);
    fs::write("comprehensive_comparison_poly.svg", &svg_poly)
        .expect("Failed to write comprehensive_comparison_poly.svg");
    println!("Saved: comprehensive_comparison_poly.svg\n");

    // Trigonometric for comparison
    let trig = st.mul(vec![x, sinx]);
    println!("Comparison trig: {}", st.to_string(trig));

    let svg_trig = plot_svg(&st, trig, &cfg_comp);
    fs::write("comprehensive_comparison_trig.svg", &svg_trig)
        .expect("Failed to write comprehensive_comparison_trig.svg");
    println!("Saved: comprehensive_comparison_trig.svg\n");

    println!("=== Summary ===");
    println!("This demo showcased:");
    println!("✓ Building complex expressions from primitives");
    println!("✓ Simplification");
    println!("✓ LaTeX output formatting");
    println!("✓ Plotting with SVG output");
    println!("✓ Symbolic differentiation");
    println!("✓ Pattern substitution");
    println!("✓ Multiple related visualizations");
    println!();
    println!("All files saved in current directory!");
    println!("Open the .svg files in a web browser to view the plots.");
}
