# calculus - Calculus Module

## Overview

The `calculus` crate provides symbolic differentiation, integration, series expansion, and limits for expressions. It implements standard calculus rules with chain rule support, conservative integration patterns, and Maclaurin series generation.

## Modules

- **diff**: Differentiation rules
- **integrate**: Conservative integration patterns
- **series**: Maclaurin series expansion and polynomial limits

## Differentiation

### Main Function

```rust
pub fn diff(store: &mut Store, id: ExprId, var: &str) -> ExprId
```

Computes the derivative of expression `id` with respect to variable `var`.

### Supported Rules

#### Constants
```rust
d/dx (c) = 0  // Any constant
```

#### Symbols
```rust
d/dx (x) = 1      // With respect to itself
d/dx (y) = 0      // Other symbols
```

#### Addition (Linearity)
```rust
d/dx (f + g) = f' + g'
```

#### Multiplication (Product Rule)
For n factors: `d/dx (f₁ * f₂ * ... * fₙ) = Σᵢ (f'ᵢ * ∏ⱼ≠ᵢ fⱼ)`

Example:
```rust
let mut st = Store::new();
let x = st.sym("x");
let two = st.int(2);
let x2 = st.pow(x, two);
let xp1 = st.add(vec![x, st.int(1)]);
let expr = st.mul(vec![x2, xp1]);  // x^2 * (x+1)

let derivative = diff(&mut st, expr, "x");
// Result: 2x*(x+1) + x^2*1 = 3x^2 + 2x
```

#### Power Rule (Integer Exponent)
```rust
d/dx (u^n) = n * u^(n-1) * u'  // n integer
```

**Special cases:**
- `d/dx (x^0) = 0`
- `d/dx (x^1) = 1`

#### General Power Rule
```rust
d/dx (u^v) = u^v * (v' * ln(u) + v * u'/u)
```

Used when exponent is non-constant or non-integer:
```rust
let x_to_x = st.pow(x, x);
let derivative = diff(&mut st, x_to_x, "x");
// Result: x^x * (ln(x) + 1)
```

#### Trigonometric Functions
```rust
d/dx sin(u) = cos(u) * u'
d/dx cos(u) = -sin(u) * u'
```

#### Hyperbolic Functions (v1.1)
```rust
d/dx sinh(u) = cosh(u) * u'
d/dx cosh(u) = sinh(u) * u'
d/dx tanh(u) = (1 - tanh^2(u)) * u'
```

#### Exponential and Logarithm
```rust
d/dx exp(u) = exp(u) * u'
d/dx ln(u) = u'/u = u' * u^(-1)
d/dx log(u) = u'/u  // Treated same as ln
```

### Chain Rule

All function derivatives automatically apply the chain rule:
```rust
let sin_x2 = st.func("sin", vec![st.pow(x, st.int(2))]);
let derivative = diff(&mut st, sin_x2, "x");
// Result: cos(x^2) * 2x
```

### Examples

**Basic polynomial:**
```rust
let mut st = Store::new();
let x = st.sym("x");
// f(x) = x^3 + 2x
let expr = st.add(vec![
    st.pow(x, st.int(3)),
    st.mul(vec![st.int(2), x]),
]);
let df = diff(&mut st, expr, "x");
// Result: 3x^2 + 2
```

**Composite function:**
```rust
// d/dx sin(x^2)
let x2 = st.pow(x, st.int(2));
let sin_x2 = st.func("sin", vec![x2]);
let derivative = diff(&mut st, sin_x2, "x");
// Result: cos(x^2) * 2x
```

## Integration

### Main Function

```rust
pub fn integrate(store: &mut Store, id: ExprId, var: &str) -> Option<ExprId>
```

Attempts to integrate expression `id` with respect to `var`. Returns `None` if no pattern matches.

### Supported Patterns

#### Constants and Symbols
```rust
∫ c dx = c*x          // Constant
∫ y dx = y*x          // Symbol other than var
```

#### Power Rule
```rust
∫ x^n dx = x^(n+1)/(n+1)    // n ≠ -1
∫ x^(-1) dx = ln(x)          // Special case
```

