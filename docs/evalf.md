# evalf - Numeric Evaluation Module

## Overview

The `evalf` crate provides arbitrary-precision floating-point evaluation of symbolic expressions. It converts symbolic expressions to numeric values by substituting variables with concrete numbers and evaluating all operations.

## Core Types

### EvalContext

```rust
pub struct EvalContext {
    bindings: HashMap<String, f64>,
}
```

A context that holds variable bindings for evaluation.

**Methods:**
```rust
pub fn new() -> Self
pub fn bind<S: Into<String>>(&mut self, name: S, value: f64) -> &mut Self
pub fn get(&self, name: &str) -> Option<f64>
pub fn clear(&mut self)
```

### EvalError

```rust
pub enum EvalError {
    UnboundVariable(String),
    UnknownFunction(String),
    DomainError(String),
    NonFinite,
}
```

Error types that can occur during evaluation.

## API

### Main Evaluation Function

```rust
pub fn eval(store: &Store, id: ExprId, ctx: &EvalContext) -> Result<f64, EvalError>
```

Evaluates an expression to a floating-point number using the given context.

**Example:**
```rust
use expr_core::Store;
use evalf::{eval, EvalContext};

let mut st = Store::new();
let x = st.sym("x");
let expr = st.pow(x, st.int(2));  // x^2

let mut ctx = EvalContext::new();
ctx.bind("x", 5.0);

let result = eval(&st, expr, &ctx).unwrap();
assert_eq!(result, 25.0);
```

### Convenience Function

```rust
pub fn eval_at(store: &Store, id: ExprId, var: &str, value: f64) -> Result<f64, EvalError>
```

Evaluates an expression with a single variable binding.

**Example:**
```rust
let result = eval_at(&st, expr, "x", 5.0).unwrap();
assert_eq!(result, 25.0);
```

## Supported Operations

### Arithmetic

- **Addition**: Sum of all terms
- **Multiplication**: Product of all factors
- **Power**: Base raised to exponent using `powf`

### Constants

- **Integer**: Converted to f64
- **Rational**: Computed as `numerator / denominator`

### Symbols

Looked up in the evaluation context. Returns `UnboundVariable` error if not bound.

### Functions

#### Trigonometric Functions

- `sin(x)`: Sine
- `cos(x)`: Cosine
- `tan(x)`: Tangent
- `asin(x)`, `arcsin(x)`: Arcsine (domain: [-1, 1])
- `acos(x)`, `arccos(x)`: Arccosine (domain: [-1, 1])
- `atan(x)`, `arctan(x)`: Arctangent

#### Hyperbolic Functions

- `sinh(x)`: Hyperbolic sine
- `cosh(x)`: Hyperbolic cosine
- `tanh(x)`: Hyperbolic tangent

#### Exponential and Logarithmic

- `exp(x)`: Exponential function (e^x)
- `ln(x)`, `log(x)`: Natural logarithm (domain: x > 0)
- `log10(x)`: Base-10 logarithm (domain: x > 0)
- `log2(x)`: Base-2 logarithm (domain: x > 0)

#### Other Functions

- `sqrt(x)`: Square root (domain: x >= 0)
- `abs(x)`: Absolute value
- `floor(x)`: Floor function (largest integer <= x)
- `ceil(x)`: Ceiling function (smallest integer >= x)
- `round(x)`: Round to nearest integer

#### Multi-argument Functions

- `atan2(y, x)`: Two-argument arctangent
- `min(x1, x2, ...)`: Minimum value (variadic)
- `max(x1, x2, ...)`: Maximum value (variadic)

## Examples

### Basic Polynomial

