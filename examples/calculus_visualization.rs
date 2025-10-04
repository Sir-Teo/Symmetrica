//! Calculus visualization example
//! Demonstrates plotting functions alongside their derivatives using calculus crate

use calculus::diff;
use expr_core::Store;
use plot::{plot_svg, PlotConfig};
use simplify::simplify;
use std::fs;

fn main() {
    let mut st = Store::new();
    let x = st.sym("x");

    // Example 1: f(x) = x^2 and f'(x) = 2x
    println!("=== Example 1: Parabola and its derivative ===");
    let two = st.int(2);
    let x2 = st.pow(x, two);
    let dx2 = diff(&mut st, x2, "x");
    let dx2_simp = simplify(&mut st, dx2);
    println!("f(x)  = {}", st.to_string(x2));
    println!("f'(x) = {}", st.to_string(dx2_simp));

    let cfg = PlotConfig::new("x", -3.0, 3.0, 100, 400, 300);
    let svg_f = plot_svg(&st, x2, &cfg);
    let svg_df = plot_svg(&st, dx2_simp, &cfg);
    fs::write("calc_parabola.svg", &svg_f).expect("Failed to write calc_parabola.svg");
    fs::write("calc_parabola_deriv.svg", &svg_df).expect("Failed to write calc_parabola_deriv.svg");
    println!("Saved: calc_parabola.svg, calc_parabola_deriv.svg\n");

    // Example 2: f(x) = sin(x) and f'(x) = cos(x)
    println!("=== Example 2: Sine and its derivative (cosine) ===");
    let sinx = st.func("sin", vec![x]);
    let dsinx = diff(&mut st, sinx, "x");
    let dsinx_simp = simplify(&mut st, dsinx);
    println!("f(x)  = {}", st.to_string(sinx));
    println!("f'(x) = {}", st.to_string(dsinx_simp));

    let cfg2 = PlotConfig::new("x", -6.28, 6.28, 200, 600, 300);
    let svg_sin = plot_svg(&st, sinx, &cfg2);
    let svg_cos = plot_svg(&st, dsinx_simp, &cfg2);
    fs::write("calc_sin.svg", &svg_sin).expect("Failed to write calc_sin.svg");
    fs::write("calc_sin_deriv.svg", &svg_cos).expect("Failed to write calc_sin_deriv.svg");
    println!("Saved: calc_sin.svg, calc_sin_deriv.svg\n");

    // Example 3: f(x) = exp(x) and f'(x) = exp(x)
    println!("=== Example 3: Exponential and its derivative (itself) ===");
    let expx = st.func("exp", vec![x]);
    let dexpx = diff(&mut st, expx, "x");
    let dexpx_simp = simplify(&mut st, dexpx);
    println!("f(x)  = {}", st.to_string(expx));
    println!("f'(x) = {}", st.to_string(dexpx_simp));

    let cfg3 = PlotConfig::new("x", -2.0, 2.0, 100, 400, 300);
    let svg_exp = plot_svg(&st, expx, &cfg3);
    let svg_dexp = plot_svg(&st, dexpx_simp, &cfg3);
    fs::write("calc_exp.svg", &svg_exp).expect("Failed to write calc_exp.svg");
    fs::write("calc_exp_deriv.svg", &svg_dexp).expect("Failed to write calc_exp_deriv.svg");
    println!("Saved: calc_exp.svg, calc_exp_deriv.svg");
    println!("Note: Both plots should look identical!\n");

    // Example 4: f(x) = x^3 - 3x and f'(x) = 3x^2 - 3
    println!("=== Example 4: Cubic with critical points ===");
    let three_exp = st.int(3);
    let x3 = st.pow(x, three_exp);
    let neg_three = st.int(-3);
    let neg_three_x = st.mul(vec![neg_three, x]);
    let cubic = st.add(vec![x3, neg_three_x]);
    let dcubic = diff(&mut st, cubic, "x");
    let dcubic_simp = simplify(&mut st, dcubic);
    println!("f(x)  = {}", st.to_string(cubic));
    println!("f'(x) = {}", st.to_string(dcubic_simp));

    let cfg4 = PlotConfig::new("x", -3.0, 3.0, 150, 400, 300);
    let svg_cubic = plot_svg(&st, cubic, &cfg4);
    let svg_dcubic = plot_svg(&st, dcubic_simp, &cfg4);
    fs::write("calc_cubic.svg", &svg_cubic).expect("Failed to write calc_cubic.svg");
    fs::write("calc_cubic_deriv.svg", &svg_dcubic).expect("Failed to write calc_cubic_deriv.svg");
    println!("Saved: calc_cubic.svg, calc_cubic_deriv.svg");
    println!("Note: Derivative crosses zero at critical points of f\n");

    // Example 5: f(x) = ln(x) and f'(x) = 1/x
    println!("=== Example 5: Logarithm and its derivative ===");
    let lnx = st.func("ln", vec![x]);
    let dlnx = diff(&mut st, lnx, "x");
    let dlnx_simp = simplify(&mut st, dlnx);
    println!("f(x)  = {}", st.to_string(lnx));
    println!("f'(x) = {}", st.to_string(dlnx_simp));

    let cfg5 = PlotConfig::new("x", 0.1, 5.0, 100, 400, 300);
    let svg_ln = plot_svg(&st, lnx, &cfg5);
    let svg_dln = plot_svg(&st, dlnx_simp, &cfg5);
    fs::write("calc_ln.svg", &svg_ln).expect("Failed to write calc_ln.svg");
    fs::write("calc_ln_deriv.svg", &svg_dln).expect("Failed to write calc_ln_deriv.svg");
    println!("Saved: calc_ln.svg, calc_ln_deriv.svg\n");

    // Example 6: Second derivative - f(x) = x^4, f'(x) = 4x^3, f''(x) = 12x^2
    println!("=== Example 6: Higher derivatives ===");
    let four = st.int(4);
    let x4 = st.pow(x, four);
    let dx4 = diff(&mut st, x4, "x");
    let dx4_simp = simplify(&mut st, dx4);
    let d2x4 = diff(&mut st, dx4_simp, "x");
    let d2x4_simp = simplify(&mut st, d2x4);
    println!("f(x)   = {}", st.to_string(x4));
    println!("f'(x)  = {}", st.to_string(dx4_simp));
    println!("f''(x) = {}", st.to_string(d2x4_simp));

    let cfg6 = PlotConfig::new("x", -2.0, 2.0, 150, 400, 300);
    let svg_f6 = plot_svg(&st, x4, &cfg6);
    let svg_df6 = plot_svg(&st, dx4_simp, &cfg6);
    let svg_d2f6 = plot_svg(&st, d2x4_simp, &cfg6);
    fs::write("calc_x4.svg", &svg_f6).expect("Failed to write calc_x4.svg");
    fs::write("calc_x4_first_deriv.svg", &svg_df6)
        .expect("Failed to write calc_x4_first_deriv.svg");
    fs::write("calc_x4_second_deriv.svg", &svg_d2f6)
        .expect("Failed to write calc_x4_second_deriv.svg");
    println!("Saved: calc_x4.svg, calc_x4_first_deriv.svg, calc_x4_second_deriv.svg\n");

    println!("All calculus visualization examples generated successfully!");
    println!("\nObservations:");
    println!("- Where f'(x) = 0, f(x) has critical points (maxima/minima)");
    println!("- Where f'(x) > 0, f(x) is increasing");
    println!("- Where f'(x) < 0, f(x) is decreasing");
    println!("- Where f''(x) > 0, f(x) is concave up");
    println!("- Where f''(x) < 0, f(x) is concave down");
}