**Examples:**
```rust
let x2 = st.pow(x, st.int(2));
let integral = integrate(&mut st, x2, "x").unwrap();
// Result: (1/3) * x^3

let inv_x = st.pow(x, st.int(-1));
let integral = integrate(&mut st, inv_x, "x").unwrap();
// Result: ln(x)
```

#### Linear Trigonometric Functions
```rust
∫ sin(ax + b) dx = -(1/a) * cos(ax + b)
∫ cos(ax + b) dx = (1/a) * sin(ax + b)
```

**Example:**
```rust
let two_x = st.mul(vec![st.int(2), x]);
let sin_2x = st.func("sin", vec![two_x]);
let integral = integrate(&mut st, sin_2x, "x").unwrap();
// Result: (-1/2) * cos(2x)
```

#### Linear Exponential
```rust
∫ exp(ax + b) dx = (1/a) * exp(ax + b)
```

**Example:**
```rust
let three_x = st.mul(vec![st.int(3), x]);
let exp_3x = st.func("exp", vec![three_x]);
let integral = integrate(&mut st, exp_3x, "x").unwrap();
// Result: (1/3) * exp(3x)
```

#### Hyperbolic Functions (v1.1)
```rust
∫ sinh(ax + b) dx = (1/a) * cosh(ax + b)
∫ cosh(ax + b) dx = (1/a) * sinh(ax + b)
∫ tanh(ax + b) dx = (1/a) * ln(cosh(ax + b))
```

**Examples:**
```rust
// ∫ sinh(x) dx = cosh(x)
let sinhx = st.func("sinh", vec![x]);
let integral = integrate(&mut st, sinhx, "x").unwrap();
// Result: cosh(x)

// ∫ tanh(2x) dx = (1/2) * ln(cosh(2x))
let two_x = st.mul(vec![st.int(2), x]);
let tanh_2x = st.func("tanh", vec![two_x]);
let integral = integrate(&mut st, tanh_2x, "x").unwrap();
// Result: (1/2) * ln(cosh(2x))
```

#### Logarithmic Integral (u'/u pattern)
```rust
∫ u'/u dx = ln(u)
```

**Example:**
```rust
let u = st.add(vec![st.pow(x, st.int(2)), st.int(1)]);  // x^2 + 1
let du = diff(&mut st, u, "x");                          // 2x
let integrand = st.mul(vec![du, st.pow(u, st.int(-1))]);
let integral = integrate(&mut st, integrand, "x").unwrap();
// Result: ln(x^2 + 1)
```

#### Partial Fractions

Automatic partial fraction decomposition for rational functions with distinct linear factors:
```rust
// ∫ (2x + 3)/(x^2 + 3x + 2) dx
// Denominator factors as (x+1)(x+2)
// → ln(x+1) + ln(x+2)
```

The integrator detects this pattern and applies `polys::partial_fractions_simple`.

#### Integration by Parts

For products of functions, the integrator uses the **LIATE heuristic** to choose `u` and `dv`:
- **L**ogarithmic (highest priority for `u`)
- **I**nverse trigonometric
- **A**lgebraic (polynomials)
- **T**rigonometric
- **E**xponential (lowest priority for `u`)

Formula: `∫ u dv = uv - ∫ v du`

**Example:**
```rust
// ∫ ln(x) * x^2 dx
let lnx = st.func("ln", vec![x]);
let x2 = st.pow(x, st.int(2));
let product = st.mul(vec![lnx, x2]);
let integral = integrate(&mut st, product, "x").unwrap();

// Chooses: u = ln(x), dv = x^2 dx
// Computes: du = 1/x dx, v = x^3/3
// Result: (x^3 * ln(x))/3 - ∫ x^3/3 * 1/x dx
//       = (x^3 * ln(x))/3 - x^3/9
```

**Supported product patterns:**
- `ln(x) * polynomial` → Uses integration by parts with `u = ln(x)`
- `polynomial * trig/exp` → Chooses polynomial as `u`
- Products where both factors depend on the variable

