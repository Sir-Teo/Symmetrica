# Phase 6: Enhanced Simplification Examples

Complete examples demonstrating v2.0 advanced simplification features.

## Trigonometric Identities

### Product-to-Sum Formulas

```rust
use expr_core::Store;
use simplify::simplify_trig;

fn main() {
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");
    
    // sin(x)cos(y) → [sin(x+y) + sin(x-y)]/2
    let sin_x = st.func("sin", vec![x]);
    let cos_y = st.func("cos", vec![y]);
    let product = st.mul(vec![sin_x, cos_y]);
    
    let result = simplify_trig(&mut st, product);
    println!("{}", st.to_string(result));
    // Output: 1/2 * (sin(x + y) + sin(x + (-1) * y))
}
```

**Supported patterns:**
- `sin(A)cos(B) → [sin(A+B) + sin(A-B)]/2`
- `cos(A)cos(B) → [cos(A+B) + cos(A-B)]/2`
- `sin(A)sin(B) → [cos(A-B) - cos(A+B)]/2`

### Sum-to-Product Formulas

```rust
use expr_core::Store;
use simplify::simplify_trig;

fn main() {
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");
    
    // sin(x) + sin(y) → 2sin((x+y)/2)cos((x-y)/2)
    let sin_x = st.func("sin", vec![x]);
    let sin_y = st.func("sin", vec![y]);
    let sum = st.add(vec![sin_x, sin_y]);
    
    let result = simplify_trig(&mut st, sum);
    println!("{}", st.to_string(result));
    // Output: 2 * sin(1/2 * (x + y)) * cos(1/2 * (x + (-1) * y))
}
```

**Supported patterns:**
- `sin(A) + sin(B) → 2sin((A+B)/2)cos((A-B)/2)`
- `sin(A) - sin(B) → 2cos((A+B)/2)sin((A-B)/2)`
- `cos(A) + cos(B) → 2cos((A+B)/2)cos((A-B)/2)`
- `cos(A) - cos(B) → -2sin((A+B)/2)sin((A-B)/2)`

### Half-Angle Formulas

```rust
use expr_core::Store;
use simplify::simplify_trig;

fn main() {
    let mut st = Store::new();
    let x = st.sym("x");
    
    // sin²(x/2) → (1 - cos(x))/2
    let half = st.rat(1, 2);
    let x_half = st.mul(vec![half, x]);
    let sin_half = st.func("sin", vec![x_half]);
    let two = st.int(2);
    let sin_sq = st.pow(sin_half, two);
    
    let result = simplify_trig(&mut st, sin_sq);
    println!("{}", st.to_string(result));
    // Output: 1/2 * (1 + (-1) * cos(x))
}
```

**Supported patterns:**
- `sin²(x/2) → (1 - cos(x))/2`
- `cos²(x/2) → (1 + cos(x))/2`
- `tan²(x/2) → (1 - cos(x))/(1 + cos(x))`

## Radical Simplification

### Perfect Squares and Powers

```rust
use expr_core::Store;
use simplify::simplify_radicals;

fn main() {
    let mut st = Store::new();
    
    // √16 → 4
    let sixteen = st.int(16);
    let half = st.rat(1, 2);
    let sqrt_16 = st.pow(sixteen, half);
    let result = simplify_radicals(&mut st, sqrt_16);
    println!("{}", st.to_string(result)); // 4
    
    // √(9/4) → 3/2
    let nine_fourths = st.rat(9, 4);
    let sqrt_rat = st.pow(nine_fourths, half);
    let result2 = simplify_radicals(&mut st, sqrt_rat);
    println!("{}", st.to_string(result2)); // 3/2
    
    // √(x⁴) → x²
    let x = st.sym("x");
    let four = st.int(4);
    let x4 = st.pow(x, four);
    let sqrt_x4 = st.pow(x4, half);
    let result3 = simplify_radicals(&mut st, sqrt_x4);
    println!("{}", st.to_string(result3)); // x^2
}
```

### Factoring Perfect Squares

```rust
use expr_core::Store;
use simplify::simplify_radicals;

fn main() {
    let mut st = Store::new();
    let x = st.sym("x");
    
    // √(4x) → 2√x
    let four = st.int(4);
    let four_x = st.mul(vec![four, x]);
    let half = st.rat(1, 2);
    let sqrt_4x = st.pow(four_x, half);
    
    let result = simplify_radicals(&mut st, sqrt_4x);
    println!("{}", st.to_string(result));
    // Output: 2 * x^(1/2)
}
```

### Denominator Rationalization

