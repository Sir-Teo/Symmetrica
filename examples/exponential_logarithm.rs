//! Exponential and logarithmic function plotting example
//! Demonstrates plotting exp and ln functions

use expr_core::Store;
use plot::{plot_svg, PlotConfig};
use std::fs;

fn main() {
    let mut st = Store::new();
    let x = st.sym("x");

    // Example 1: f(x) = exp(x)
    println!("=== Example 1: exp(x) ===");
    let expx = st.func("exp", vec![x]);
    let cfg = PlotConfig::new("x", -2.0, 2.0, 100, 400, 300);
    let svg = plot_svg(&st, expx, &cfg);
    fs::write("exp_x.svg", &svg).expect("Failed to write exp_x.svg");
    println!("Expression: {}", st.to_string(expx));
    println!("Saved to: exp_x.svg\n");

    // Example 2: f(x) = ln(x)
    println!("=== Example 2: ln(x) ===");
    let lnx = st.func("ln", vec![x]);
    let cfg2 = PlotConfig::new("x", 0.1, 5.0, 100, 400, 300); // x > 0 for ln
    let svg2 = plot_svg(&st, lnx, &cfg2);
    fs::write("ln_x.svg", &svg2).expect("Failed to write ln_x.svg");
    println!("Expression: {}", st.to_string(lnx));
    println!("Saved to: ln_x.svg\n");

    // Example 3: f(x) = exp(-x)
    println!("=== Example 3: exp(-x) - Exponential decay ===");
    let neg_one = st.int(-1);
    let neg_x = st.mul(vec![neg_one, x]);
    let exp_neg_x = st.func("exp", vec![neg_x]);
    let cfg3 = PlotConfig::new("x", -2.0, 4.0, 100, 400, 300);
    let svg3 = plot_svg(&st, exp_neg_x, &cfg3);
    fs::write("exp_neg_x.svg", &svg3).expect("Failed to write exp_neg_x.svg");
    println!("Expression: {}", st.to_string(exp_neg_x));
    println!("Saved to: exp_neg_x.svg\n");

    // Example 4: f(x) = exp(x^2)
    println!("=== Example 4: exp(x^2) - Gaussian-like ===");
    let two = st.int(2);
    let x2 = st.pow(x, two);
    let exp_x2 = st.func("exp", vec![x2]);
    let cfg4 = PlotConfig::new("x", -2.0, 2.0, 100, 400, 300);
    let svg4 = plot_svg(&st, exp_x2, &cfg4);
    fs::write("exp_x_squared.svg", &svg4).expect("Failed to write exp_x_squared.svg");
    println!("Expression: {}", st.to_string(exp_x2));
    println!("Saved to: exp_x_squared.svg\n");

    // Example 5: f(x) = ln(x^2)
    println!("=== Example 5: ln(x^2) ===");
    let ln_x2 = st.func("ln", vec![x2]);
    let cfg5 = PlotConfig::new("x", 0.1, 5.0, 100, 400, 300);
    let svg5 = plot_svg(&st, ln_x2, &cfg5);
    fs::write("ln_x_squared.svg", &svg5).expect("Failed to write ln_x_squared.svg");
    println!("Expression: {}", st.to_string(ln_x2));
    println!("Saved to: ln_x_squared.svg\n");

    // Example 6: f(x) = x * exp(-x)
    println!("=== Example 6: x * exp(-x) - Product ===");
    let x_exp_neg_x = st.mul(vec![x, exp_neg_x]);
    let cfg6 = PlotConfig::new("x", -1.0, 5.0, 100, 400, 300);
    let svg6 = plot_svg(&st, x_exp_neg_x, &cfg6);
    fs::write("x_exp_neg_x.svg", &svg6).expect("Failed to write x_exp_neg_x.svg");
    println!("Expression: {}", st.to_string(x_exp_neg_x));
    println!("Saved to: x_exp_neg_x.svg\n");

    // Example 7: f(x) = exp(sin(x))
    println!("=== Example 7: exp(sin(x)) - Composition ===");
    let sinx = st.func("sin", vec![x]);
    let exp_sinx = st.func("exp", vec![sinx]);
    let cfg7 = PlotConfig::new("x", -6.28, 6.28, 200, 600, 300);
    let svg7 = plot_svg(&st, exp_sinx, &cfg7);
    fs::write("exp_sin_x.svg", &svg7).expect("Failed to write exp_sin_x.svg");
    println!("Expression: {}", st.to_string(exp_sinx));
    println!("Saved to: exp_sin_x.svg\n");

    println!("All exponential and logarithmic plots generated successfully!");
}