#### Trigonometric Power Patterns (v1.1)

**sin(x) * cos(x):**
```rust
∫ sin(x) cos(x) dx = -cos(2x)/4
```

Using the identity `sin(x)cos(x) = sin(2x)/2`:
```rust
let sinx = st.func("sin", vec![x]);
let cosx = st.func("cos", vec![x]);
let prod = st.mul(vec![sinx, cosx]);
let integral = integrate(&mut st, prod, "x").unwrap();
// Result: -cos(2x)/4
```

**sin^2(x) using double-angle formula:**
```rust
∫ sin^2(x) dx = x/2 - sin(2x)/4
```

Using the identity `sin^2(x) = (1 - cos(2x))/2`:
```rust
let sinx = st.func("sin", vec![x]);
let two = st.int(2);
let sin2 = st.pow(sinx, two);
let integral = integrate(&mut st, sin2, "x").unwrap();
// Result: x/2 - sin(2x)/4
```

**cos^2(x) using double-angle formula:**
```rust
∫ cos^2(x) dx = x/2 + sin(2x)/4
```

Using the identity `cos^2(x) = (1 + cos(2x))/2`:
```rust
let cosx = st.func("cos", vec![x]);
let two = st.int(2);
let cos2 = st.pow(cosx, two);
let integral = integrate(&mut st, cos2, "x").unwrap();
// Result: x/2 + sin(2x)/4
```

#### U-Substitution Pattern Detection (v1.1)

Automatically detects and applies u-substitution for patterns of the form `∫ f(g(x)) * g'(x) dx`:

**Power rule with composite functions:**
```rust
∫ u^n * u' dx = u^(n+1) / (n+1) + C
```

**Example: ∫ 2x(x² + 1)⁵ dx**
```rust
let x = st.sym("x");
let two = st.int(2);
let two_x = st.mul(vec![two, x]); // u' = 2x

let x2 = st.pow(x, two);
let one = st.int(1);
let u = st.add(vec![x2, one]); // u = x² + 1
let five = st.int(5);
let u5 = st.pow(u, five); // u⁵

let integrand = st.mul(vec![two_x, u5]);
let result = integrate(&mut st, integrand, "x").unwrap();
// Result: (x² + 1)⁶ / 6
```

The engine automatically:
1. Identifies u^n patterns in products
2. Computes du/dx for the base u
3. Checks if remaining factors match c * du for some constant c
4. Applies the power rule with the adjusted coefficient

**Supported patterns:**
- `∫ 2x(x² + a)^n dx` where du = 2x dx
- `∫ 3x²(x³ + b)^n dx` where du = 3x² dx
- Any `∫ c * f'(x) * [f(x)]^n dx` pattern with rational coefficients

#### Risch Algorithm Foundation (v1.1)

The Risch algorithm is a decision procedure for symbolic integration of elementary functions. v1.1 introduces foundational components:

**Differential Field Towers:**
- Tower extension detection for exp/log structures
- Classification of expressions as base field, exponential, or logarithmic extensions
- Support for analyzing nested function structures

**Logarithmic Derivative:**
```rust
∫ f'/f dx = ln(f) + C
```

The logarithmic derivative `d/dx(ln(f)) = f'/f` is computed automatically for integration patterns.

**Example: Detect tower structure**
```rust
let expx = st.func("exp", vec![x]);
let ext = detect_extension(&st, expx, "x");
// Returns ExtensionType::Exponential(x)

let lnx = st.func("ln", vec![x]);
let ext = detect_extension(&st, lnx, "x");
// Returns ExtensionType::Logarithmic(x)
```

**Enhanced Exponential Integration:**
The Risch foundation enables cleaner integration of exponentials:
- `∫ exp(x) dx = exp(x)`
- `∫ exp(ax) dx = (1/a) exp(ax)`
- `∫ exp(ax + b) dx = (1/a) exp(ax + b)`

These patterns are now handled through the Risch framework with proper tower analysis.

#### Weierstrass Substitution (v1.1)

The Weierstrass substitution (tangent half-angle substitution) handles rational trigonometric integrals using `t = tan(x/2)`:

