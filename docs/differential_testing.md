# Differential Testing (Phase L)

## Overview

Differential testing compares Symmetrica's outputs with a reference Computer Algebra System (SymPy) to validate mathematical correctness. This ensures our implementation matches well-established mathematical identities and catches semantic bugs that unit tests might miss.

## What is Differential Testing?

**Differential testing** validates software by comparing its output against a trusted reference implementation on the same inputs. For a CAS:

- **Input**: Mathematical expression (e.g., `x^2 + 3*x`)
- **Operation**: Differentiation, simplification, integration, etc.
- **Comparison**: Symmetrica result vs. SymPy result
- **Validation**: Check if results are mathematically equivalent

## Requirements

### Python and SymPy

```bash
# Install Python 3 (if not already installed)
brew install python3  # macOS
# or: apt-get install python3  # Linux

# Install SymPy
pip3 install sympy
```

### Verification

```bash
python3 -c "import sympy; print(sympy.__version__)"
```

## Test Infrastructure

### Location

```
crates/tests_e2e/tests/differential_tests.rs
```

### Running Tests

**All differential tests:**
```bash
cargo test --test differential_tests
```

**Specific test:**
```bash
cargo test --test differential_tests test_diff_power_rule
```

**With output:**
```bash
cargo test --test differential_tests -- --nocapture
```

### Automatic Skipping

Tests automatically skip if Python/SymPy is unavailable:
```rust
if !sympy_available() {
    eprintln!("Skipping differential test: SymPy not available");
    return;
}
```

## Test Categories

### 1. Differentiation

**Tests:**
- `test_diff_power_rule` - d/dx(x^n) = n*x^(n-1)
- `test_diff_product_rule` - d/dx(f*g) = f'*g + f*g'
- `test_diff_chain_rule` - d/dx(f(g(x))) = f'(g(x))*g'(x)

**Example:**
```rust
// Test: d/dx(x^3) = 3*x^2
let x = st.sym("x");
let x3 = st.pow(x, st.int(3));
let deriv = diff(&mut st, x3, "x");

// Compare with SymPy
let sympy_result = sympy_eval("x**3|x", "diff");
```

### 2. Integration

**Tests:**
- `test_integrate_power_rule` - ∫x^n dx = x^(n+1)/(n+1)
- `test_integrate_exponential` - ∫exp(x) dx = exp(x)
- `test_integrate_trig` - ∫sin(x) dx = -cos(x)
- `test_fundamental_theorem_calculus` - d/dx(∫f dx) = f

**Example:**
```rust
// Test: ∫x^2 dx = x^3/3
let x2 = st.pow(st.sym("x"), st.int(2));
let integral = integrate(&mut st, x2, "x");

// Verify by differentiation
let deriv = diff(&mut st, integral, "x");
assert_eq!(deriv, x2);  // Fundamental theorem
```

### 3. Simplification

**Tests:**
- `test_simplify_algebraic` - (x+1)^2 - (x^2 + 2x + 1) = 0
- `test_algebraic_identities` - (a+b)^2 = a^2 + 2ab + b^2

**Example:**
```rust
// Test: x + x should simplify to 2*x
let sum = st.add(vec![x, x]);
let simplified = simplify(&mut st, sum);
// Compare with SymPy's simplify(x + x)
```

### 4. Comprehensive Suite

**Test:**
- `test_differential_comprehensive` - Batch test multiple operations

Runs a suite of expressions through both systems and reports results.

## Comparison Strategy

### Exact Structural Comparison

```rust
assert_eq!(
    st.get(symmetrica_result).digest,
    st.get(expected).digest,
    "Results should be structurally identical"
);
```

Works when both systems produce identical canonical forms.

### Semantic Equivalence

For expressions that may differ in structure but are mathematically equivalent:

```rust
// Verify by differentiation
let deriv_sym = diff(&mut st, symmetrica_result, "x");
let deriv_expected = diff(&mut st, expected_result, "x");
assert_eq!(deriv_sym, deriv_expected);
```

### String Pattern Matching

Heuristic check for complex expressions:

```rust
let result_str = st.to_string(simplified);
assert!(result_str.contains("cos"), "Should contain cos term");
```

## SymPy Integration

### Architecture

```
┌─────────────┐
│ Differential│
│   Test      │
└──────┬──────┘
       │
       ├──────────────┐
       │              │
       ▼              ▼
┌──────────┐   ┌──────────┐
│Symmetrica│   │  Python  │
│  Engine  │   │  + SymPy │
└────┬─────┘   └────┬─────┘
     │              │
     └───── ═══ ────┘
        Compare
```

### SymPy Evaluation Function

```rust
fn sympy_eval(expr: &str, operation: &str) -> Option<String> {
    let python_code = format!(
        r#"
import sympy as sp
x = sp.Symbol('x')
expr = sp.parse_expr('{}')
result = sp.{}(expr)
print(result)
"#,
        expr, operation
    );
    
    Command::new("python3")
        .args(["-c", &python_code])
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
}
```

### Supported Operations

| Operation | SymPy Function | Example |
|-----------|---------------|---------|
| Differentiation | `sp.diff(expr, var)` | `diff(x**2, x)` |
| Integration | `sp.integrate(expr, var)` | `integrate(x**2, x)` |
| Simplification | `sp.simplify(expr)` | `simplify(x + x)` |
| Expansion | `sp.expand(expr)` | `expand((x+1)**2)` |

## Limitations

### 1. Canonical Form Differences

Symmetrica and SymPy may produce different but equivalent forms:

**Symmetrica:** `x^3/3`  
**SymPy:** `x**3/3`

