//! Power function plotting example
//! Demonstrates plotting various power functions including fractional and negative exponents

use expr_core::Store;
use plot::{plot_svg, PlotConfig};
use std::fs;

fn main() {
    let mut st = Store::new();
    let x = st.sym("x");

    // Example 1: f(x) = x^(1/2) - Square root
    println!("=== Example 1: x^(1/2) - Square root ===");
    let half = st.rat(1, 2);
    let sqrt_x = st.pow(x, half);
    let cfg = PlotConfig::new("x", 0.0, 5.0, 100, 400, 300);
    let svg = plot_svg(&st, sqrt_x, &cfg);
    fs::write("sqrt_x.svg", &svg).expect("Failed to write sqrt_x.svg");
    println!("Expression: {}", st.to_string(sqrt_x));
    println!("Saved to: sqrt_x.svg\n");

    // Example 2: f(x) = x^(1/3) - Cube root
    println!("=== Example 2: x^(1/3) - Cube root ===");
    let third = st.rat(1, 3);
    let cbrt_x = st.pow(x, third);
    let cfg2 = PlotConfig::new("x", -5.0, 5.0, 150, 400, 300);
    let svg2 = plot_svg(&st, cbrt_x, &cfg2);
    fs::write("cbrt_x.svg", &svg2).expect("Failed to write cbrt_x.svg");
    println!("Expression: {}", st.to_string(cbrt_x));
    println!("Saved to: cbrt_x.svg\n");

    // Example 3: f(x) = x^(3/2)
    println!("=== Example 3: x^(3/2) ===");
    let three_halves = st.rat(3, 2);
    let x_three_halves = st.pow(x, three_halves);
    let cfg3 = PlotConfig::new("x", 0.0, 5.0, 100, 400, 300);
    let svg3 = plot_svg(&st, x_three_halves, &cfg3);
    fs::write("x_three_halves.svg", &svg3).expect("Failed to write x_three_halves.svg");
    println!("Expression: {}", st.to_string(x_three_halves));
    println!("Saved to: x_three_halves.svg\n");

    // Example 4: f(x) = x^(-1/2) - Inverse square root
    println!("=== Example 4: x^(-1/2) - Inverse square root ===");
    let neg_half = st.rat(-1, 2);
    let inv_sqrt_x = st.pow(x, neg_half);
    let cfg4 = PlotConfig::new("x", 0.1, 5.0, 100, 400, 300);
    let svg4 = plot_svg(&st, inv_sqrt_x, &cfg4);
    fs::write("inv_sqrt_x.svg", &svg4).expect("Failed to write inv_sqrt_x.svg");
    println!("Expression: {}", st.to_string(inv_sqrt_x));
    println!("Saved to: inv_sqrt_x.svg\n");

    // Example 5: f(x) = x^(2/3)
    println!("=== Example 5: x^(2/3) ===");
    let two_thirds = st.rat(2, 3);
    let x_two_thirds = st.pow(x, two_thirds);
    let cfg5 = PlotConfig::new("x", 0.0, 5.0, 100, 400, 300);
    let svg5 = plot_svg(&st, x_two_thirds, &cfg5);
    fs::write("x_two_thirds.svg", &svg5).expect("Failed to write x_two_thirds.svg");
    println!("Expression: {}", st.to_string(x_two_thirds));
    println!("Saved to: x_two_thirds.svg\n");

    // Example 6: f(x) = (x^2)^(1/2) - Should be |x| but we evaluate as positive branch
    println!("=== Example 6: (x^2)^(1/2) ===");
    let two = st.int(2);
    let x2 = st.pow(x, two);
    let sqrt_x2 = st.pow(x2, half);
    let cfg6 = PlotConfig::new("x", -5.0, 5.0, 150, 400, 300);
    let svg6 = plot_svg(&st, sqrt_x2, &cfg6);
    fs::write("sqrt_x_squared.svg", &svg6).expect("Failed to write sqrt_x_squared.svg");
    println!("Expression: {}", st.to_string(sqrt_x2));
    println!("Saved to: sqrt_x_squared.svg\n");

    // Example 7: f(x) = x^5
    println!("=== Example 7: x^5 - Odd high power ===");
    let five = st.int(5);
    let x5 = st.pow(x, five);
    let cfg7 = PlotConfig::new("x", -2.0, 2.0, 100, 400, 300);
    let svg7 = plot_svg(&st, x5, &cfg7);
    fs::write("x_fifth.svg", &svg7).expect("Failed to write x_fifth.svg");
    println!("Expression: {}", st.to_string(x5));
    println!("Saved to: x_fifth.svg\n");

    println!("All power function plots generated successfully!");
}
