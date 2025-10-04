//! Edge cases and special behaviors example
//! Demonstrates how the plotter handles discontinuities, undefined regions, etc.

use expr_core::Store;
use plot::{plot_svg, PlotConfig};
use std::fs;

fn main() {
    let mut st = Store::new();
    let x = st.sym("x");

    println!("=== Edge Cases and Special Behaviors Demo ===\n");

    // Example 1: Discontinuity at x=0 (1/x)
    println!("=== Example 1: Discontinuity at x=0 ===");
    let neg_one = st.int(-1);
    let inv_x = st.pow(x, neg_one);
    println!("Expression: {}", st.to_string(inv_x));
    println!("Behavior: eval_f64 returns None at x=0, creating a gap in the plot");

    let cfg1 = PlotConfig::new("x", -5.0, 5.0, 200, 400, 300);
    let svg1 = plot_svg(&st, inv_x, &cfg1);
    fs::write("edge_discontinuity.svg", &svg1).expect("Failed to write edge_discontinuity.svg");
    println!("Saved: edge_discontinuity.svg\n");

    // Example 2: Domain restriction (ln(x) for x <= 0)
    println!("=== Example 2: Domain restriction - ln(x) ===");
    let lnx = st.func("ln", vec![x]);
    println!("Expression: {}", st.to_string(lnx));
    println!("Behavior: ln(x) undefined for x <= 0; those points are omitted");

    let cfg2 = PlotConfig::new("x", -2.0, 5.0, 150, 400, 300);
    let svg2 = plot_svg(&st, lnx, &cfg2);
    fs::write("edge_domain_restriction.svg", &svg2)
        .expect("Failed to write edge_domain_restriction.svg");
    println!("Saved: edge_domain_restriction.svg\n");

    // Example 3: Vertical asymptote (tan-like via sin/cos)
    println!("=== Example 3: Vertical asymptotes ===");
    let sinx = st.func("sin", vec![x]);
    let cosx = st.func("cos", vec![x]);
    let inv_cosx = st.pow(cosx, neg_one);
    let tan_like = st.mul(vec![sinx, inv_cosx]);
    println!("Expression: sin(x) / cos(x) (like tan(x))");
    println!("Behavior: Undefined when cos(x)=0 (at π/2, 3π/2, etc.)");

    let cfg3 = PlotConfig::new("x", -6.28, 6.28, 300, 600, 300);
    let svg3 = plot_svg(&st, tan_like, &cfg3);
    fs::write("edge_vertical_asymptote.svg", &svg3)
        .expect("Failed to write edge_vertical_asymptote.svg");
    println!("Saved: edge_vertical_asymptote.svg\n");

    // Example 4: Rapid oscillation near singularity
    println!("=== Example 4: Rapid oscillation - sin(1/x) ===");
    let sin_inv_x = st.func("sin", vec![inv_x]);
    println!("Expression: {}", st.to_string(sin_inv_x));
    println!("Behavior: Oscillates infinitely fast as x→0; sampling may miss details");

    let cfg4 = PlotConfig::new("x", -1.0, 1.0, 500, 600, 300);
    let svg4 = plot_svg(&st, sin_inv_x, &cfg4);
    fs::write("edge_rapid_oscillation.svg", &svg4)
        .expect("Failed to write edge_rapid_oscillation.svg");
    println!("Saved: edge_rapid_oscillation.svg");
    println!("Note: Even with 500 samples, aliasing occurs near x=0\n");

    // Example 5: Very large values (exp(x) over wide range)
    println!("=== Example 5: Large dynamic range - exp(x) ===");
    let expx = st.func("exp", vec![x]);
    println!("Expression: {}", st.to_string(expx));
    println!("Behavior: exp(x) grows exponentially; plot auto-scales y-axis");

    let cfg5 = PlotConfig::new("x", -5.0, 5.0, 150, 400, 300);
    let svg5 = plot_svg(&st, expx, &cfg5);
    fs::write("edge_large_values.svg", &svg5).expect("Failed to write edge_large_values.svg");
    println!("Saved: edge_large_values.svg");
    println!("Note: Exponential growth makes small values near x=-5 hard to see\n");

    // Example 6: Constant function (no y variation)
    println!("=== Example 6: Constant function ===");
    let five = st.int(5);
    println!("Expression: {}", st.to_string(five));
    println!("Behavior: y-range has zero height; adjusted to avoid degenerate case");

    let cfg6 = PlotConfig::new("x", -5.0, 5.0, 100, 400, 300);
    let svg6 = plot_svg(&st, five, &cfg6);
    fs::write("edge_constant.svg", &svg6).expect("Failed to write edge_constant.svg");
    println!("Saved: edge_constant.svg");
    println!("Note: Plotter adds ±1 to y-range when y_max - y_min < 1e-12\n");

    // Example 7: Expression with no valid points (wrong variable)
    println!("=== Example 7: No valid points (variable mismatch) ===");
    let y = st.sym("y"); // Using 'y' but plotting against 'x'
    println!("Expression: y (but plotting variable is 'x')");
    println!("Behavior: All evaluations return None; empty SVG with no polyline");

    let cfg7 = PlotConfig::new("x", -5.0, 5.0, 100, 400, 300);
    let svg7 = plot_svg(&st, y, &cfg7);
    fs::write("edge_no_valid_points.svg", &svg7).expect("Failed to write edge_no_valid_points.svg");
    println!("Saved: edge_no_valid_points.svg");
    println!("Note: Empty SVG with just the border rectangle\n");

    // Example 8: Very steep slope
    println!("=== Example 8: Very steep slope - x^10 ===");
    let ten = st.int(10);
    let x10 = st.pow(x, ten);
    println!("Expression: {}", st.to_string(x10));
    println!("Behavior: Extremely steep near edges; flat near center");

    let cfg8 = PlotConfig::new("x", -1.5, 1.5, 200, 400, 300);
    let svg8 = plot_svg(&st, x10, &cfg8);
    fs::write("edge_steep_slope.svg", &svg8).expect("Failed to write edge_steep_slope.svg");
    println!("Saved: edge_steep_slope.svg\n");

    // Example 9: Negative exponent with even denominator
    println!("=== Example 9: x^(-1/2) - defined only for x > 0 ===");
    let neg_half = st.rat(-1, 2);
    let x_neg_half = st.pow(x, neg_half);
    println!("Expression: {}", st.to_string(x_neg_half));
    println!("Behavior: x^(-1/2) = 1/sqrt(x), only defined for x > 0 in real numbers");

    let cfg9 = PlotConfig::new("x", -2.0, 5.0, 150, 400, 300);
    let svg9 = plot_svg(&st, x_neg_half, &cfg9);
    fs::write("edge_negative_power_sqrt.svg", &svg9)
        .expect("Failed to write edge_negative_power_sqrt.svg");
    println!("Saved: edge_negative_power_sqrt.svg");
    println!("Note: powf returns NaN for negative base with fractional exponent\n");

    // Example 10: Single sample point
    println!("=== Example 10: Minimum samples (samples=1) ===");
    let two = st.int(2);
    let x2 = st.pow(x, two);
    println!("Expression: {}", st.to_string(x2));
    println!("Behavior: Single point plot; internally bumped to samples=2 minimum");

    let cfg10 = PlotConfig::new("x", -2.0, 2.0, 1, 400, 300);
    let svg10 = plot_svg(&st, x2, &cfg10);
    fs::write("edge_single_sample.svg", &svg10).expect("Failed to write edge_single_sample.svg");
    println!("Saved: edge_single_sample.svg\n");

    println!("=== Summary ===");
    println!("The plotter handles various edge cases gracefully:");
    println!("• Discontinuities: Non-finite values are skipped, creating gaps");
    println!("• Domain restrictions: Invalid evaluations omitted from polyline");
    println!("• Auto-scaling: Y-axis range computed from finite values");
    println!("• Constant functions: Range adjusted to avoid zero-height");
    println!("• Empty results: Valid SVG with no polyline when no points evaluable");
    println!("• Deterministic output: Fixed precision (6 digits) for stable SVG files");
}
