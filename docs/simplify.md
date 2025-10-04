# simplify - Simplification Module

## Overview

The `simplify` crate provides explicit simplification passes on top of `expr_core`'s canonical constructors. It implements algebraic transformations including like-term collection, factor merging, and assumption-guarded logarithmic identities.

## Core Function

```rust
pub fn simplify(store: &mut Store, id: ExprId) -> ExprId
```

Recursively simplifies an expression using default (empty) assumptions.

```rust
pub fn simplify_with(store: &mut Store, id: ExprId, ctx: &Context) -> ExprId
```

Simplifies with an explicit assumptions context for guarded transformations.

## Simplification Rules

### Addition (Like-Term Collection)

Collects terms with the same "base" by extracting coefficients:
- `2x + 3x → 5x`
- `x + 2x + (1/2)x → (7/2)x`
- `2x - 2x → 0`

**Algorithm:**
1. Recursively simplify each term
2. Split each term into `(coeff, base)` where `term = coeff * base`
3. Group by base and sum coefficients
4. Rebuild as `coeff * base` for each non-zero coefficient

**Examples:**
```rust
let mut st = Store::new();
let x = st.sym("x");
let two_x = st.mul(vec![st.int(2), x]);
let three_x = st.mul(vec![st.int(3), x]);
let expr = st.add(vec![two_x, three_x]);

let simplified = simplify(&mut st, expr);
// Result: 5*x
```

### Multiplication (Power Merging)

Merges powers with the same base:
- `x^2 * x^3 → x^5`
- `x * x^2 → x^3` (treats `x` as `x^1`)
- `2 * x^2 * 3 * x^3 → 6 * x^5`

**Algorithm:**
1. Recursively simplify each factor
2. For each non-numeric factor, extract `(base, exponent)`
3. Group by base and sum exponents
4. Rebuild as `base^(summed_exponent)`

**Examples:**
```rust
let mut st = Store::new();
let x = st.sym("x");
let x2 = st.pow(x, st.int(2));
let x3 = st.pow(x, st.int(3));
let expr = st.mul(vec![x2, x3]);

let simplified = simplify(&mut st, expr);
// Result: x^5
```

### Power Simplification

**Guarded rule with assumptions:**
- `(x^2)^(1/2) → x` **if** `x` is assumed positive

Without assumptions, no simplification occurs (avoids incorrect `|x|` removal).

**Example:**
```rust
use assumptions::{Context, Prop};

let mut st = Store::new();
let x = st.sym("x");
let x2 = st.pow(x, st.int(2));
let sqrt = st.pow(x2, st.rat(1, 2));

let mut ctx = Context::new();
ctx.assume("x", Prop::Positive);

let simplified = simplify_with(&mut st, sqrt, &ctx);
// Result: x (only with positive assumption)
```

### Function Simplifications

#### ln(exp(u)) → u
Always safe transformation:
```rust
let exp_x = st.func("exp", vec![x]);
let ln_exp_x = st.func("ln", vec![exp_x]);
let simplified = simplify(&mut st, ln_exp_x);
// Result: x
```

#### exp(ln(u)) → u (guarded)
Only when `u` is assumed positive:
```rust
let mut ctx = Context::new();
ctx.assume("x", Prop::Positive);
let ln_x = st.func("ln", vec![x]);
let exp_ln_x = st.func("exp", vec![ln_x]);
let simplified = simplify_with(&mut st, exp_ln_x, &ctx);
// Result: x (with assumption)
```

#### ln(x^k) → k * ln(x) (guarded)
When `x` is positive and `k` is rational:
```rust
let mut ctx = Context::new();
ctx.assume("x", Prop::Positive);
let x3 = st.pow(x, st.int(3));
let ln_x3 = st.func("ln", vec![x3]);
let simplified = simplify_with(&mut st, ln_x3, &ctx);
// Result: 3 * ln(x)
```

#### ln(x * y) → ln(x) + ln(y) (guarded)
When all factors are positive symbols:
```rust
let mut ctx = Context::new();
ctx.assume("x", Prop::Positive);
ctx.assume("y", Prop::Positive);
let prod = st.mul(vec![x, y]);
let ln_prod = st.func("ln", vec![prod]);
let simplified = simplify_with(&mut st, ln_prod, &ctx);
// Result: ln(x) + ln(y)
```

#### ln(x/y) → ln(x) - ln(y) (guarded)
Recognizes `x * y^(-1)` pattern when both are positive:
```rust
let mut ctx = Context::new();
ctx.assume("x", Prop::Positive);
ctx.assume("y", Prop::Positive);
let inv_y = st.pow(y, st.int(-1));
let quot = st.mul(vec![x, inv_y]);
let ln_quot = st.func("ln", vec![quot]);
let simplified = simplify_with(&mut st, ln_quot, &ctx);
// Result: ln(x) - ln(y)
```

