# solver - Equation Solver Module

## Overview

The `solver` crate provides symbolic equation solving for univariate polynomial equations over rational numbers. It uses polynomial factorization from the `polys` crate to find all rational roots.

**Algorithm:**
1. Convert expression to univariate polynomial
2. Factor polynomial into irreducible factors over Q
3. Extract roots from linear and quadratic factors
4. Return None if higher-degree irreducible factors remain

## Core Function

```rust
pub fn solve_univariate(store: &mut Store, expr: ExprId, var: &str) -> Option<Vec<ExprId>>
```

Solves the equation `expr = 0` for variable `var` using polynomial factorization.

**Returns:**
- `Some(Vec<ExprId>)`: List of all rational roots (with multiplicity)
- `None`: Cannot solve (not a polynomial, or has irreducible factors of degree > 2)

## Supported Equation Types

### Linear Equations (Degree 1)

**Form:** `ax + b = 0`

**Solution:** `x = -b/a`

**Example:**
```rust
use expr_core::Store;
use solver::solve_univariate;

let mut st = Store::new();
let x = st.sym("x");

// Solve: 2x + 3 = 0
let eq = st.add(vec![st.mul(vec![st.int(2), x]), st.int(3)]);
let roots = solve_univariate(&mut st, eq, "x").unwrap();
// Result: x = -3/2
assert_eq!(st.to_string(roots[0]), "-3/2");
```

### Quadratic Equations (Degree 2)

**Form:** `ax² + bx + c = 0`

**Discriminant:** `Δ = b² - 4ac`

**Solutions:**
```
x = (-b ± √Δ) / (2a)
```

#### Case 1: Rational Roots (Δ is a perfect square)
```rust
let mut st = Store::new();
let x = st.sym("x");

// Solve: x^2 + 3x + 2 = 0
// Factors: (x+1)(x+2) = 0
let eq = st.add(vec![
    st.pow(x, st.int(2)),
    st.mul(vec![st.int(3), x]),
    st.int(2),
]);

let roots = solve_univariate(&mut st, eq, "x").unwrap();
// Result: x = -1, x = -2
```

#### Case 2: Irrational Roots (Δ not a perfect square)
```rust
// Solve: x^2 - 2 = 0
let eq = st.add(vec![st.pow(x, st.int(2)), st.int(-2)]);
let roots = solve_univariate(&mut st, eq, "x").unwrap();
// Result: x = √2, x = -√2 (represented as powers)
```

The solver builds symbolic expressions using fractional exponents:
```rust
// √Δ is represented as Δ^(1/2)
```

### Higher Degree Polynomials (Degree ≥ 3)

Uses **Rational Root Theorem** + **deflation** strategy:

#### Algorithm

1. **Find one rational root** using RRT candidates: `p/q` where
   - `p` divides the constant term
   - `q` divides the leading coefficient

2. **Deflate**: Divide polynomial by `(x - root)` to reduce degree

3. **Repeat** until degree ≤ 2

4. **Finish** with quadratic or linear solver

**Example:**
```rust
// Solve: x^3 - x = 0
// Factors: x(x-1)(x+1) = 0
let eq = st.add(vec![
    st.pow(x, st.int(3)),
    st.mul(vec![st.int(-1), x]),
]);

let roots = solve_univariate(&mut st, eq, "x").unwrap();
// Result: x = 0, x = 1, x = -1
```

**Limitation:** Returns `None` if no rational roots exist:
```rust
// x^3 + x + 1 = 0 has only irrational/complex roots
let eq = st.add(vec![st.pow(x, st.int(3)), x, st.int(1)]);
assert!(solve_univariate(&mut st, eq, "x").is_none());
```

## Rational Root Theorem

For polynomial `aₙxⁿ + ... + a₁x + a₀ = 0` with integer coefficients, any rational root `p/q` (in lowest terms) satisfies:
- `p` divides `a₀` (constant term)
- `q` divides `aₙ` (leading coefficient)

### Implementation

```rust
fn clear_denominators(p: &UniPoly) -> (Vec<i64>, i64)
```
Converts rational coefficients to integers by multiplying through by LCM of denominators.

```rust
fn divisors(n: i64) -> Vec<i64>
```
Finds all positive divisors of `n` (and their negatives).

```rust
fn eval_q(p: &UniPoly, x: Q) -> Q
```
Evaluates polynomial at rational point using Horner's method.

```rust
fn deflate_by_root(p: &UniPoly, r: Q) -> Option<UniPoly>
```
Divides polynomial by `(x - r)` using synthetic division. Returns `None` if `r` is not a root.

## Special Cases

### Zero Polynomial
```rust
let zero = st.int(0);
let roots = solve_univariate(&mut st, zero, "x").unwrap();
// Result: empty vector (degenerate, infinitely many solutions)
assert!(roots.is_empty());
```

### Constant Non-Zero
```rust
let five = st.int(5);
let roots = solve_univariate(&mut st, five, "x").unwrap();
// Result: empty vector (no solutions)
assert!(roots.is_empty());
```

### Not a Polynomial
```rust
let sin_x = st.func("sin", vec![x]);
let roots = solve_univariate(&mut st, sin_x, "x");
// Result: None (not a polynomial)
assert!(roots.is_none());
```

## Root Representation

