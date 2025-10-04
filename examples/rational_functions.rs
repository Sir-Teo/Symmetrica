//! Rational function plotting example
//! Demonstrates plotting rational functions (ratios of polynomials)

use expr_core::Store;
use plot::{plot_svg, PlotConfig};
use std::fs;

fn main() {
    let mut st = Store::new();
    let x = st.sym("x");

    // Example 1: f(x) = 1/x
    println!("=== Example 1: 1/x - Reciprocal ===");
    let neg_one = st.int(-1);
    let inv_x = st.pow(x, neg_one);
    let cfg = PlotConfig::new("x", -5.0, 5.0, 200, 400, 300);
    let svg = plot_svg(&st, inv_x, &cfg);
    fs::write("reciprocal.svg", &svg).expect("Failed to write reciprocal.svg");
    println!("Expression: {}", st.to_string(inv_x));
    println!("Saved to: reciprocal.svg");
    println!("Note: Discontinuity at x=0 results in gaps\n");

    // Example 2: f(x) = 1/x^2
    println!("=== Example 2: 1/x^2 ===");
    let neg_two = st.int(-2);
    let inv_x2 = st.pow(x, neg_two);
    let cfg2 = PlotConfig::new("x", -5.0, 5.0, 200, 400, 300);
    let svg2 = plot_svg(&st, inv_x2, &cfg2);
    fs::write("reciprocal_squared.svg", &svg2).expect("Failed to write reciprocal_squared.svg");
    println!("Expression: {}", st.to_string(inv_x2));
    println!("Saved to: reciprocal_squared.svg\n");

    // Example 3: f(x) = x / (x^2 + 1)
    println!("=== Example 3: x / (x^2 + 1) ===");
    let two = st.int(2);
    let x2 = st.pow(x, two);
    let one = st.int(1);
    let x2_plus_1 = st.add(vec![x2, one]);
    let inv_x2_plus_1 = st.pow(x2_plus_1, neg_one);
    let x_over_x2_plus_1 = st.mul(vec![x, inv_x2_plus_1]);
    let cfg3 = PlotConfig::new("x", -5.0, 5.0, 150, 400, 300);
    let svg3 = plot_svg(&st, x_over_x2_plus_1, &cfg3);
    fs::write("x_over_x2_plus_1.svg", &svg3).expect("Failed to write x_over_x2_plus_1.svg");
    println!("Expression: {}", st.to_string(x_over_x2_plus_1));
    println!("Saved to: x_over_x2_plus_1.svg\n");

    // Example 4: f(x) = (x^2 - 1) / (x^2 + 1)
    println!("=== Example 4: (x^2 - 1) / (x^2 + 1) ===");
    let minus_one = st.int(-1);
    let x2_minus_1 = st.add(vec![x2, minus_one]);
    let rational = st.mul(vec![x2_minus_1, inv_x2_plus_1]);
    let cfg4 = PlotConfig::new("x", -5.0, 5.0, 150, 400, 300);
    let svg4 = plot_svg(&st, rational, &cfg4);
    fs::write("rational_x2_minus_1_over_x2_plus_1.svg", &svg4)
        .expect("Failed to write rational_x2_minus_1_over_x2_plus_1.svg");
    println!("Expression: {}", st.to_string(rational));
    println!("Saved to: rational_x2_minus_1_over_x2_plus_1.svg\n");

    // Example 5: f(x) = 1 / (1 + x^2) - Witch of Agnesi
    println!("=== Example 5: 1 / (1 + x^2) - Witch of Agnesi ===");
    let one_plus_x2 = st.add(vec![one, x2]);
    let witch = st.pow(one_plus_x2, neg_one);
    let cfg5 = PlotConfig::new("x", -5.0, 5.0, 150, 400, 300);
    let svg5 = plot_svg(&st, witch, &cfg5);
    fs::write("witch_of_agnesi.svg", &svg5).expect("Failed to write witch_of_agnesi.svg");
    println!("Expression: {}", st.to_string(witch));
    println!("Saved to: witch_of_agnesi.svg\n");
    // Example 6: f(x) = x^3 / (x^2 + 1)
    println!("=== Example 6: x^3 / (x^2 + 1) ===");
    let three = st.int(3);
    let x3 = st.pow(x, three);
    let x3_over_x2_plus_1 = st.mul(vec![x3, inv_x2_plus_1]);
    let cfg6 = PlotConfig::new("x", -5.0, 5.0, 150, 400, 300);
    let svg6 = plot_svg(&st, x3_over_x2_plus_1, &cfg6);
    fs::write("x3_over_x2_plus_1.svg", &svg6).expect("Failed to write x3_over_x2_plus_1.svg");
    println!("Expression: {}", st.to_string(x3_over_x2_plus_1));
    println!("Saved to: x3_over_x2_plus_1.svg\n");

    println!("All rational function plots generated successfully!");
}