## Assumptions System

The `assumptions` crate provides a tri-valued logic system for symbol properties:

### Properties (Prop)
- `Real`: Symbol is real-valued
- `Positive`: Symbol is positive (> 0)
- `Integer`: Symbol is an integer
- `Nonzero`: Symbol is non-zero

### Context
```rust
let mut ctx = Context::new();
ctx.assume("x", Prop::Positive);
ctx.assume("n", Prop::Integer);

// Query: returns Truth::True, Truth::False, or Truth::Unknown
let is_positive = ctx.has("x", Prop::Positive);  // Truth::True
let is_integer = ctx.has("x", Prop::Integer);    // Truth::Unknown
```

### Guarded Transformations

Simplifications that depend on assumptions only fire when the context confirms the required properties. This prevents incorrect transformations like:
- `sqrt(x^2) → x` without knowing sign of `x` (should be `|x|`)
- `ln(e^x) → x` when `x` might be complex (domain issues)

## Idempotence

Simplification is idempotent:
```rust
let s1 = simplify(&mut st, expr);
let s2 = simplify(&mut st, s1);
assert_eq!(s1, s2);
```

Applying simplify multiple times produces the same result after the first pass.

## Recursion Strategy

Simplification is **bottom-up recursive**:
1. Simplify all children first
2. Apply local transformation rules
3. Rebuild with canonical constructors

This ensures that nested structures are fully simplified.

## Helper Functions

### split_coeff
```rust
fn split_coeff(store: &mut Store, id: ExprId) -> ((i64, i64), ExprId)
```

Decomposes a term into `(rational_coefficient, base)` where:
- `3x → ((3, 1), x)`
- `(1/2)x^2 → ((1, 2), x^2)`
- `5 → ((5, 1), 1)`

### is_one
```rust
fn is_one(store: &Store, id: ExprId) -> bool
```

Tests if an expression is the integer `1`.

### is_positive_symbol
```rust
fn is_positive_symbol(ctx: &Context, store: &Store, id: ExprId) -> bool
```

Returns true if `id` is a symbol known to be positive in the given context.

## Integration with expr_core

Simplify uses the canonical constructors from `expr_core`:
- All rebuilt expressions maintain canonical form
- Digests remain stable for equivalent expressions
- Hash-consing prevents duplication

## Performance

- **Time complexity**: O(n) in expression size for a single pass
- **Space complexity**: O(n) for temporary storage during recursion
- **Idempotent**: Single pass suffices; repeated calls are no-ops

## Limitations

Current limitations (potential future enhancements):
- No trigonometric simplifications (e.g., `sin^2 + cos^2 → 1`)
- No polynomial factoring beyond power merging
- No partial fraction decomposition
- Limited to rational coefficients (no algebraic numbers)

## Testing

Comprehensive test suite covers:
- Like-term collection with various coefficients
- Power merging (simple, with coefficients, multiple bases)
- Cancellation to zero
- Function argument simplification
- Assumption-guarded transformations
- Edge cases (empty sums/products, single terms)

Run tests:
```bash
cargo test -p simplify
```

## Example: Complete Workflow

```rust
use expr_core::Store;
use simplify::simplify;

let mut st = Store::new();
let x = st.sym("x");

// Build: 2x + 3x + x^2 + x^2
let two_x = st.mul(vec![st.int(2), x]);
let three_x = st.mul(vec![st.int(3), x]);
let x2_1 = st.pow(x, st.int(2));
let x2_2 = st.pow(x, st.int(2));

let expr = st.add(vec![two_x, three_x, x2_1, x2_2]);
println!("Before: {}", st.to_string(expr));

let simplified = simplify(&mut st, expr);
println!("After:  {}", st.to_string(simplified));
// Result: "2 * x^2 + 5 * x"
```

## Best Practices

1. **Always simplify after construction**: Canonical forms from `expr_core` are minimal; explicit simplification yields better results.
2. **Use assumptions judiciously**: Add only what you know to avoid over-simplification.
3. **Check idempotence in tests**: Ensures no oscillation or infinite rewrites.
4. **Profile before optimizing**: Most expressions simplify quickly; focus on algorithmic improvements.

## References

- Depends on: `expr_core`, `arith`, `assumptions`
- Used by: `calculus`, `solver`, `cli`
- Related patterns: Term rewriting, canonical forms, e-graphs (future direction)
