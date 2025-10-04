# polys - Polynomial Module

## Overview

The `polys` crate provides univariate dense polynomial types over rational numbers (Q), along with division, GCD, and bidirectional conversions between expressions and polynomials. It also includes partial fraction decomposition for rational functions.

## Core Type: UniPoly

```rust
pub struct UniPoly {
    pub var: String,
    pub coeffs: Vec<Q>,  // coeffs[k] = coefficient of x^k
}
```

**Properties:**
- Dense representation (all degrees from 0 to max are stored)
- No trailing zeros (highest non-zero coefficient is last)
- Coefficients are normalized rationals from the `arith` crate

## Construction

```rust
pub fn new<S: Into<String>>(var: S, coeffs: Vec<Q>) -> Self
```

Creates a polynomial, automatically trimming trailing zeros.

```rust
pub fn zero<S: Into<String>>(var: S) -> Self
```

Creates the zero polynomial.

### Examples

```rust
use polys::UniPoly;
use arith::Q;

// p(x) = 2 + 3x + x^2
let p = UniPoly::new("x", vec![Q(2, 1), Q(3, 1), Q(1, 1)]);

// Zero polynomial
let z = UniPoly::zero("x");
assert!(z.is_zero());
```

## Basic Operations

### Degree and Leading Coefficient

```rust
pub fn degree(&self) -> Option<usize>  // None for zero polynomial
pub fn leading_coeff(&self) -> Q
pub fn is_zero(&self) -> bool
```

**Examples:**
```rust
let p = UniPoly::new("x", vec![Q(2, 1), Q(3, 1), Q(1, 1)]);
assert_eq!(p.degree(), Some(2));
assert_eq!(p.leading_coeff(), Q(1, 1));
```

### Arithmetic

```rust
pub fn add(&self, rhs: &Self) -> Self
pub fn sub(&self, rhs: &Self) -> Self
pub fn mul(&self, rhs: &Self) -> Self
```

**Precondition:** Both polynomials must have the same variable.

**Examples:**
```rust
let p1 = UniPoly::new("x", vec![Q(1, 1), Q(2, 1)]);  // 1 + 2x
let p2 = UniPoly::new("x", vec![Q(3, 1), Q(4, 1)]);  // 3 + 4x

let sum = p1.add(&p2);    // 4 + 6x
let diff = p1.sub(&p2);   // -2 - 2x
let prod = p1.mul(&p2);   // 3 + 10x + 8x^2
```

### Derivative

```rust
pub fn deriv(&self) -> Self
```

Symbolic derivative with respect to the polynomial's variable:
```rust
let p = UniPoly::new("x", vec![Q(2, 1), Q(3, 1), Q(1, 1)]);  // 2 + 3x + x^2
let dp = p.deriv();  // 3 + 2x
```

### Evaluation

```rust
pub fn eval_q(&self, x: Q) -> Q
```

Evaluates the polynomial at a rational point using Horner's method:
```rust
let p = UniPoly::new("x", vec![Q(1, 1), Q(2, 1), Q(1, 1)]);  // 1 + 2x + x^2
let val = p.eval_q(Q(2, 1));  // 1 + 4 + 4 = 9
assert_eq!(val, Q(9, 1));
```

### Monic Polynomial

```rust
pub fn monic(&self) -> Self
```

Divides all coefficients by the leading coefficient to make it 1:
```rust
let p = UniPoly::new("x", vec![Q(2, 1), Q(4, 1)]);  // 2 + 4x
let m = p.monic();  // (1/2) + x
```

## Division and GCD

### Division with Remainder

```rust
pub fn div_rem(&self, div: &Self) -> Result<(Self, Self), &'static str>
```

Returns `(quotient, remainder)` such that `self = quotient * div + remainder` with `deg(remainder) < deg(div)`.

**Example:**
```rust
let dividend = UniPoly::new("x", vec![Q(2, 1), Q(3, 1), Q(1, 1)]);  // 2 + 3x + x^2
let divisor = UniPoly::new("x", vec![Q(1, 1), Q(1, 1)]);            // 1 + x

let (q, r) = dividend.div_rem(&divisor).unwrap();
// q = 2 + x,  r = 0
assert!(r.is_zero());
```

