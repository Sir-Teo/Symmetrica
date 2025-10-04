# assumptions - Assumptions and Constraints Module

## Overview

The `assumptions` crate provides a tri-valued logic system for tracking symbolic properties and constraints. It enables assumption-guarded transformations in the simplification engine, ensuring mathematical correctness.

## Core Types

### Truth

```rust
pub enum Truth {
    True,
    False,
    Unknown,
}
```

Represents the tri-valued logic result of a property query.

### Prop

```rust
pub enum Prop {
    Real,      // Symbol is real-valued
    Positive,  // Symbol is positive (> 0)
    Integer,   // Symbol is an integer
    Nonzero,   // Symbol is non-zero
}
```

Properties that can be assumed about symbols.

### Context

```rust
pub struct Context {
    map: HashMap<String, HashSet<Prop>>,
}
```

Stores assumptions about symbols. Each symbol can have multiple properties.

## API

### Creating a Context

```rust
pub fn new() -> Self
```

Creates an empty assumptions context:
```rust
use assumptions::Context;

let ctx = Context::new();
```

Also available via `Default`:
```rust
let ctx = Context::default();
```

### Making Assumptions

```rust
pub fn assume<S: Into<String>>(&mut self, sym: S, prop: Prop)
```

Declares that a symbol has a given property:
```rust
use assumptions::{Context, Prop};

let mut ctx = Context::new();
ctx.assume("x", Prop::Positive);
ctx.assume("x", Prop::Real);      // Can assume multiple properties
ctx.assume("n", Prop::Integer);
```

### Querying Assumptions

```rust
pub fn has(&self, sym: &str, prop: Prop) -> Truth
```

Checks if a symbol has a property:
```rust
use assumptions::Truth;

let result = ctx.has("x", Prop::Positive);
match result {
    Truth::True => println!("x is positive"),
    Truth::False => println!("x is not positive"),
    Truth::Unknown => println!("Unknown if x is positive"),
}
```

**Return values:**
- `Truth::True`: Property is explicitly assumed
- `Truth::Unknown`: No information about this property (or symbol not in context)
- `Truth::False`: Never returned currently (closed-world assumption)

## Usage in Simplification

The `simplify` crate uses assumptions for guarded transformations.

### Example: Square Root Simplification

```rust
use expr_core::Store;
use simplify::simplify_with;
use assumptions::{Context, Prop};

let mut st = Store::new();
let x = st.sym("x");

// Build: (x^2)^(1/2)
let x2 = st.pow(x, st.int(2));
let sqrt_x2 = st.pow(x2, st.rat(1, 2));

// Without assumptions: no simplification
let result1 = simplify_with(&mut st, sqrt_x2, &Context::default());
// Result: (x^2)^(1/2) (unchanged)

// With positive assumption: simplifies to x
let mut ctx = Context::new();
ctx.assume("x", Prop::Positive);
let result2 = simplify_with(&mut st, sqrt_x2, &ctx);
// Result: x
```

**Why?** Without knowing x's sign, `√(x²) = |x|`, not `x`. With positivity, we safely simplify.

### Example: exp(ln(x)) Simplification

```rust
let ln_x = st.func("ln", vec![x]);
let exp_ln_x = st.func("exp", vec![ln_x]);

// Without assumption: no simplification (ln domain issues)
let result1 = simplify_with(&mut st, exp_ln_x, &Context::default());
// Result: exp(ln(x))

// With positive assumption: simplifies to x
let mut ctx = Context::new();
ctx.assume("x", Prop::Positive);
let result2 = simplify_with(&mut st, exp_ln_x, &ctx);
// Result: x
```

**Why?** `ln(x)` is only defined for `x > 0`. The assumption guarantees we're in the domain.

### Example: Logarithm Product Rule

```rust
let prod = st.mul(vec![x, y]);
let ln_prod = st.func("ln", vec![prod]);

// With both symbols positive: ln(xy) → ln(x) + ln(y)
let mut ctx = Context::new();
ctx.assume("x", Prop::Positive);
ctx.assume("y", Prop::Positive);

let result = simplify_with(&mut st, ln_prod, &ctx);
// Result: ln(x) + ln(y)
```

## Property Hierarchy

Currently, properties are independent (no implications):
```rust
ctx.assume("x", Prop::Positive);
// Does NOT imply Real, Nonzero, etc.
```

**Future enhancement:** Add property implications:
- `Positive` → `Real` ∧ `Nonzero`
- `Integer` → `Real`
- etc.

## Design Rationale

### Why Tri-Valued Logic?

**Two-valued logic problems:**
- Cannot distinguish "known false" from "unknown"
- Closed-world assumption: absence of proof = false

**Tri-valued solution:**
- `True`: Explicitly known
- `False`: Explicitly known (not used currently)
- `Unknown`: No information

### Why Explicitly State Assumptions?