**Transformation Formulas:**
- `sin(x) = 2t/(1+t²)`
- `cos(x) = (1-t²)/(1+t²)`
- `dx = 2/(1+t²) dt`

**Supported Patterns:**

**∫ 1/(1 + cos(x)) dx:**
```rust
let one = st.int(1);
let cosx = st.func("cos", vec![x]);
let denom = st.add(vec![one, cosx]);
let neg_one = st.int(-1);
let integrand = st.pow(denom, neg_one); // (1 + cos(x))^(-1)

let result = integrate(&mut st, integrand, "x").unwrap();
// Result: tan(x/2)
```

**∫ 1/(1 - cos(x)) dx:**
```rust
let one = st.int(1);
let cosx = st.func("cos", vec![x]);
let neg_one = st.int(-1);
let neg_cosx = st.mul(vec![neg_one, cosx]);
let denom = st.add(vec![one, neg_cosx]); // 1 - cos(x)
let integrand = st.pow(denom, neg_one);

let result = integrate(&mut st, integrand, "x").unwrap();
// Result: -cot(x/2)
```

**Key Results:**
- `∫ 1/(1 + cos(x)) dx = tan(x/2) + C`
- `∫ 1/(1 - cos(x)) dx = -cot(x/2) + C`

These integrals are automatically recognized and integrated using the Weierstrass framework.

### Conservative Strategy

Integration returns `None` when:
- Pattern is not recognized
- Advanced substitution is required (beyond linear cases)
- Result involves special functions (erf, Si, Ci, Ei, etc.)
- Integration by parts recursion doesn't terminate

This ensures correctness over coverage.

### Examples

**Polynomial:**
```rust
let x3 = st.pow(x, st.int(3));
let integral = integrate(&mut st, x3, "x").unwrap();
// Result: (1/4) * x^4
```

**Rational function:**
```rust
// Build: (2x + 3) / (x^2 + 3x + 2)
let num = st.add(vec![st.mul(vec![st.int(2), x]), st.int(3)]);
let den = st.add(vec![st.pow(x, st.int(2)), st.mul(vec![st.int(3), x]), st.int(2)]);
let expr = st.mul(vec![num, st.pow(den, st.int(-1))]);
let simplified = simplify(&mut st, expr);
let integral = integrate(&mut st, simplified, "x").unwrap();
// Result: ln(x+1) + ln(x+2)
```

## Series Expansion

### Maclaurin Series

```rust
pub fn maclaurin(st: &Store, id: ExprId, var: &str, order: usize) -> Option<Series>
```

Computes the Maclaurin series (Taylor series at 0) up to given order.

**Supported functions:**
- `exp(u)`: Uses composition with known series
- `sin(u)`: Uses composition
- `cos(u)`: Uses composition
- `ln(1 + u)`: Direct series for `u` near 0
- Polynomials: Direct coefficient extraction

### Series Type

```rust
pub struct Series {
    pub var: String,
    pub coeffs: Vec<(i64, i64)>,  // Rational coefficients
}
```

Index `k` corresponds to the coefficient of `var^k`.

### Examples

**Exponential:**
```rust
let expx = st.func("exp", vec![x]);
let series = maclaurin(&st, expx, "x", 6).unwrap();
// coeffs[0] = (1, 1)  →  1
// coeffs[1] = (1, 1)  →  1
// coeffs[2] = (1, 2)  →  1/2
// coeffs[3] = (1, 6)  →  1/6
// Series: 1 + x + x²/2 + x³/6 + ...
```

**Sine:**
```rust
let sinx = st.func("sin", vec![x]);
let series = maclaurin(&st, sinx, "x", 6).unwrap();
// coeffs[0] = (0, 1)   →  0
// coeffs[1] = (1, 1)   →  1
// coeffs[2] = (0, 1)   →  0
// coeffs[3] = (-1, 6)  →  -1/6
// Series: x - x³/6 + ...
```

