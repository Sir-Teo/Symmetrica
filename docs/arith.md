# arith - Arithmetic Utilities Module

## Overview

The `arith` crate provides shared small rational arithmetic utilities over `i64` integers. It implements normalized rational numbers with both tuple-based and newtype APIs, along with GCD computation.

## Core Type: Q

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Q(pub i64, pub i64);
```

A rational number represented as `Q(numerator, denominator)`.

**Invariants:**
- Denominator is always positive (`den > 0`)
- GCD of numerator and denominator is 1 (reduced form)
- Zero is represented as `Q(0, 1)`

## Construction

### Direct Construction
```rust
pub fn new(num: i64, den: i64) -> Self
```

Creates a normalized rational:
```rust
use arith::Q;

let q1 = Q::new(6, 9);
assert_eq!(q1, Q(2, 3));  // Automatically reduced

let q2 = Q::new(3, -4);
assert_eq!(q2, Q(-3, 4));  // Denominator made positive
```

### Constants
```rust
pub fn zero() -> Self  // Q(0, 1)
pub fn one() -> Self   // Q(1, 1)
```

### Predicates
```rust
pub fn is_zero(&self) -> bool
```

## GCD Computation

```rust
pub fn gcd_i64(a: i64, b: i64) -> i64
```

Computes the greatest common divisor using Euclid's algorithm.

**Properties:**
- Always returns non-negative result
- `gcd(0, n) = |n|`
- `gcd(a, b) = gcd(|a|, |b|)`

**Examples:**
```rust
use arith::gcd_i64;

assert_eq!(gcd_i64(54, 24), 6);
assert_eq!(gcd_i64(-12, 8), 4);
assert_eq!(gcd_i64(0, 5), 5);
```

## Normalization

```rust
pub fn normalize_rat(num: i64, den: i64) -> (i64, i64)
```

Normalizes a rational to reduced form:
1. Panics if denominator is zero
2. Makes denominator positive
3. Divides both by their GCD
4. Special case: `0/n → (0, 1)`

**Examples:**
```rust
use arith::normalize_rat;

assert_eq!(normalize_rat(4, 6), (2, 3));
assert_eq!(normalize_rat(3, -4), (-3, 4));
assert_eq!(normalize_rat(0, 5), (0, 1));

// Panics:
// normalize_rat(1, 0);  // zero denominator
```

## Tuple-Based Arithmetic

All functions operate on `(i64, i64)` tuples representing `(numerator, denominator)`.

### Addition
```rust
pub fn rat_add(a: (i64, i64), b: (i64, i64)) -> (i64, i64)
pub fn q_add(a: (i64, i64), b: (i64, i64)) -> (i64, i64)  // Alias
```

Computes `a + b`:
```rust
let result = rat_add((1, 2), (1, 3));
assert_eq!(result, (5, 6));  // 1/2 + 1/3 = 5/6
```

### Subtraction
```rust
pub fn rat_sub(a: (i64, i64), b: (i64, i64)) -> (i64, i64)
pub fn q_sub(a: (i64, i64), b: (i64, i64)) -> (i64, i64)  // Alias
```

Computes `a - b`:
```rust
let result = rat_sub((1, 2), (1, 3));
assert_eq!(result, (1, 6));  // 1/2 - 1/3 = 1/6
```

### Multiplication
```rust
pub fn rat_mul(a: (i64, i64), b: (i64, i64)) -> (i64, i64)
pub fn q_mul(a: (i64, i64), b: (i64, i64)) -> (i64, i64)  // Alias
```

Computes `a * b`:
```rust
let result = rat_mul((2, 3), (3, 4));
assert_eq!(result, (1, 2));  // 2/3 * 3/4 = 1/2
```

### Division
```rust
pub fn q_div(a: (i64, i64), b: (i64, i64)) -> (i64, i64)
```

Computes `a / b`:
```rust
let result = q_div((1, 2), (1, 4));
assert_eq!(result, (2, 1));  // (1/2) / (1/4) = 2
```

## Q-Based Arithmetic

Functions operating directly on `Q` type:

```rust
pub fn add_q(a: Q, b: Q) -> Q
pub fn sub_q(a: Q, b: Q) -> Q
pub fn mul_q(a: Q, b: Q) -> Q
pub fn div_q(a: Q, b: Q) -> Q
```

**Examples:**
```rust
use arith::{Q, add_q, mul_q};

