# Symmetrica Examples

This directory contains comprehensive examples demonstrating the capabilities of Symmetrica's core modules.

## Quick Start

Run any example with:
```bash
cargo run --example <example_name>
```

For instance:
```bash
cargo run --example basic_polynomial
```

Each example generates one or more SVG files in the current directory that can be viewed in any web browser.

## Examples Overview

### 1. `basic_polynomial.rs`
**Focus**: Polynomial functions

Demonstrates plotting of basic polynomial expressions:
- Parabola (x²)
- Cubic (x³)
- Quadratic with linear term (x² + 2x + 1)
- Higher degree polynomial (x⁴ - 2x²)

**Outputs**: `parabola.svg`, `cubic.svg`, `quadratic.svg`, `quartic.svg`

**Key concepts**: Basic expression building, power functions

---

### 2. `trigonometric.rs`
**Focus**: Trigonometric functions

Plots various trigonometric expressions:
- Basic: sin(x), cos(x)
- Frequency modulation: sin(2x)
- Amplitude scaling: 2·sin(x)
- Combinations: sin(x) + cos(x), sin(x)·cos(x)
- Composition: sin(x²)

**Outputs**: `sin_x.svg`, `cos_x.svg`, `sin_2x.svg`, `2_sin_x.svg`, `sin_plus_cos.svg`, `sin_times_cos.svg`, `sin_x_squared.svg`

**Key concepts**: Function nodes, trigonometric evaluation

---

### 3. `exponential_logarithm.rs`
**Focus**: Exponential and logarithmic functions

Demonstrates exp and ln functions:
- Basic: exp(x), ln(x)
- Exponential decay: exp(-x)
- Compositions: exp(x²), ln(x²), exp(sin(x))
- Products: x·exp(-x)

**Outputs**: `exp_x.svg`, `ln_x.svg`, `exp_neg_x.svg`, `exp_x_squared.svg`, `ln_x_squared.svg`, `x_exp_neg_x.svg`, `exp_sin_x.svg`

**Key concepts**: Exponential growth/decay, logarithmic domain restrictions

---

### 4. `rational_functions.rs`
**Focus**: Rational functions (ratios of polynomials)

Plots various rational expressions:
- Simple reciprocals: 1/x, 1/x²
- Proper rationals: x/(x²+1), (x²-1)/(x²+1)
- Classic curves: Witch of Agnesi 1/(1+x²)
- Higher degree: x³/(x²+1)

**Outputs**: `reciprocal.svg`, `reciprocal_squared.svg`, `x_over_x2_plus_1.svg`, etc.

**Key concepts**: Negative exponents, discontinuities, asymptotic behavior

---

### 5. `power_functions.rs`
**Focus**: Various power functions including fractional exponents

Demonstrates different exponent types:
- Fractional exponents: x^(1/2), x^(1/3), x^(3/2), x^(2/3)
- Negative fractional: x^(-1/2)
- Compositions: (x²)^(1/2)
- Higher odd powers: x⁵

**Outputs**: `sqrt_x.svg`, `cbrt_x.svg`, `x_three_halves.svg`, `inv_sqrt_x.svg`, etc.

**Key concepts**: Rational exponents, domain considerations for even roots

---

### 6. `composite_functions.rs`
**Focus**: Complex function compositions

Shows nested and combined functions:
- Trig compositions: sin(cos(x))
- Exp + trig: exp(cos(x)), sin(exp(x/2))
- Log compositions: ln(x²+1), ln(sin(x)+2)
- Oscillating: x·sin(1/x)
- Damped: cos(x)·exp(-x²)

**Outputs**: `sin_cos_x.svg`, `exp_cos_x.svg`, `ln_x2_plus_1.svg`, `x_sin_inv_x.svg`, `damped_cosine.svg`, etc.

**Key concepts**: Function composition, nested evaluation

---

### 7. `plot_configuration.rs`
**Focus**: PlotConfig parameters and their effects

Demonstrates different configuration settings:
- Sampling rates: low (20), medium (100), high (500)
- Plot dimensions: small (200×150), standard (400×300), large (800×600)
- X-axis ranges: narrow [-1,1], standard [-6.28,6.28], wide [-20,20]
- Aspect ratios: square (400×400), wide (800×200)
- Range types: symmetric, asymmetric

**Outputs**: `config_low_samples.svg`, `config_medium_samples.svg`, `config_high_samples.svg`, etc.

