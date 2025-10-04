//! Basic polynomial plotting example
//! Demonstrates plotting simple polynomial expressions like x^2, x^3, etc.

use expr_core::Store;
use plot::{plot_svg, PlotConfig};
use std::fs;

fn main() {
    let mut st = Store::new();
    let x = st.sym("x");

    // Example 1: Parabola f(x) = x^2
    println!("=== Example 1: Parabola x^2 ===");
    let two = st.int(2);
    let x2 = st.pow(x, two);
    let cfg = PlotConfig::new("x", -2.0, 2.0, 100, 400, 300);
    let svg = plot_svg(&st, x2, &cfg);
    fs::write("parabola.svg", &svg).expect("Failed to write parabola.svg");
    println!("Expression: {}", st.to_string(x2));
    println!("Saved to: parabola.svg\n");

    // Example 2: Cubic f(x) = x^3
    println!("=== Example 2: Cubic x^3 ===");
    let three = st.int(3);
    let x3 = st.pow(x, three);
    let cfg2 = PlotConfig::new("x", -2.0, 2.0, 100, 400, 300);
    let svg2 = plot_svg(&st, x3, &cfg2);
    fs::write("cubic.svg", &svg2).expect("Failed to write cubic.svg");
    println!("Expression: {}", st.to_string(x3));
    println!("Saved to: cubic.svg\n");

    // Example 3: Quadratic with linear term f(x) = x^2 + 2x + 1
    println!("=== Example 3: Quadratic x^2 + 2x + 1 ===");
    let one = st.int(1);
    let two_x = st.mul(vec![two, x]);
    let quadratic = st.add(vec![x2, two_x, one]);
    let cfg3 = PlotConfig::new("x", -3.0, 1.0, 100, 400, 300);
    let svg3 = plot_svg(&st, quadratic, &cfg3);
    fs::write("quadratic.svg", &svg3).expect("Failed to write quadratic.svg");
    println!("Expression: {}", st.to_string(quadratic));
    println!("Saved to: quadratic.svg\n");

    // Example 4: Higher degree polynomial f(x) = x^4 - 2x^2
    println!("=== Example 4: Quartic x^4 - 2x^2 ===");
    let four = st.int(4);
    let x4 = st.pow(x, four);
    let neg_two = st.int(-2);
    let neg_two_x2 = st.mul(vec![neg_two, x2]);
    let quartic = st.add(vec![x4, neg_two_x2]);
    let cfg4 = PlotConfig::new("x", -2.0, 2.0, 150, 400, 300);
    let svg4 = plot_svg(&st, quartic, &cfg4);
    fs::write("quartic.svg", &svg4).expect("Failed to write quartic.svg");
    println!("Expression: {}", st.to_string(quartic));
    println!("Saved to: quartic.svg\n");

    println!("All polynomial plots generated successfully!");
}