### Rational Roots
```rust
// Stored as Integer or Rational nodes
x = 3     // Integer
x = -2/3  // Rational
```

### Irrational Roots
```rust
// Stored as symbolic expressions with Pow nodes
x = 2^(1/2)           // √2
x = (-1 + 5^(1/2))/2  // Golden ratio
```

## Example Workflows

### Solving Quadratic with Discriminant Check

```rust
let mut st = Store::new();
let x = st.sym("x");

// x^2 - 5x + 6 = 0
let eq = st.add(vec![
    st.pow(x, st.int(2)),
    st.mul(vec![st.int(-5), x]),
    st.int(6),
]);

let roots = solve_univariate(&mut st, eq, "x").unwrap();
for root in roots {
    println!("x = {}", st.to_string(root));
}
// Output: x = 2, x = 3
```

### Solving Cubic

```rust
// x^3 - 6x^2 + 11x - 6 = 0
// Factors: (x-1)(x-2)(x-3) = 0
let eq = st.add(vec![
    st.pow(x, st.int(3)),
    st.mul(vec![st.int(-6), st.pow(x, st.int(2))]),
    st.mul(vec![st.int(11), x]),
    st.int(-6),
]);

let roots = solve_univariate(&mut st, eq, "x").unwrap();
// Result: x = 1, x = 2, x = 3
```

### Handling Irrational Roots

```rust
// x^2 - 2 = 0
let eq = st.add(vec![st.pow(x, st.int(2)), st.int(-2)]);
let roots = solve_univariate(&mut st, eq, "x").unwrap();

for root in &roots {
    println!("{}", st.to_string(*root));
}
// Output includes expressions like "(2)^(1/2)"
```

## Integration with Other Modules

### polys
Uses `expr_to_unipoly` to convert equations to polynomial form:
```rust
let p = expr_to_unipoly(store, expr, var)?;
```

### expr_core
Builds root expressions using canonical constructors.

### simplify
Results are NOT automatically simplified. For canonical output:
```rust
let roots = solve_univariate(&mut st, expr, "x")?;
let simplified_roots: Vec<_> = roots.into_iter()
    .map(|r| simplify::simplify(&mut st, r))
    .collect();
```

## Performance

- **Linear**: O(1) arithmetic operations
- **Quadratic**: O(1) with discriminant check
- **Higher degree**: O(d² × k) where d = degree, k = number of RRT candidates

**Worst case:** Factorials grow quickly for high-degree polynomials with large coefficients.

## Limitations

### No Complex Roots
Only real roots are returned. Complex roots are not supported:
```rust
// x^2 + 1 = 0 has roots ±i (not found)
let eq = st.add(vec![st.pow(x, st.int(2)), st.int(1)]);
assert!(solve_univariate(&mut st, eq, "x").unwrap().is_empty());
```

### No Algebraic Extensions
Cannot represent roots like `∛2` (cube root):
```rust
// x^3 - 2 = 0 has root ∛2 (not rational, returns None)
```

### No Numerical Approximation
All solutions are exact symbolic expressions. No floating-point approximations.

### No Multiplicity Tracking
Repeated roots are returned multiple times:
```rust
// (x-1)^2 = 0 → x = 1, x = 1 (twice)
```

## Testing

Comprehensive test coverage:
- Linear equations
- Quadratic with rational roots
- Quadratic with irrational roots
- Cubic with all rational roots
- Zero polynomial (empty result)
- Constant polynomial (empty result)
- Non-polynomial expressions (None)
- No rational roots case (None)

Run tests:
```bash
cargo test -p solver
```

## Future Enhancements

- **Cubic/Quartic formulas**: Cardano's formula, Ferrari's method
- **Complex roots**: Support for Gaussian rationals Q[i]
- **Algebraic numbers**: Represent roots symbolically (e.g., Root objects)
- **Numerical solving**: Newton-Raphson for high-degree polynomials
- **Multiplicity**: Track root multiplicities
- **Systems of equations**: Multivariate polynomial systems
- **Inequalities**: Solve polynomial inequalities

## Example: Complete Workflow

```rust
use expr_core::Store;
use solver::solve_univariate;
use simplify::simplify;

let mut st = Store::new();
let x = st.sym("x");

// Build equation: x^2 - 5x + 6 = 0
let x2 = st.pow(x, st.int(2));
let five_x = st.mul(vec![st.int(-5), x]);
let eq = st.add(vec![x2, five_x, st.int(6)]);

println!("Solving: {} = 0", st.to_string(eq));

match solve_univariate(&mut st, eq, "x") {
    Some(roots) => {
        println!("Found {} roots:", roots.len());
        for (i, root) in roots.iter().enumerate() {
            let simplified = simplify(&mut st, *root);
            println!("  x_{} = {}", i+1, st.to_string(simplified));
        }
    }
    None => {
        println!("Cannot solve (no rational roots or not a polynomial)");
    }
}

// Output:
// Solving: 6 - 5 * x + x^2 = 0
// Found 2 roots:
//   x_1 = 2
//   x_2 = 3
```

## References

- Depends on: `expr_core`, `polys`, `arith`
- Used by: `cli`
- Classic algorithms: Rational Root Theorem, synthetic division
- Related: Galois theory, numerical root finding (not implemented)
