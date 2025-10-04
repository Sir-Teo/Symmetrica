//! Plot configuration example
//! Demonstrates different PlotConfig settings: samples, ranges, dimensions

use expr_core::Store;
use plot::{plot_svg, PlotConfig};
use std::fs;

fn main() {
    let mut st = Store::new();
    let x = st.sym("x");

    // Create a single test function: sin(x)
    let sinx = st.func("sin", vec![x]);
    println!("Test function: {}\n", st.to_string(sinx));

    // Example 1: Low sampling (visible as coarse polyline)
    println!("=== Example 1: Low sampling (20 samples) ===");
    let cfg1 = PlotConfig::new("x", -6.28, 6.28, 20, 400, 300);
    let svg1 = plot_svg(&st, sinx, &cfg1);
    fs::write("config_low_samples.svg", &svg1).expect("Failed to write config_low_samples.svg");
    println!("Samples: 20, Range: [-6.28, 6.28], Size: 400x300");
    println!("Saved to: config_low_samples.svg\n");

    // Example 2: Medium sampling
    println!("=== Example 2: Medium sampling (100 samples) ===");
    let cfg2 = PlotConfig::new("x", -6.28, 6.28, 100, 400, 300);
    let svg2 = plot_svg(&st, sinx, &cfg2);
    fs::write("config_medium_samples.svg", &svg2)
        .expect("Failed to write config_medium_samples.svg");
    println!("Samples: 100, Range: [-6.28, 6.28], Size: 400x300");
    println!("Saved to: config_medium_samples.svg\n");

    // Example 3: High sampling (smooth curve)
    println!("=== Example 3: High sampling (500 samples) ===");
    let cfg3 = PlotConfig::new("x", -6.28, 6.28, 500, 400, 300);
    let svg3 = plot_svg(&st, sinx, &cfg3);
    fs::write("config_high_samples.svg", &svg3).expect("Failed to write config_high_samples.svg");
    println!("Samples: 500, Range: [-6.28, 6.28], Size: 400x300");
    println!("Saved to: config_high_samples.svg\n");

    // Example 4: Small plot dimensions
    println!("=== Example 4: Small dimensions (200x150) ===");
    let cfg4 = PlotConfig::new("x", -6.28, 6.28, 100, 200, 150);
    let svg4 = plot_svg(&st, sinx, &cfg4);
    fs::write("config_small_size.svg", &svg4).expect("Failed to write config_small_size.svg");
    println!("Samples: 100, Range: [-6.28, 6.28], Size: 200x150");
    println!("Saved to: config_small_size.svg\n");

    // Example 5: Large plot dimensions
    println!("=== Example 5: Large dimensions (800x600) ===");
    let cfg5 = PlotConfig::new("x", -6.28, 6.28, 200, 800, 600);
    let svg5 = plot_svg(&st, sinx, &cfg5);
    fs::write("config_large_size.svg", &svg5).expect("Failed to write config_large_size.svg");
    println!("Samples: 200, Range: [-6.28, 6.28], Size: 800x600");
    println!("Saved to: config_large_size.svg\n");

    // Example 6: Narrow x range (zoomed in)
    println!("=== Example 6: Narrow range [-1, 1] ===");
    let cfg6 = PlotConfig::new("x", -1.0, 1.0, 100, 400, 300);
    let svg6 = plot_svg(&st, sinx, &cfg6);
    fs::write("config_narrow_range.svg", &svg6).expect("Failed to write config_narrow_range.svg");
    println!("Samples: 100, Range: [-1.0, 1.0], Size: 400x300");
    println!("Saved to: config_narrow_range.svg\n");

    // Example 7: Wide x range
    println!("=== Example 7: Wide range [-20, 20] ===");
    let cfg7 = PlotConfig::new("x", -20.0, 20.0, 300, 800, 300);
    let svg7 = plot_svg(&st, sinx, &cfg7);
    fs::write("config_wide_range.svg", &svg7).expect("Failed to write config_wide_range.svg");
    println!("Samples: 300, Range: [-20.0, 20.0], Size: 800x300");
    println!("Saved to: config_wide_range.svg\n");

    // Example 8: Asymmetric range (only positive)
    println!("=== Example 8: Asymmetric range [0, 10] ===");
    let cfg8 = PlotConfig::new("x", 0.0, 10.0, 150, 400, 300);
    let svg8 = plot_svg(&st, sinx, &cfg8);
    fs::write("config_asymmetric_range.svg", &svg8)
        .expect("Failed to write config_asymmetric_range.svg");
    println!("Samples: 150, Range: [0.0, 10.0], Size: 400x300");
    println!("Saved to: config_asymmetric_range.svg\n");

    // Example 9: Square aspect ratio
    println!("=== Example 9: Square aspect ratio (400x400) ===");
    let cfg9 = PlotConfig::new("x", -6.28, 6.28, 150, 400, 400);
    let svg9 = plot_svg(&st, sinx, &cfg9);
    fs::write("config_square_aspect.svg", &svg9).expect("Failed to write config_square_aspect.svg");
    println!("Samples: 150, Range: [-6.28, 6.28], Size: 400x400");
    println!("Saved to: config_square_aspect.svg\n");

    // Example 10: Wide aspect ratio (panoramic)
    println!("=== Example 10: Wide aspect ratio (800x200) ===");
    let cfg10 = PlotConfig::new("x", -6.28, 6.28, 200, 800, 200);
    let svg10 = plot_svg(&st, sinx, &cfg10);
    fs::write("config_wide_aspect.svg", &svg10).expect("Failed to write config_wide_aspect.svg");
    println!("Samples: 200, Range: [-6.28, 6.28], Size: 800x200");
    println!("Saved to: config_wide_aspect.svg\n");

    println!("All configuration examples generated successfully!");
    println!("\nKey takeaways:");
    println!("- More samples = smoother curves (but larger SVG files)");
    println!("- Adjust range to zoom in/out on interesting features");
    println!("- Choose dimensions based on output medium (web, print, etc.)");
    println!("- Consider aspect ratio for proper visualization");
}