let a = Q(1, 2);
let b = Q(1, 3);

let sum = add_q(a, b);
assert_eq!(sum, Q(5, 6));

let prod = mul_q(a, b);
assert_eq!(prod, Q(1, 6));
```

## API Aliases

For ergonomic use, both naming conventions are provided:
- **rat_* / q_norm**: Tuple-based functions
- **q_* / Q**: Newtype-based functions
- **add_q / mul_q / etc.**: Q-type operations

**Rationale:**
- `calculus` crate prefers `q_*` naming
- `polys` crate uses `Q` extensively
- Both styles coexist for compatibility

## Usage in Other Crates

### expr_core
Uses `normalize_rat`, `rat_add`, `rat_mul` for rational arithmetic in canonical constructors:
```rust
let (n, d) = normalize_rat(num, den);
if d == 1 {
    return self.int(n);  // Store as integer
}
self.intern(Op::Rational, Payload::Rat(n, d), vec![])
```

### polys
Uses `Q` extensively for polynomial coefficients:
```rust
pub struct UniPoly {
    pub coeffs: Vec<Q>,  // Each coefficient is a Q
}
```

### simplify
Uses `rat_add` and `rat_mul` for collecting like terms:
```rust
let mut coeff = (1i64, 1i64);
coeff = rat_mul(coeff, (*n, *d));
```

### matrix
Uses `Q` for exact linear algebra over rationals:
```rust
pub struct MatrixQ {
    pub data: Vec<Q>,
}
```

## Overflow Behavior

**Warning:** All operations use `i64` arithmetic. Overflow will panic in debug mode or wrap in release mode.

**Best practices:**
- Use for small rationals (denominators < 10^6)
- For large problems, consider arbitrary-precision libraries (e.g., `num-bigint`)

**Overflow example:**
```rust
// Large intermediate values may overflow:
let a = Q(i64::MAX / 2, 1);
let b = Q(i64::MAX / 2, 1);
// add_q(a, b) will overflow!
```

## Testing

Comprehensive unit tests cover:
- GCD with various inputs (zero, negative, etc.)
- Normalization (sign, reduction, zero handling)
- All arithmetic operations
- Q struct methods
- Edge cases

Run tests:
```bash
cargo test -p arith
```

## Design Rationale

### Why i64?
- Fast arithmetic on modern CPUs
- Sufficient precision for most symbolic computation
- Simple implementation without external dependencies

### Why Normalized?
- Canonical representation enables structural equality
- Prevents growth of numerators/denominators
- Compatible with `expr_core`'s hash-consing

### Why Both APIs?
- Tuple API is lightweight for internal use
- Q newtype provides type safety for `polys` and `matrix`
- Aliases offer ergonomic flexibility

## Performance

- **GCD**: O(log(min(a, b))) using Euclidean algorithm
- **Arithmetic**: O(1) for operations, O(log) for normalization (GCD)
- **Memory**: 16 bytes per Q (two i64s)

## Limitations

- **No arbitrary precision**: Limited to i64 range
- **Overflow**: Not detected in release mode
- **No floating point**: Only exact rationals

## Example: Rational Arithmetic

```rust
use arith::{Q, add_q, mul_q, div_q};

// Compute (1/2 + 1/3) * (2/5)
let a = Q(1, 2);
let b = Q(1, 3);
let c = Q(2, 5);

let sum = add_q(a, b);     // 5/6
let product = mul_q(sum, c); // (5/6) * (2/5) = 1/3

println!("{}/{}", product.0, product.1);  // Output: 1/3
```

## Example: Building a Matrix

```rust
use arith::Q;

// 2x2 identity matrix
let data = vec![
    Q(1, 1), Q(0, 1),
    Q(0, 1), Q(1, 1),
];

// Access element
let element = data[0];
assert_eq!(element, Q::one());
```

## Future Enhancements

- **Arbitrary precision**: Integration with `num-bigint`
- **Overflow detection**: Safe arithmetic wrapper
- **Performance**: SIMD operations for bulk computations
- **Algebraic numbers**: Support for Q[√2], Q[i], etc.

## References

- Depends on: None (stdlib only)
- Used by: `expr_core`, `simplify`, `polys`, `calculus`, `matrix`, `solver`
- Classic reference: Knuth TAOCP Vol. 2, Section 4.5 (Rational Arithmetic)