### Euclidean GCD

```rust
pub fn gcd(a: Self, b: Self) -> Self
```

Computes the greatest common divisor using the Euclidean algorithm. Result is monic.

**Example:**
```rust
let p1 = UniPoly::new("x", vec![Q(-1, 1), Q(0, 1), Q(1, 1)]);  // x^2 - 1
let p2 = UniPoly::new("x", vec![Q(0, 1), Q(-1, 1), Q(1, 1)]);  // x^2 - x

let g = UniPoly::gcd(p1, p2);
// Result: x - 1 (monic)
```

## Expression Conversions

### Expr → UniPoly

```rust
pub fn expr_to_unipoly(store: &Store, id: ExprId, var: &str) -> Option<UniPoly>
```

Converts an expression to a univariate polynomial if it consists of:
- Sum of monomials in `var`
- Non-negative integer exponents
- Rational coefficients

Returns `None` if the expression is not a valid polynomial.

**Examples:**
```rust
use expr_core::Store;

let mut st = Store::new();
let x = st.sym("x");
let expr = st.add(vec![
    st.pow(x, st.int(2)),
    st.mul(vec![st.int(3), x]),
    st.int(2),
]);  // 2 + 3x + x^2

let poly = expr_to_unipoly(&st, expr, "x").unwrap();
assert_eq!(poly.coeffs, vec![Q(2, 1), Q(3, 1), Q(1, 1)]);
```

**Non-polynomial examples (return None):**
```rust
// Negative exponent
let inv_x = st.pow(x, st.int(-1));
assert!(expr_to_unipoly(&st, inv_x, "x").is_none());

// Function
let sin_x = st.func("sin", vec![x]);
assert!(expr_to_unipoly(&st, sin_x, "x").is_none());

// Wrong variable
let y = st.sym("y");
assert!(expr_to_unipoly(&st, y, "x").is_none());
```

### UniPoly → Expr

```rust
pub fn unipoly_to_expr(store: &mut Store, p: &UniPoly) -> ExprId
```

Converts a polynomial back to an expression:
```rust
let poly = UniPoly::new("x", vec![Q(2, 1), Q(3, 1), Q(1, 1)]);
let expr = unipoly_to_expr(&mut st, &poly);
// Result: 2 + 3*x + x^2
```

### Roundtrip Property

For valid polynomial expressions:
```rust
let expr = /* polynomial expression */;
let poly = expr_to_unipoly(&st, expr, "x").unwrap();
let back = unipoly_to_expr(&mut st, &poly);
assert_eq!(st.get(expr).digest, st.get(back).digest);
```

## Partial Fractions

```rust
pub fn partial_fractions_simple(num: &UniPoly, den: &UniPoly) 
    -> Option<(UniPoly, Vec<(Q, Q)>)>
```

Decomposes a rational function `num/den` into:
- **Polynomial part**: Result of long division
- **Terms**: List of `(A_i, r_i)` representing `A_i/(x - r_i)`

**Requirements:**
- Denominator must factor into **distinct** linear factors over Q
- Uses Rational Root Theorem to find roots
- Computes residues via derivative evaluation

**Result:**
```
num/den = quotient + Σ (A_i / (x - r_i))
```

### Algorithm

1. **Long division**: Extract polynomial quotient
2. **Find rational roots**: Use divisors of constant and leading coefficients
3. **Deflate**: Remove each root to ensure no repeated factors
4. **Check distinctness**: Verify `den'(root) ≠ 0` for all roots
5. **Compute residues**: `A_i = remainder(r_i) / den'(r_i)`

### Examples

**Simple case:**
```rust
// (2x + 3) / (x^2 + 3x + 2)
// Denominator factors: (x+1)(x+2)
let num = UniPoly::new("x", vec![Q(3, 1), Q(2, 1)]);
let den = UniPoly::new("x", vec![Q(2, 1), Q(3, 1), Q(1, 1)]);

let (q, terms) = partial_fractions_simple(&num, &den).unwrap();
assert!(q.is_zero());  // No polynomial part
assert_eq!(terms.len(), 2);
// terms: [(1, -1), (1, -2)]
// Meaning: 1/(x+1) + 1/(x+2)
```

