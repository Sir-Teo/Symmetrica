# Phase 5: Symbolic Summation Examples

Complete examples demonstrating v1.4 summation features.

## Arithmetic Series

### Basic Summation

```rust
use expr_core::Store;
use summation::sum_arithmetic;

fn main() {
    let mut st = Store::new();
    let k = st.sym("k");
    let n = st.sym("n");
    let one = st.int(1);
    
    // ∑(k=1 to n) k = n(n+1)/2
    let result = sum_arithmetic(&mut st, k, one, n, one, one)
        .expect("arithmetic sum");
    
    println!("{}", st.to_string(result));
    // Output: 1/2 * n * (1 + n)
}
```

### General Arithmetic Series

```rust
use expr_core::Store;
use summation::sum_arithmetic;

fn main() {
    let mut st = Store::new();
    let k = st.sym("k");
    let n = st.sym("n");
    let one = st.int(1);
    let three = st.int(3);
    let two = st.int(2);
    
    // ∑(k=1 to n) (3 + 2k) = arithmetic series with a=3, d=2
    let result = sum_arithmetic(&mut st, k, one, n, three, two)
        .expect("arithmetic sum");
    
    println!("{}", st.to_string(result));
}
```

## Geometric Series

### Finite Geometric Series

```rust
use expr_core::Store;
use summation::sum_geometric;

fn main() {
    let mut st = Store::new();
    let k = st.sym("k");
    let n = st.sym("n");
    let zero = st.int(0);
    let two = st.int(2);
    
    // ∑(k=0 to n) 2^k = (2^(n+1) - 1)/(2 - 1)
    let term = st.pow(two, k);
    let result = sum_geometric(&mut st, term, "k", zero, n, two)
        .expect("geometric sum");
    
    println!("{}", st.to_string(result));
    // Output: (-1) + 2^(1 + n)
}
```

### General Geometric Series

```rust
use expr_core::Store;
use summation::sum_geometric;

fn main() {
    let mut st = Store::new();
    let k = st.sym("k");
    let n = st.sym("n");
    let one = st.int(1);
    let half = st.rat(1, 2);
    
    // ∑(k=1 to n) (1/2)^k
    let term = st.pow(half, k);
    let result = sum_geometric(&mut st, term, "k", one, n, half)
        .expect("geometric sum");
    
    println!("{}", st.to_string(result));
}
```

## Power Sums

```rust
use expr_core::Store;
use summation::sum_power;

fn main() {
    let mut st = Store::new();
    let k = st.sym("k");
    let n = st.sym("n");
    let one = st.int(1);
    let two = st.int(2);
    
    // ∑(k=1 to n) k² = n(n+1)(2n+1)/6
    let result = sum_power(&mut st, "k", two, one, n)
        .expect("power sum");
    
    println!("{}", st.to_string(result));
}
```

## Gosper's Algorithm

For hypergeometric terms (ratios of factorials, binomials, etc.):

```rust
use expr_core::Store;
use summation::gosper_sum;

fn main() {
    let mut st = Store::new();
    let k = st.sym("k");
    let n = st.sym("n");
    let one = st.int(1);
    
    // Gosper's algorithm for hypergeometric sums
    // Example: ∑ binomial(n, k) * k
    let n_sym = st.sym("n");
    let binom = st.func("binom", vec![n_sym, k]);
    let term = st.mul(vec![binom, k]);
    
    if let Some(result) = gosper_sum(&mut st, term, "k", one, n) {
        println!("{}", st.to_string(result));
    }
}
```

## Zeilberger's Algorithm

Generates recurrence relations when closed form is not available:

```rust
use expr_core::Store;
use summation::zeilberger_recurrence;

fn main() {
    let mut st = Store::new();
    let n = st.sym("n");
    let k = st.sym("k");
    
    // Example: binomial(n, k) generates Pascal's identity
    let binom = st.func("binom", vec![n, k]);
    
    if let Some(cert) = zeilberger_recurrence(&mut st, binom, "k", "n") {
        println!("Recurrence coefficients: {:?}", cert.coefficients);
    }
}
```

## Infinite Products

### Factorial Products