**Composition:**
```rust
// sin(x^2)
let x2 = st.pow(x, st.int(2));
let sin_x2 = st.func("sin", vec![x2]);
let series = maclaurin(&st, sin_x2, "x", 6).unwrap();
// Result: x^2 - x^6/6 + ...
```

## Limits

### Polynomial Limits

```rust
pub fn limit_poly(st: &Store, id: ExprId, var: &str, point: LimitPoint) -> LimitResult
```

Computes limits of polynomial expressions.

### LimitPoint

```rust
pub enum LimitPoint {
    Zero,
    PosInf,
}
```

### LimitResult

```rust
pub enum LimitResult {
    Finite((i64, i64)),  // Rational value
    Infinity,
    Undefined,
}
```

### Examples

**At zero:**
```rust
// lim (x^2 + 3x + 2) as x → 0
let poly = st.add(vec![st.pow(x, st.int(2)), st.mul(vec![st.int(3), x]), st.int(2)]);
let limit = limit_poly(&st, poly, "x", LimitPoint::Zero);
// Result: Finite((2, 1)) = 2
```

**At infinity:**
```rust
// lim (x^2 + 3x + 2) as x → +∞
let limit = limit_poly(&st, poly, "x", LimitPoint::PosInf);
// Result: Infinity
```

## Integration with Simplify

All calculus operations automatically simplify their results using the `simplify` crate:
```rust
let df = diff(&mut st, expr, "x");
// df is already simplified
```

## Performance

- **Differentiation**: O(n) in expression size
- **Integration**: O(n) for pattern matching; O(n²) for partial fractions
- **Series**: O(n * order) for polynomial manipulation

## Limitations

**Differentiation:**
- Unknown functions return 0 (no symbolic derivatives)
- Multi-argument functions not supported

**Integration:**
- ✅ Integration by parts (LIATE heuristic for products)
- ✅ Hyperbolic functions (sinh, cosh, tanh) - v1.1
- ✅ Trigonometric power patterns (sin²(x), cos²(x), sin(x)cos(x)) - v1.1
- ✅ U-substitution for composite functions (f(g(x)) * g'(x)) - v1.1
- No advanced trigonometric substitution (Weierstrass)
- No advanced techniques (Risch algorithm not fully implemented)
- Partial fractions limited to distinct linear factors over Q
- Rational exponents not yet supported in power rule

**Series:**
- No Laurent series (negative powers)
- Limited function repertoire
- No automatic radius of convergence

## Testing

Comprehensive tests cover:
- Power, sum, product rules
- Chain rule for nested functions
- Integration patterns (power rule, trig, exponential, u'/u)
- Integration by parts (ln(x) * polynomial products)
- Partial fractions with various denominators
- Maclaurin series for elementary functions
- Limits at 0 and infinity

Run tests:
```bash
cargo test -p calculus
```

## Future Enhancements

- Definite integration with bounds
- Multivariate calculus (partial derivatives)
- ✅ ~~Integration by parts~~ (implemented with LIATE heuristic)
- Trigonometric substitution
- Taylor series at arbitrary points
- Radius of convergence computation
- Generalized substitution (u-substitution)
- Symbolic limits (L'Hôpital's rule)

## Example: Complete Workflow

```rust
use expr_core::Store;
use calculus::{diff, integrate, maclaurin};

let mut st = Store::new();
let x = st.sym("x");

// Original function: x^3
let f = st.pow(x, st.int(3));

// Differentiate: 3x^2
let df = diff(&mut st, f, "x");
println!("f'(x) = {}", st.to_string(df));

// Integrate back: (3/4)x^4
let int_df = integrate(&mut st, df, "x").unwrap();
println!("∫f'(x) dx = {}", st.to_string(int_df));

// Series expansion of exp(x)
let exp_x = st.func("exp", vec![x]);
let series = maclaurin(&st, exp_x, "x", 5).unwrap();
println!("exp(x) ≈ 1 + x + x²/2 + x³/6 + x⁴/24");
```

## References

- Depends on: `expr_core`, `simplify`, `arith`, `polys`
- Used by: `solver`, `cli`
- Classic references: Calculus textbooks (Stewart, Apostol), CAS design papers
