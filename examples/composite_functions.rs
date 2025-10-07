//! Composite function plotting example
//! Demonstrates plotting complex compositions of functions

#![allow(clippy::approx_constant)]

use expr_core::Store;
use plot::{plot_svg, PlotConfig};
use std::fs;

fn main() {
    let mut st = Store::new();
    let x = st.sym("x");

    // Example 1: f(x) = sin(cos(x))
    println!("=== Example 1: sin(cos(x)) - Trig composition ===");
    let cosx = st.func("cos", vec![x]);
    let sin_cosx = st.func("sin", vec![cosx]);
    let cfg = PlotConfig::new("x", -6.28, 6.28, 200, 600, 300);
    let svg = plot_svg(&st, sin_cosx, &cfg);
    fs::write("sin_cos_x.svg", &svg).expect("Failed to write sin_cos_x.svg");
    println!("Expression: {}", st.to_string(sin_cosx));
    println!("Saved to: sin_cos_x.svg\n");

    // Example 2: f(x) = exp(cos(x))
    println!("=== Example 2: exp(cos(x)) ===");
    let exp_cosx = st.func("exp", vec![cosx]);
    let cfg2 = PlotConfig::new("x", -6.28, 6.28, 200, 600, 300);
    let svg2 = plot_svg(&st, exp_cosx, &cfg2);
    fs::write("exp_cos_x.svg", &svg2).expect("Failed to write exp_cos_x.svg");
    println!("Expression: {}", st.to_string(exp_cosx));
    println!("Saved to: exp_cos_x.svg\n");

    // Example 3: f(x) = ln(x^2 + 1)
    println!("=== Example 3: ln(x^2 + 1) ===");
    let two = st.int(2);
    let x2 = st.pow(x, two);
    let one = st.int(1);
    let x2_plus_1 = st.add(vec![x2, one]);
    let ln_x2_plus_1 = st.func("ln", vec![x2_plus_1]);
    let cfg3 = PlotConfig::new("x", -5.0, 5.0, 150, 400, 300);
    let svg3 = plot_svg(&st, ln_x2_plus_1, &cfg3);
    fs::write("ln_x2_plus_1.svg", &svg3).expect("Failed to write ln_x2_plus_1.svg");
    println!("Expression: {}", st.to_string(ln_x2_plus_1));
    println!("Saved to: ln_x2_plus_1.svg\n");

    // Example 4: f(x) = x * sin(1/x) for x != 0
    println!("=== Example 4: x * sin(1/x) - Oscillating function ===");
    let neg_one = st.int(-1);
    let inv_x = st.pow(x, neg_one);
    let sin_inv_x = st.func("sin", vec![inv_x]);
    let x_sin_inv_x = st.mul(vec![x, sin_inv_x]);
    let cfg4 = PlotConfig::new("x", -2.0, 2.0, 400, 600, 300);
    let svg4 = plot_svg(&st, x_sin_inv_x, &cfg4);
    fs::write("x_sin_inv_x.svg", &svg4).expect("Failed to write x_sin_inv_x.svg");
    println!("Expression: {}", st.to_string(x_sin_inv_x));
    println!("Saved to: x_sin_inv_x.svg");
    println!("Note: Rapid oscillations near x=0\n");

    // Example 5: f(x) = cos(x) * exp(-x^2)
    println!("=== Example 5: cos(x) * exp(-x^2) - Damped oscillation ===");
    let neg_one = st.int(-1);
    let neg_x2 = st.mul(vec![neg_one, x2]);
    let exp_neg_x2 = st.func("exp", vec![neg_x2]);
    let damped_cos = st.mul(vec![cosx, exp_neg_x2]);
    let cfg5 = PlotConfig::new("x", -5.0, 5.0, 200, 600, 300);
    let svg5 = plot_svg(&st, damped_cos, &cfg5);
    fs::write("damped_cosine.svg", &svg5).expect("Failed to write damped_cosine.svg");
    println!("Expression: {}", st.to_string(damped_cos));
    println!("Saved to: damped_cosine.svg\n");

    // Example 6: f(x) = ln(sin(x) + 2)
    println!("=== Example 6: ln(sin(x) + 2) ===");
    let sinx = st.func("sin", vec![x]);
    let two_const = st.int(2);
    let sinx_plus_2 = st.add(vec![sinx, two_const]);
    let ln_sinx_plus_2 = st.func("ln", vec![sinx_plus_2]);
    let cfg6 = PlotConfig::new("x", -6.28, 6.28, 200, 600, 300);
    let svg6 = plot_svg(&st, ln_sinx_plus_2, &cfg6);
    fs::write("ln_sin_x_plus_2.svg", &svg6).expect("Failed to write ln_sin_x_plus_2.svg");
    println!("Expression: {}", st.to_string(ln_sinx_plus_2));
    println!("Saved to: ln_sin_x_plus_2.svg\n");

    // Example 7: f(x) = sin(exp(x/2))
    println!("=== Example 7: sin(exp(x/2)) ===");
    let half = st.rat(1, 2);
    let half_x = st.mul(vec![half, x]);
    let exp_half_x = st.func("exp", vec![half_x]);
    let sin_exp_half_x = st.func("sin", vec![exp_half_x]);
    let cfg7 = PlotConfig::new("x", -3.0, 3.0, 200, 600, 300);
    let svg7 = plot_svg(&st, sin_exp_half_x, &cfg7);
    fs::write("sin_exp_half_x.svg", &svg7).expect("Failed to write sin_exp_half_x.svg");
    println!("Expression: {}", st.to_string(sin_exp_half_x));
    println!("Saved to: sin_exp_half_x.svg\n");

    println!("All composite function plots generated successfully!");
}