```rust
use expr_core::Store;
use summation::evaluate_finite_product;

fn main() {
    let mut st = Store::new();
    let k = st.sym("k");
    let one = st.int(1);
    let five = st.int(5);
    
    // ∏(k=1 to 5) k = 5! = 120
    let result = evaluate_finite_product(&mut st, k, "k", one, five)
        .expect("factorial product");
    
    println!("{}", st.to_string(result));
    // Output: 120
}
```

### Geometric Products

```rust
use expr_core::Store;
use summation::evaluate_finite_product;

fn main() {
    let mut st = Store::new();
    let k = st.sym("k");
    let n = st.sym("n");
    let two = st.int(2);
    let zero = st.int(0);
    
    // ∏(k=0 to n) 2^k = 2^(n(n+1)/2)
    let term = st.pow(two, k);
    let result = evaluate_finite_product(&mut st, term, "k", zero, n)
        .expect("geometric product");
    
    println!("{}", st.to_string(result));
}
```

### Gamma Function Connection

```rust
use expr_core::Store;
use summation::product_to_gamma_ratio;

fn main() {
    let mut st = Store::new();
    let x = st.sym("x");
    let n = st.sym("n");
    
    // ∏(k=0 to n-1) (x+k) = Γ(x+n)/Γ(x)
    let result = product_to_gamma_ratio(&mut st, x, n)
        .expect("gamma ratio");
    
    println!("{}", st.to_string(result));
    // Output: gamma(x + n) * gamma(x)^(-1)
}
```

## Pochhammer Symbols

### Rising Factorial

```rust
use expr_core::Store;
use summation::rising_factorial;

fn main() {
    let mut st = Store::new();
    let x = st.sym("x");
    let four = st.int(4);
    
    // (x)₄ = x(x+1)(x+2)(x+3)
    let result = rising_factorial(&mut st, x, four)
        .expect("rising factorial");
    
    println!("{}", st.to_string(result));
}
```

### Falling Factorial

```rust
use expr_core::Store;
use summation::falling_factorial;

fn main() {
    let mut st = Store::new();
    let x = st.sym("x");
    let four = st.int(4);
    
    // x^(4) = x(x-1)(x-2)(x-3)
    let result = falling_factorial(&mut st, x, four)
        .expect("falling factorial");
    
    println!("{}", st.to_string(result));
}
```

## Convergence Tests

### Ratio Test

```rust
use expr_core::Store;
use summation::{ratio_test, ConvergenceResult};

fn main() {
    let mut st = Store::new();
    let k = st.sym("k");
    let half = st.rat(1, 2);
    
    // Test convergence of ∑ (1/2)^k
    let term = st.pow(half, k);
    
    match ratio_test(&mut st, term, "k") {
        Some(ConvergenceResult::Convergent) => println!("Series converges"),
        Some(ConvergenceResult::Divergent) => println!("Series diverges"),
        Some(ConvergenceResult::Inconclusive) => println!("Test inconclusive"),
        None => println!("Cannot determine"),
    }
}
```

## Performance

- **Arithmetic/Geometric sums:** O(1) - closed form
- **Power sums:** O(1) for n ≤ 5
- **Gosper's algorithm:** O(n) polynomial operations
- **Zeilberger's algorithm:** O(n²) for certificate computation
- **Products:** O(n) for concrete evaluation, O(1) for symbolic

## Test Coverage

- **Phase 5 Total:** 59 tests across all modules
  - Basic sums: 21 tests
  - Hypergeometric: 9 tests
  - Integration tests: 7 tests
  - Pochhammer: 10 tests
  - Convergence: 3 tests
  - Products: 3 tests + 6 integration tests
- **All tests:** 100% passing

## API Reference

```rust
// Public exports from summation crate
pub use sum_arithmetic;          // Arithmetic series
pub use sum_geometric;           // Geometric series
pub use sum_power;               // Power sums
pub use gosper_sum;              // Gosper's algorithm
pub use zeilberger_recurrence;   // Zeilberger's algorithm
pub use evaluate_finite_product; // Product evaluation
pub use product_to_gamma_ratio;  // Gamma function connection
pub use rising_factorial;        // Pochhammer symbols
pub use falling_factorial;       // Falling factorial
pub use ratio_test;              // Convergence tests
```

## See Also

- [Phase 6: Enhanced Simplification](phase6-examples.md)
- [Calculus Documentation](calculus.md)
- [Interactive Playground](playground.html)