**Solution:** Use semantic equivalence checks (differentiation, substitution).

### 2. Integration Constants

```rust
// Symmetrica: x^2/2
// SymPy: x^2/2 + C
```

Indefinite integrals differ by constants. Verify by differentiation instead.

### 3. Symbolic vs. Numeric

Differential testing focuses on **symbolic** operations. Numeric evaluation may differ due to:
- Floating-point precision
- Approximation methods
- Domain restrictions

### 4. Unsupported Operations

Some operations may not have direct equivalents:
- Custom functions
- Piecewise expressions (different syntax)
- Special functions not in SymPy core

## Best Practices

### 1. Test Mathematical Properties

✅ **Good**: "d/dx(∫f dx) = f (fundamental theorem)"  
❌ **Bad**: "∫x^2 dx exactly equals 'x^3/3'"

### 2. Use Well-Defined Identities

Test operations with clear, unambiguous mathematical meanings:
- Power rule
- Product rule
- Fundamental theorem of calculus
- Algebraic identities

### 3. Handle Failures Gracefully

```rust
if let Some(sympy_result) = sympy_eval(expr, "diff") {
    // Compare results
} else {
    eprintln!("SymPy evaluation failed (expected for complex cases)");
}
```

### 4. Document Edge Cases

When tests reveal differences:
```rust
// Note: SymPy expands (a+b)^2 automatically
// Symmetrica keeps it factored unless expand() is called
```

## CI Integration

### GitHub Actions

```yaml
- name: Install SymPy
  run: pip3 install sympy

- name: Differential Tests
  run: cargo test --test differential_tests -- --nocapture
```

### Optional in CI

Since differential testing requires Python:
```yaml
- name: Differential Tests (Optional)
  run: cargo test --test differential_tests || true
  continue-on-error: true
```

## Debugging Mismatches

### 1. Print Both Results

```rust
eprintln!("Symmetrica: {}", st.to_string(sym_result));
eprintln!("SymPy:      {}", sympy_result);
```

### 2. Check Semantic Equivalence

```rust
// If structural equality fails, try:
let diff_sym = diff(&mut st, sym_result, "x");
let diff_sympy = diff(&mut st, sympy_result, "x");
// If derivatives match, results are equivalent
```

### 3. Simplify Both

```rust
let sym_simplified = simplify(&mut st, sym_result);
let sympy_parsed = from_sexpr(&mut st, &sympy_result)?;
let sympy_simplified = simplify(&mut st, sympy_parsed);
```

### 4. Use Numeric Sampling

```rust
// Evaluate at random points
for x_val in [-1.0, 0.0, 1.0, 2.5] {
    let sym_val = evaluate(&st, sym_result, "x", x_val);
    let sympy_val = evaluate(&st, sympy_result, "x", x_val);
    assert_approx_eq!(sym_val, sympy_val, 1e-10);
}
```

## Coverage Metrics

Differential testing significantly increases confidence:

**Before differential testing:**
- Unit tests: ✅ Specific cases
- Property tests: ✅ Mathematical laws
- Fuzz tests: ✅ No crashes

**After differential testing:**
- ✅ Matches established reference
- ✅ Semantically correct results
- ✅ Validates entire pipeline

## Example Output

```
$ cargo test --test differential_tests -- --nocapture

running 10 tests
test test_diff_power_rule ... 
  Symmetrica: 3 * x^2
  SymPy:      3*x**2
ok

test test_integrate_power_rule ...
  Symmetrica ∫x^2 dx: 1/3 * x^3
  SymPy ∫x^2 dx: x**3/3
  Derivative check: ✓
ok

test test_fundamental_theorem_calculus ...
  Original:    x^3 + 2 * x
  ∫f dx:       1/4 * x^4 + x^2
  d/dx(∫f dx): x^3 + 2 * x
  ✓ Fundamental theorem verified
ok

test result: ok. 10 passed; 0 failed
```

## Roadmap Alignment

Phase L deliverables:
- ✅ Fuzzing on parser/simplifier/differentiation
- ✅ Property-based testing
- ✅ **Differential testing vs reference CAS** ← **This implementation**
- 🔲 Coverage metrics dashboard (future)
- 🔲 Crash-free fuzzing threshold for 1.0 (future)

## Future Enhancements

### 1. More Reference Systems

Compare against multiple CAS:
- SymPy (Python)
- Mathematica (via WolframScript)
- Maple (via command-line)
- SageMath

### 2. Automated Equivalence Checking

Smart comparison that handles:
- Different canonical forms
- Trigonometric identities
- Algebraic rearrangements

### 3. Continuous Differential Testing

Run differential tests on:
- Every commit (CI)
- Nightly builds (extended suite)
- Release candidates (comprehensive)

### 4. Differential Fuzzing

Combine fuzzing with differential testing:
```rust
// Generate random expression
let expr = fuzz_generate_expr();

// Test both systems
let sym_result = symmetrica_diff(expr);
let sympy_result = sympy_diff(expr);

// Verify equivalence
assert_equivalent(sym_result, sympy_result);
```

## See Also

- [Fuzzing](fuzzing.md) - Robustness testing
- [Property Testing](property_testing.md) - Mathematical properties
- [SymPy Documentation](https://docs.sympy.org/)
- [Metamorphic Testing](https://en.wikipedia.org/wiki/Metamorphic_testing)

## References

- Differential Testing: McKeeman, "Differential Testing for Software" (1998)
- SymPy: Meurer et al., "SymPy: symbolic computing in Python" (2017)
- CAS Testing: Fateman, "Comparing Computer Algebra Systems" (1992)
