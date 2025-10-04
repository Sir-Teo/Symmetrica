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

### Conservative Strategy

Integration returns `None` when:
- Pattern is not recognized
- Integration by parts is needed
- Substitution is required
- Result involves special functions (erf, Si, etc.)

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
- No integration by parts
- No trigonometric substitution
- No advanced techniques (Risch algorithm not implemented)
- Partial fractions limited to distinct linear factors over Q

**Series:**
- No Laurent series (negative powers)
- Limited function repertoire
- No automatic radius of convergence

## Testing

Comprehensive tests cover:
- Power, sum, product rules
- Chain rule for nested functions
- Integration patterns (power rule, trig, exponential, u'/u)
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
- Integration by parts
- Trigonometric substitution
- Taylor series at arbitrary points
- Radius of convergence computation
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
