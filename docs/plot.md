# plot - Plotting Module

## Overview

The `plot` crate provides simple SVG plotting for expressions by evaluating them numerically over a range and rendering as polylines. It uses deterministic fixed-precision output and supports basic mathematical functions without external dependencies.

## Core Function

```rust
pub fn plot_svg(st: &Store, id: ExprId, cfg: &PlotConfig) -> String
```

Generates an SVG plot of the expression over the specified range.

**Returns:** Complete SVG document as a string.

## Configuration

```rust
pub struct PlotConfig {
    pub var: String,      // Variable name (e.g., "x")
    pub x_min: f64,       // Left boundary
    pub x_max: f64,       // Right boundary
    pub samples: usize,   // Number of points to sample
    pub width: u32,       // SVG width in pixels
    pub height: u32,      // SVG height in pixels
}
```

**Constructor:**
```rust
pub fn new(var: &str, x_min: f64, x_max: f64, samples: usize, width: u32, height: u32) -> Self
```

### Example Configuration

```rust
use plot::PlotConfig;

let cfg = PlotConfig::new(
    "x",      // Variable
    -2.0,     // x_min
    2.0,      // x_max
    100,      // samples
    640,      // width
    480,      // height
);
```

## Evaluation

```rust
pub fn eval_f64(st: &Store, id: ExprId, var: &str, x: f64) -> Option<f64>
```

Evaluates an expression at a numeric point.

**Returns:**
- `Some(y)`: Finite value
- `None`: Evaluation failed (undefined function, domain error, infinity, NaN)

### Supported Operations

**Arithmetic:**
- `Add`: Sum of all terms
- `Mul`: Product of all factors
- `Pow`: Base raised to exponent (`b.powf(e)`)

**Functions:**
- `sin(x)`: Sine
- `cos(x)`: Cosine
- `exp(x)`: Exponential
- `ln(x)`: Natural logarithm (domain: x > 0)

**Constants:**
- `Integer`: Converted to f64
- `Rational`: Computed as `num / den`

### Examples

**Polynomial:**
```rust
let x = st.sym("x");
let expr = st.pow(x, st.int(2));  // x^2

let y = eval_f64(&st, expr, "x", 3.0).unwrap();
assert_eq!(y, 9.0);
```

**Trigonometric:**
```rust
let sin_x = st.func("sin", vec![x]);
let y = eval_f64(&st, sin_x, "x", std::f64::consts::FRAC_PI_2).unwrap();
assert!((y - 1.0).abs() < 1e-12);
```

**Domain error:**
```rust
let ln_x = st.func("ln", vec![x]);
let y = eval_f64(&st, ln_x, "x", -1.0);
assert!(y.is_none());  // ln(-1) is undefined
```

## Plotting Algorithm

1. **Sample**: Evaluate expression at `samples` uniformly spaced points in `[x_min, x_max]`
2. **Filter**: Keep only finite values (discard NaN, infinity, evaluation failures)
3. **Compute range**: Determine y_min and y_max from finite points
4. **Map to screen**: Transform (x, y) → (screen_x, screen_y)
   - x: linear mapping to [MARGIN, width - MARGIN]
   - y: linear mapping to [MARGIN, height - MARGIN], inverted (SVG y-axis points down)
5. **Generate SVG**: Create polyline with fixed-precision coordinates

### Coordinate Mapping

```rust
screen_x = MARGIN + (x - x_min) / (x_max - x_min) * plot_width
screen_y = height - MARGIN - (y - y_min) / (y_max - y_min) * plot_height
```

### Deterministic Output

Coordinates are formatted with fixed precision (6 decimal places):
```rust
format!("{:.6},{:.6}", x, y)
```

This ensures identical SVG output across runs, enabling reproducible testing.

## SVG Structure

Generated SVG contains:
- **Border**: Gray rectangle for context
- **Polyline**: Blue stroke connecting data points
- **Fixed size**: Specified width × height

**Example SVG:**
```xml
<svg xmlns="http://www.w3.org/2000/svg" width="640" height="480">
<rect x="0" y="0" width="640" height="480" fill="none" stroke="#ccc" stroke-width="1" />
<polyline fill="none" stroke="#1f77b4" stroke-width="1.5" points="10.0,470.0 20.5,450.3 ..." />
</svg>
```

## Examples

### Parabola

```rust
use expr_core::Store;
use plot::{plot_svg, PlotConfig};

let mut st = Store::new();
let x = st.sym("x");
let x2 = st.pow(x, st.int(2));

let cfg = PlotConfig::new("x", -2.0, 2.0, 50, 400, 300);
let svg = plot_svg(&st, x2, &cfg);

// Save to file or display in browser
std::fs::write("parabola.svg", svg).unwrap();
```

### Sine Wave

```rust
let sin_x = st.func("sin", vec![x]);
let cfg = PlotConfig::new("x", 0.0, 6.28, 100, 640, 480);
let svg = plot_svg(&st, sin_x, &cfg);
```