**Safety:** Prevents incorrect simplifications:
```rust
// WITHOUT assumptions:
// sqrt(x^2) → x  (WRONG if x < 0)

// WITH assumptions:
// Only applies if x is known positive
```

**Soundness:** Transformations are mathematically valid given constraints.

## Closed-World Assumption

Current implementation uses **open-world** for properties:
- Not assumed → Unknown (not False)
- Conservative: Only apply transformations when safe

**Future:** Could add negation:
```rust
ctx.assume_not("x", Prop::Positive);  // Explicitly x ≤ 0
```

## Integration with Other Modules

### simplify
Main consumer. Uses `Context::has()` to guard transformations:
```rust
fn is_positive_symbol(ctx: &Context, store: &Store, id: ExprId) -> bool {
    if let (Op::Symbol, Payload::Sym(s)) = (&store.get(id).op, &store.get(id).payload) {
        return matches!(ctx.has(s, Prop::Positive), Truth::True);
    }
    false
}
```

### Future: solver
Could use assumptions for domain constraints:
```rust
// Solve x^2 = 4 with x > 0
// Result: x = 2 (not x = -2)
```

### Future: calculus
Could use assumptions for integration bounds:
```rust
// ∫₀^∞ e^(-x) dx requires x real
```

## Example: Multiple Properties

```rust
use assumptions::{Context, Prop, Truth};

let mut ctx = Context::new();

// Assume n is an integer and positive
ctx.assume("n", Prop::Integer);
ctx.assume("n", Prop::Positive);

// Query
assert!(matches!(ctx.has("n", Prop::Integer), Truth::True));
assert!(matches!(ctx.has("n", Prop::Positive), Truth::True));
assert!(matches!(ctx.has("n", Prop::Real), Truth::Unknown));  // Not assumed
```

## Testing

Test coverage:
- Assumption and query roundtrips
- Multiple properties per symbol
- Unknown property queries
- Empty context behavior

Run tests:
```bash
cargo test -p assumptions
```

## Performance

- **Space**: O(s * p) where s = symbols, p = properties per symbol
- **assume()**: O(1) hash table insertion
- **has()**: O(1) hash table lookup

Typical usage: < 10 symbols, < 5 properties each → negligible overhead.

## Limitations

### No Property Implications

```rust
ctx.assume("x", Prop::Positive);
// Should imply Nonzero and Real, but doesn't currently
```

**Workaround:** Manually assume all implied properties.

### No Negation

Cannot express "x is NOT positive":
```rust
// No API for:
// ctx.assume_not("x", Prop::Positive);
```

### No Relationships

Cannot express relationships between symbols:
```rust
// Cannot say: "x < y"
// Cannot say: "a + b = c"
```

### No Numeric Bounds

Cannot express ranges:
```rust
// Cannot say: "0 < x < 1"
```

### No Complex Properties

Limited to simple predicates:
```rust
// Cannot say: "x is a prime number"
// Cannot say: "x is algebraic over Q"
```

## Future Enhancements

### Property Hierarchy
```rust
// Automatically derive implications
ctx.assume("x", Prop::Positive);
// Implicitly: x is Real, Nonzero
```

### Negation
```rust
ctx.assume_not("x", Prop::Positive);  // x ≤ 0
```

### Relationships
```rust
ctx.assume_less("x", "y");  // x < y
ctx.assume_equal("a + b", "c");  // a + b = c
```

### Numeric Ranges
```rust
ctx.assume_range("x", 0.0, 1.0);  // 0 < x < 1
```

### Inference
```rust
// Deduce new facts from assumptions
ctx.assume("x", Prop::Positive);
ctx.assume("y", Prop::Positive);
// Infer: x + y is Positive
```

### Contradiction Detection
```rust
ctx.assume("x", Prop::Positive);
ctx.assume("x", Prop::Zero);
// Detect: contradiction!
```

## Example: Workflow

```rust
use expr_core::Store;
use simplify::simplify_with;
use assumptions::{Context, Prop};

let mut st = Store::new();
let x = st.sym("x");
let y = st.sym("y");

// Set up assumptions
let mut ctx = Context::new();
ctx.assume("x", Prop::Positive);
ctx.assume("y", Prop::Positive);

// Build expression: ln(x * y)
let prod = st.mul(vec![x, y]);
let ln_prod = st.func("ln", vec![prod]);

println!("Before: {}", st.to_string(ln_prod));
// Output: ln(x * y)

// Simplify with assumptions
let result = simplify_with(&mut st, ln_prod, &ctx);
println!("After:  {}", st.to_string(result));
// Output: ln(x) + ln(y)
```

## References

- Depends on: None (stdlib only)
- Used by: `simplify`
- Related concepts:
  - Three-valued logic (Kleene, Łukasiewicz)
  - SMT solvers (Z3, CVC4)
  - Assumption systems in CAS (Mathematica's Assumptions, SymPy's Q)