```rust
use expr_core::Store;
use evalf::{eval, EvalContext};

let mut st = Store::new();
let x = st.sym("x");

// Build: 2x^2 + 3x + 1
let expr = st.add(vec![
    st.mul(vec![st.int(2), st.pow(x, st.int(2))]),
    st.mul(vec![st.int(3), x]),
    st.int(1),
]);

let mut ctx = EvalContext::new();
ctx.bind("x", 2.0);

let result = eval(&st, expr, &ctx).unwrap();
// 2(4) + 3(2) + 1 = 8 + 6 + 1 = 15
assert_eq!(result, 15.0);
```

### Trigonometric Expression

```rust
let mut st = Store::new();
let x = st.sym("x");

// Build: sin(x)^2 + cos(x)^2
let sin_x = st.func("sin", vec![x]);
let cos_x = st.func("cos", vec![x]);
let sin2 = st.pow(sin_x, st.int(2));
let cos2 = st.pow(cos_x, st.int(2));
let expr = st.add(vec![sin2, cos2]);

let result = eval_at(&st, expr, "x", 0.5).unwrap();
// sin^2 + cos^2 = 1 (for any x)
assert!((result - 1.0).abs() < 1e-10);
```

### Multiple Variables

```rust
let mut st = Store::new();
let x = st.sym("x");
let y = st.sym("y");

// Build: x^2 + y^2
let expr = st.add(vec![
    st.pow(x, st.int(2)),
    st.pow(y, st.int(2)),
]);

let mut ctx = EvalContext::new();
ctx.bind("x", 3.0);
ctx.bind("y", 4.0);

let result = eval(&st, expr, &ctx).unwrap();
// 3^2 + 4^2 = 9 + 16 = 25
assert_eq!(result, 25.0);
```

### Rational Functions

```rust
let mut st = Store::new();
let x = st.sym("x");

// Build: (x + 1) / (x - 1)
let num = st.add(vec![x, st.int(1)]);
let den = st.add(vec![x, st.int(-1)]);
let expr = st.mul(vec![num, st.pow(den, st.int(-1))]);

let result = eval_at(&st, expr, "x", 3.0).unwrap();
// (3 + 1) / (3 - 1) = 4 / 2 = 2
assert_eq!(result, 2.0);
```

### Logarithmic Identities

```rust
let mut st = Store::new();
let x = st.sym("x");

// Build: exp(ln(x))
let ln_x = st.func("ln", vec![x]);
let exp_ln_x = st.func("exp", vec![ln_x]);

let result = eval_at(&st, exp_ln_x, "x", 5.0).unwrap();
// exp(ln(x)) = x
assert_eq!(result, 5.0);
```

## Error Handling

### Unbound Variables

```rust
let mut st = Store::new();
let x = st.sym("x");
let ctx = EvalContext::new();  // Empty context

let result = eval(&st, x, &ctx);
assert!(matches!(result, Err(EvalError::UnboundVariable(_))));
```

### Domain Errors

```rust
let mut st = Store::new();

// ln of negative number
let expr = st.func("ln", vec![st.int(-1)]);
let result = eval(&st, expr, &EvalContext::new());
assert!(matches!(result, Err(EvalError::DomainError(_))));

// sqrt of negative number
let expr = st.func("sqrt", vec![st.int(-4)]);
let result = eval(&st, expr, &EvalContext::new());
assert!(matches!(result, Err(EvalError::DomainError(_))));

// asin outside [-1, 1]
let expr = st.func("asin", vec![st.int(2)]);
let result = eval(&st, expr, &EvalContext::new());
assert!(matches!(result, Err(EvalError::DomainError(_))));
```

### Unknown Functions

```rust
let mut st = Store::new();
let expr = st.func("unknown_function", vec![st.int(1)]);
let result = eval(&st, expr, &EvalContext::new());
assert!(matches!(result, Err(EvalError::UnknownFunction(_))));
```

### Non-Finite Results

```rust
// Division by zero produces infinity
let mut st = Store::new();
let expr = st.mul(vec![st.int(1), st.pow(st.int(0), st.int(-1))]);
let result = eval(&st, expr, &EvalContext::new());
assert!(matches!(result, Err(EvalError::NonFinite)));
```