### Rational Function

```rust
// Plot 1/x
let inv_x = st.pow(x, st.int(-1));
let cfg = PlotConfig::new("x", -5.0, 5.0, 200, 640, 480);
let svg = plot_svg(&st, inv_x, &cfg);
// Note: Points near x=0 will be filtered (infinity/NaN)
```

### Composite Function

```rust
// Plot sin(x^2)
let x2 = st.pow(x, st.int(2));
let sin_x2 = st.func("sin", vec![x2]);
let cfg = PlotConfig::new("x", -3.0, 3.0, 150, 640, 480);
let svg = plot_svg(&st, sin_x2, &cfg);
```

## Edge Cases

### No Finite Points

If all evaluations fail or produce non-finite values:
```rust
let y = st.sym("y");  // Wrong variable
let cfg = PlotConfig::new("x", -1.0, 1.0, 10, 640, 480);
let svg = plot_svg(&st, y, &cfg);
// Result: Empty SVG (no polyline)
```

### Zero Y-Range

If all y-values are identical:
```rust
let five = st.int(5);  // Constant function
let cfg = PlotConfig::new("x", 0.0, 10.0, 10, 640, 480);
let svg = plot_svg(&st, five, &cfg);
// Automatically expands range to [y-1, y+1]
```

### Single Sample

Minimum 2 samples for a line:
```rust
let cfg = PlotConfig::new("x", 0.0, 1.0, 1, 640, 480);
// Automatically adjusted to samples.max(2)
```

## Performance

- **Evaluation**: O(n * e) where n = samples, e = expression complexity
- **Rendering**: O(n) to build SVG string
- **Memory**: O(n) for storing sample points

Typical usage:
- 100 samples: < 1ms
- 1000 samples: < 10ms

## Limitations

### Numerical Evaluation Only

Cannot plot symbolic expressions directly. All values must be computable as f64.

### Limited Function Support

Only sin, cos, exp, ln are recognized. Unknown functions return `None`:
```rust
let tan_x = st.func("tan", vec![x]);
let y = eval_f64(&st, tan_x, "x", 1.0);
assert!(y.is_none());  // tan not supported
```

### No Axes or Labels

Generated SVG is minimal:
- No axis lines
- No tick marks
- No labels or titles

For presentation-quality plots, post-process or use dedicated plotting library.

### Discontinuities

Functions with discontinuities (e.g., 1/x) are plotted as continuous lines:
- No automatic gap detection
- Visual artifacts near discontinuities

### Single Variable Only

Cannot plot multivariate functions or parametric curves.

### No Styling Options

Hardcoded colors and stroke widths:
- Polyline: Blue (#1f77b4), 1.5px
- Border: Gray (#ccc), 1px

## Integration with CLI

```bash
matika_cli plot \
  --sexpr "(^ (Sym x) (Int 2))" \
  --var x \
  --xmin -2 \
  --xmax 2 \
  --samples 100 \
  --width 640 \
  --height 480 > plot.svg
```

Opens `plot.svg` in browser for visualization.

## Testing

Comprehensive tests:
- Deterministic output (repeated calls yield identical SVG)
- Basic functions (sin, cos, exp, ln)
- Domain errors (ln of negative, evaluation of unbound variables)
- Edge cases (empty plots, constant functions, single sample)
- Coordinate precision

Run tests:
```bash
cargo test -p plot
```

## Example: Multi-Function Plot

```rust
use expr_core::Store;
use plot::{plot_svg, PlotConfig};

let mut st = Store::new();
let x = st.sym("x");

// Plot sin(x), cos(x), exp(-x)
let functions = vec![
    ("sin_x", st.func("sin", vec![x])),
    ("cos_x", st.func("cos", vec![x])),
    ("exp_neg_x", st.func("exp", vec![st.mul(vec![st.int(-1), x])])),
];

for (name, expr) in functions {
    let cfg = PlotConfig::new("x", 0.0, 6.28, 100, 640, 480);
    let svg = plot_svg(&st, expr, &cfg);
    std::fs::write(format!("{}.svg", name), svg).unwrap();
}
```

## Future Enhancements

- **Axis rendering**: Draw x and y axes with tick marks
- **Labels**: Add titles, axis labels, legends
- **Multiple curves**: Plot several functions on same canvas
- **Styling**: Configurable colors, line widths, point markers
- **Adaptive sampling**: Increase sampling near rapid changes
- **Discontinuity detection**: Break polyline at jumps
- **More functions**: tan, atan, sqrt, abs, etc.
- **Log scales**: Logarithmic x or y axis
- **Export formats**: PNG, PDF via rasterization
- **Interactive**: Pan, zoom, hover tooltips (requires JavaScript)

## References

- Depends on: `expr_core`
- Used by: `cli`
- SVG spec: https://www.w3.org/TR/SVG/
- Related: Matplotlib (Python), Plots.jl (Julia), Plotly