```rust
use expr_core::Store;
use simplify::simplify_radicals;

fn main() {
    let mut st = Store::new();
    let x = st.sym("x");
    
    // 1/√x → √x/x
    let neg_half = st.rat(-1, 2);
    let one_over_sqrt_x = st.pow(x, neg_half);
    
    let result = simplify_radicals(&mut st, one_over_sqrt_x);
    // Result is rationalized form
}
```

## Logarithm & Exponential Rules

### Product Rule (Assumption-Guarded)

```rust
use expr_core::Store;
use simplify::simplify_logarithms;
use assumptions::{Context, Prop};

fn main() {
    let mut st = Store::new();
    let mut ctx = Context::new();
    let x = st.sym("x");
    let y = st.sym("y");
    
    // Assume x, y > 0 for safe expansion
    ctx.assume("x", Prop::Positive);
    ctx.assume("y", Prop::Positive);
    
    // log(xy) → log(x) + log(y)
    let product = st.mul(vec![x, y]);
    let ln_prod = st.func("ln", vec![product]);
    
    let result = simplify_logarithms(&mut st, ln_prod, &ctx);
    println!("{}", st.to_string(result));
    // Output: ln(x) + ln(y)
}
```

### Power Rule

```rust
use expr_core::Store;
use simplify::simplify_logarithms;
use assumptions::{Context, Prop};

fn main() {
    let mut st = Store::new();
    let mut ctx = Context::new();
    let x = st.sym("x");
    
    ctx.assume("x", Prop::Positive);
    
    // log(x^n) → n·log(x)
    let three = st.int(3);
    let x3 = st.pow(x, three);
    let ln_x3 = st.func("ln", vec![x3]);
    
    let result = simplify_logarithms(&mut st, ln_x3, &ctx);
    println!("{}", st.to_string(result));
    // Output: 3 * ln(x)
}
```

### Quotient Rule

```rust
use expr_core::Store;
use simplify::simplify_logarithms;
use assumptions::{Context, Prop};

fn main() {
    let mut st = Store::new();
    let mut ctx = Context::new();
    let x = st.sym("x");
    let y = st.sym("y");
    
    ctx.assume("x", Prop::Positive);
    ctx.assume("y", Prop::Positive);
    
    // log(x/y) → log(x) - log(y)
    let neg_one = st.int(-1);
    let y_inv = st.pow(y, neg_one);
    let quotient = st.mul(vec![x, y_inv]);
    let ln_quot = st.func("ln", vec![quotient]);
    
    let result = simplify_logarithms(&mut st, ln_quot, &ctx);
    // Output: ln(x) + (-1) * ln(y)
}
```

### Logarithm Contraction

```rust
use expr_core::Store;
use simplify::contract_logarithms;
use assumptions::Context;

fn main() {
    let mut st = Store::new();
    let ctx = Context::new();
    let x = st.sym("x");
    let y = st.sym("y");
    
    // log(x) + log(y) → log(xy)
    let ln_x = st.func("ln", vec![x]);
    let ln_y = st.func("ln", vec![y]);
    let sum = st.add(vec![ln_x, ln_y]);
    
    let result = contract_logarithms(&mut st, sum, &ctx);
    // Output: ln(x * y)
}
```

## Branch-Cut Awareness

The logarithm rules are **branch-cut aware** - they only apply when domain constraints are satisfied:

```rust
use expr_core::Store;
use simplify::simplify_logarithms;
use assumptions::Context;

fn main() {
    let mut st = Store::new();
    let ctx = Context::new(); // No assumptions
    let x = st.sym("x");
    let y = st.sym("y");
    
    let product = st.mul(vec![x, y]);
    let ln_prod = st.func("ln", vec![product]);
    
    // Without positivity assumptions, NO expansion occurs
    let result = simplify_logarithms(&mut st, ln_prod, &ctx);
    // Output: ln(x * y) (unchanged - safe!)
}
```

## Performance Notes

- **Trigonometric simplification:** O(n²) for n terms (pattern matching)
- **Radical simplification:** O(1) for perfect squares, O(n) for factoring
- **Logarithm rules:** O(n) with assumption checks
- **All operations:** Zero allocations for cache hits (hash-consing)

## Test Coverage

- **Phase 6 Total:** 104 tests (58 unit + 46 integration)
  - Trig identities: 30 tests
  - Radical simplification: 17 tests
  - Logarithm rules: 18 tests
- **All tests:** 100% passing
- **CI gates:** fmt, clippy, tests all green

## API Reference

```rust
// Public exports from simplify crate
pub use simplify_trig;           // Trigonometric identities
pub use simplify_radicals;       // Radical simplification
pub use simplify_logarithms;     // Log expansion (guarded)
pub use contract_logarithms;     // Log contraction
```

## See Also

- [Phase 5: Symbolic Summation](phase5-examples.md)
- [API Documentation](api.md)
- [Interactive Playground](playground.html)