## Use Cases

### Numerical Verification

```rust
// Verify symbolic derivative numerically
let mut st = Store::new();
let x = st.sym("x");
let f = st.pow(x, st.int(3));  // x^3

let df = calculus::diff(&mut st, f, "x");  // 3x^2

// Check at x = 2
let value = eval_at(&st, df, "x", 2.0).unwrap();
assert_eq!(value, 12.0);  // 3(2^2) = 12
```

### Plotting Integration

The `plot` crate already uses similar evaluation logic. The `evalf` crate provides a more comprehensive and reusable implementation.

### Root Finding

```rust
// Find approximate roots by evaluation
let mut st = Store::new();
let x = st.sym("x");
let f = st.add(vec![st.pow(x, st.int(2)), st.int(-2)]);  // x^2 - 2

// Evaluate at candidate point
let at_1_4 = eval_at(&st, f, "x", 1.4).unwrap();
let at_1_5 = eval_at(&st, f, "x", 1.5).unwrap();
// Root is between 1.4 and 1.5 (sign change)
assert!(at_1_4 < 0.0 && at_1_5 > 0.0);
```

### Testing Simplification

```rust
// Verify simplification is numerically equivalent
let mut st = Store::new();
let x = st.sym("x");

let original = st.mul(vec![x, x]);  // x * x
let simplified = simplify::simplify(&mut st, original);  // x^2

let val1 = eval_at(&st, original, "x", 7.0).unwrap();
let val2 = eval_at(&st, simplified, "x", 7.0).unwrap();
assert_eq!(val1, val2);
```

## Performance

- **Time complexity**: O(n) where n is the expression size
- **Space complexity**: O(depth) for recursion stack
- **Typical latency**: < 1 microsecond for expressions with ~100 nodes

## Precision

Current implementation uses `f64` (IEEE 754 double precision):
- **Precision**: ~15-17 decimal digits
- **Range**: ±10^±308

**Limitations:**
- Subject to floating-point rounding errors
- Not suitable for exact rational arithmetic (use symbolic methods)
- Large integer calculations may lose precision

## Future Enhancements

### Arbitrary Precision

Add MPFR backend via feature flag:
```toml
[features]
mpfr = ["rug"]

[dependencies]
rug = { version = "1.19", optional = true }
```

```rust
pub fn eval_mpfr(
    store: &Store,
    id: ExprId,
    ctx: &EvalContext,
    precision: u32,
) -> Result<rug::Float, EvalError>
```

### Interval Arithmetic

For certified numerical bounds:
```rust
pub fn eval_interval(
    store: &Store,
    id: ExprId,
    ctx: &IntervalContext,
) -> Result<Interval, EvalError>
```

### Complex Numbers

Support for complex-valued evaluation:
```rust
pub fn eval_complex(
    store: &Store,
    id: ExprId,
    ctx: &ComplexContext,
) -> Result<Complex<f64>, EvalError>
```

### Automatic Differentiation

Forward-mode AD for gradient computation:
```rust
pub fn eval_with_gradient(
    store: &Store,
    id: ExprId,
    ctx: &EvalContext,
) -> Result<(f64, Vec<f64>), EvalError>
```

## Testing

Comprehensive test coverage:
- Basic arithmetic operations
- All supported functions
- Domain error handling
- Multi-variable expressions
- Edge cases (division by zero, etc.)

Run tests:
```bash
cargo test -p evalf
```

## Integration with Other Modules

### calculus
Numerical verification of symbolic derivatives and integrals.

### solver
Numeric root finding and equation solving.

### plot
Already uses similar evaluation logic; can migrate to use `evalf`.

### simplify
Numerical testing of simplification equivalence.

## References

- Depends on: `expr_core`
- Used by: Testing, plotting, numeric solving
- Related: `plot` module (uses similar f64 evaluation)
