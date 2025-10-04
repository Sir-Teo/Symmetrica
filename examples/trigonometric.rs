//! Trigonometric function plotting example
//! Demonstrates plotting sin, cos, and composite trig functions

use expr_core::Store;
use plot::{plot_svg, PlotConfig};
use std::fs;

fn main() {
    let mut st = Store::new();
    let x = st.sym("x");

    // Example 1: f(x) = sin(x)
    println!("=== Example 1: sin(x) ===");
    let sinx = st.func("sin", vec![x]);
    let cfg = PlotConfig::new("x", -6.28, 6.28, 200, 600, 300); // -2π to 2π
    let svg = plot_svg(&st, sinx, &cfg);
    fs::write("sin_x.svg", &svg).expect("Failed to write sin_x.svg");
    println!("Expression: {}", st.to_string(sinx));
    println!("Saved to: sin_x.svg\n");

    // Example 2: f(x) = cos(x)
    println!("=== Example 2: cos(x) ===");
    let cosx = st.func("cos", vec![x]);
    let cfg2 = PlotConfig::new("x", -6.28, 6.28, 200, 600, 300);
    let svg2 = plot_svg(&st, cosx, &cfg2);
    fs::write("cos_x.svg", &svg2).expect("Failed to write cos_x.svg");
    println!("Expression: {}", st.to_string(cosx));
    println!("Saved to: cos_x.svg\n");

    // Example 3: f(x) = sin(2x)
    println!("=== Example 3: sin(2x) - Frequency doubling ===");
    let two = st.int(2);
    let two_x = st.mul(vec![two, x]);
    let sin_2x = st.func("sin", vec![two_x]);
    let cfg3 = PlotConfig::new("x", -6.28, 6.28, 200, 600, 300);
    let svg3 = plot_svg(&st, sin_2x, &cfg3);
    fs::write("sin_2x.svg", &svg3).expect("Failed to write sin_2x.svg");
    println!("Expression: {}", st.to_string(sin_2x));
    println!("Saved to: sin_2x.svg\n");

    // Example 4: f(x) = 2*sin(x)
    println!("=== Example 4: 2*sin(x) - Amplitude scaling ===");
    let two_sinx = st.mul(vec![two, sinx]);
    let cfg4 = PlotConfig::new("x", -6.28, 6.28, 200, 600, 300);
    let svg4 = plot_svg(&st, two_sinx, &cfg4);
    fs::write("2_sin_x.svg", &svg4).expect("Failed to write 2_sin_x.svg");
    println!("Expression: {}", st.to_string(two_sinx));
    println!("Saved to: 2_sin_x.svg\n");

    // Example 5: f(x) = sin(x) + cos(x)
    println!("=== Example 5: sin(x) + cos(x) - Sum of trig functions ===");
    let sin_plus_cos = st.add(vec![sinx, cosx]);
    let cfg5 = PlotConfig::new("x", -6.28, 6.28, 200, 600, 300);
    let svg5 = plot_svg(&st, sin_plus_cos, &cfg5);
    fs::write("sin_plus_cos.svg", &svg5).expect("Failed to write sin_plus_cos.svg");
    println!("Expression: {}", st.to_string(sin_plus_cos));
    println!("Saved to: sin_plus_cos.svg\n");

    // Example 6: f(x) = sin(x) * cos(x)
    println!("=== Example 6: sin(x) * cos(x) - Product of trig functions ===");
    let sin_times_cos = st.mul(vec![sinx, cosx]);
    let cfg6 = PlotConfig::new("x", -6.28, 6.28, 200, 600, 300);
    let svg6 = plot_svg(&st, sin_times_cos, &cfg6);
    fs::write("sin_times_cos.svg", &svg6).expect("Failed to write sin_times_cos.svg");
    println!("Expression: {}", st.to_string(sin_times_cos));
    println!("Saved to: sin_times_cos.svg\n");

    // Example 7: f(x) = sin(x^2)
    println!("=== Example 7: sin(x^2) - Composed with polynomial ===");
    let two_exp = st.int(2);
    let x2 = st.pow(x, two_exp);
    let sin_x2 = st.func("sin", vec![x2]);
    let cfg7 = PlotConfig::new("x", -3.0, 3.0, 200, 600, 300);
    let svg7 = plot_svg(&st, sin_x2, &cfg7);
    fs::write("sin_x_squared.svg", &svg7).expect("Failed to write sin_x_squared.svg");
    println!("Expression: {}", st.to_string(sin_x2));
    println!("Saved to: sin_x_squared.svg\n");

    println!("All trigonometric plots generated successfully!");
}
