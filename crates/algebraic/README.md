# algebraic

Algebraic numbers for Symmetrica: quadratic extensions Q(√d) and arithmetic.

## Features

- **Quadratic extensions:** Represent elements `a + b√d` where `a, b ∈ Q` and `d` is an integer
- **Operations:** `Add`, `Sub`, `Neg`, `Mul` implemented
- **Methods:**
  - `new(a, b, d)` - construct element
  - `from_int(n, d)` - embed integer into extension
  - `conj()` - conjugate (negates √d coefficient)
  - `norm()` - field norm `a² - b²d`

## Examples

### Basic arithmetic

```rust
use algebraic::Quad;
use arith::Q;

// Elements in Q(√5)
let x = Quad::new(Q::new(1, 2), Q::new(1, 3), 5); // 1/2 + (1/3)√5
let y = Quad::new(Q::new(2, 3), Q::new(1, 6), 5); // 2/3 + (1/6)√5

// Addition
let sum = x + y;

// Multiplication
let prod = x * y;

// Negation
let neg_x = -x;
```

### Conjugate and norm

```rust
use algebraic::Quad;
use arith::Q;

let a = Quad::new(Q::new(3, 4), Q::new(2, 5), 3); // 3/4 + (2/5)√3
let c = a.conj(); // 3/4 - (2/5)√3

// Field norm: N(a) = a * conj(a) = a² - b²d
let n = a.norm();

// Verify: a * conj(a) = norm(a) (as element of Q)
let prod = a * c;
assert_eq!(prod.b, Q::zero());
assert_eq!(prod.a, n);
```

### Embedding integers

```rust
use algebraic::Quad;

// Embed 7 into Q(√2)
let x = Quad::from_int(7, 2);
assert_eq!(x.a, arith::Q::new(7, 1));
assert_eq!(x.b, arith::Q::zero());
```

## Notes

- All elements must belong to the same extension (same `d` value)
- Operations panic if extensions don't match
- Rationals `a, b` are stored as `arith::Q` (normalized i64 fractions)
- No squarefree enforcement on `d` (user responsibility)

## Future work

- Rationalization and inverse (when norm ≠ 0)
- Higher degree extensions (cyclotomic fields)
- Minimal polynomial computation
- Integration with `simplify/` for exact symbolic roots

## License

Dual-licensed under MIT or Apache-2.0.