**Improper fraction:**
```rust
// x^3 / (x+1) = x^2 - x + 1 - 1/(x+1)
let num = UniPoly::new("x", vec![Q(0,1), Q(0,1), Q(0,1), Q(1,1)]);
let den = UniPoly::new("x", vec![Q(1,1), Q(1,1)]);

let (q, terms) = partial_fractions_simple(&num, &den).unwrap();
assert_eq!(q.degree(), Some(2));  // Quotient is x^2 - x + 1
assert_eq!(terms.len(), 1);
```

**Repeated roots (returns None):**
```rust
// (x+1) / (x+1)^2 has repeated root at -1
let num = UniPoly::new("x", vec![Q(1,1), Q(1,1)]);
let den = UniPoly::new("x", vec![Q(1,1), Q(2,1), Q(1,1)]);  // (x+1)^2
assert!(partial_fractions_simple(&num, &den).is_none());
```

**No rational roots (returns None):**
```rust
// 1 / (x^2 + 1) has complex roots
let num = UniPoly::new("x", vec![Q(1,1)]);
let den = UniPoly::new("x", vec![Q(1,1), Q(0,1), Q(1,1)]);
assert!(partial_fractions_simple(&num, &den).is_none());
```

## Integration with Other Crates

### calculus
Uses partial fractions for integrating rational functions:
```rust
// ∫ (2x+3)/(x^2+3x+2) dx
// → ln(x+1) + ln(x+2)
```

### solver
Converts equations to polynomials for root finding.

### expr_core
Bidirectional conversion for seamless integration with expression trees.

## Performance

- **Addition/Subtraction**: O(max(deg(p), deg(q)))
- **Multiplication**: O(deg(p) × deg(q))
- **Division**: O(deg(dividend) × deg(divisor))
- **GCD**: O(deg²) worst case (Euclidean algorithm)
- **Partial fractions**: O(deg³) worst case (root finding + deflation)

## Limitations

- **Univariate only**: No multivariate polynomial support
- **Dense representation**: Inefficient for sparse polynomials
- **Rational coefficients**: No algebraic extensions (e.g., Q[√2])
- **Partial fractions**: Limited to distinct linear factors over Q

## Testing

Comprehensive test suite:
- Arithmetic operations (add, sub, mul)
- Division with remainder
- GCD computation
- Expression conversion roundtrips
- Partial fractions (simple, improper, edge cases)
- Derivative and evaluation

Run tests:
```bash
cargo test -p polys
```

## Future Enhancements

- Sparse polynomial representation
- Multivariate polynomials
- Factorization over Q (square-free, complete)
- Gröbner bases
- Resultants and discriminants
- Support for algebraic number fields

## Example: Polynomial Algebra

```rust
use polys::UniPoly;
use arith::Q;

let p = UniPoly::new("x", vec![Q(1, 1), Q(2, 1), Q(1, 1)]);  // 1 + 2x + x^2
let q = UniPoly::new("x", vec![Q(-1, 1), Q(1, 1)]);          // -1 + x

// Arithmetic
let sum = p.add(&q);        // 2x + x^2
let product = p.mul(&q);    // -1 - x + x^3

// Division
let (quot, rem) = p.div_rem(&q).unwrap();
// quot = 3 + x, rem = 4

// GCD
let g = UniPoly::gcd(p.clone(), q.clone());
// g = x - 1 (after normalization)

// Derivative
let dp = p.deriv();  // 2 + 2x

// Evaluation
let val = p.eval_q(Q(2, 1));  // 1 + 4 + 4 = 9
```

## References

- Depends on: `arith`, `expr_core`
- Used by: `calculus`, `solver`
- Classic references: 
  - Knuth TAOCP Vol. 2 (Seminumerical Algorithms)
  - Modern Computer Algebra (von zur Gathen & Gerhard)