**Key concepts**: PlotConfig usage, sampling effects, dimension choices

---

### 8. `calculus_visualization.rs`
**Focus**: Functions with their derivatives (requires `calculus` crate)

Plots functions alongside their derivatives:
- Parabola: x² and 2x
- Trig: sin(x) and cos(x)
- Exponential: exp(x) and exp(x)
- Cubic: x³-3x and 3x²-3
- Logarithm: ln(x) and 1/x
- Second derivatives: x⁴, 4x³, 12x²

**Outputs**: `calc_parabola.svg`, `calc_parabola_deriv.svg`, `calc_sin.svg`, `calc_sin_deriv.svg`, etc.

**Key concepts**: Integration with calculus module, derivative visualization, critical points

---

### 9. `comprehensive_demo.rs`
**Focus**: End-to-end workflow combining multiple Symmetrica features

Comprehensive demonstration:
- Building complex expressions: (x+1)²·sin(x)/(x²+1)
- Simplification
- LaTeX output
- Plotting
- Differentiation
- Pattern substitution (x → 2x)
- Multiple visualizations

**Outputs**: `comprehensive_original.svg`, `comprehensive_derivative.svg`, `comprehensive_substituted.svg`, etc.

**Key concepts**: Complete workflow, multi-crate integration

---

### 10. `edge_cases.rs`
**Focus**: Special behaviors and edge cases

Demonstrates how the plotter handles:
- Discontinuities: 1/x at x=0
- Domain restrictions: ln(x) for x≤0
- Vertical asymptotes: sin(x)/cos(x)
- Rapid oscillation: sin(1/x)
- Large dynamic range: exp(x)
- Constant functions
- Variable mismatches
- Steep slopes: x¹⁰
- Complex domain: x^(-1/2)
- Minimum sampling

**Outputs**: `edge_discontinuity.svg`, `edge_domain_restriction.svg`, `edge_vertical_asymptote.svg`, etc.

**Key concepts**: Robustness, non-finite handling, auto-scaling, deterministic output

---

## Running All Examples

To run all examples at once:

```bash
for example in basic_polynomial trigonometric exponential_logarithm rational_functions power_functions composite_functions plot_configuration calculus_visualization comprehensive_demo edge_cases; do
    echo "Running $example..."
    cargo run --example "$example"
done
```

## Output Files

All examples generate SVG files in the current working directory. SVG files are:
- **Viewable** in any modern web browser
- **Scalable** without quality loss
- **Text-based** and version-control friendly
- **Deterministic** with fixed 6-digit precision for coordinates

## Key Features Demonstrated

Across all examples, you'll see:

1. **Expression Building**: Constructing symbolic expressions with `Store`
2. **Function Types**: Add, Mul, Pow, Symbol, Integer, Rational, Function
3. **Standard Functions**: sin, cos, exp, ln
4. **Evaluation**: f64 numerical evaluation via `eval_f64`
5. **Plotting**: SVG generation via `plot_svg` with `PlotConfig`
6. **Integration**: Working with other crates (calculus, simplify, pattern, io)
7. **Edge Case Handling**: Discontinuities, domain restrictions, auto-scaling

## Architecture Notes

The plot crate:
- Uses **no external dependencies** (only `expr_core`)
- Provides **deterministic output** (fixed precision)
- Handles **non-finite values** gracefully (gaps in polyline)
- **Auto-scales** y-axis from finite sample points
- Supports **configurable** sampling, ranges, and dimensions

## Tips

- **Sampling**: More samples = smoother curves but larger files. Start with 100-200 for most functions.
- **Range**: Choose x-range to show interesting features. Use domain knowledge (e.g., ln(x) needs x>0).
- **Dimensions**: Standard 400×300 works well for documentation. Use larger for presentations.
- **Functions**: Only sin, cos, exp, ln are supported. Use compositions for more complex behavior.

## Further Exploration

Try modifying the examples:
- Change the expressions being plotted
- Adjust PlotConfig parameters
- Combine multiple expressions
- Add your own functions using `Store::func`
- Experiment with different ranges for different behaviors

## Documentation

For API documentation, see:
```bash
cargo doc --open -p plot
```

The main entry points are:
- `plot::PlotConfig`: Configuration for plots
- `plot::plot_svg`: Generate SVG string from expression
- `plot::eval_f64`: Evaluate expression at a point
