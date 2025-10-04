# Property-Based Testing (Phase L)

Property-based testing for Symmetrica using `proptest` to verify mathematical properties hold for all valid inputs.

## Overview

Unlike traditional unit tests that check specific examples, property-based tests verify that **properties** hold for randomly generated inputs. This approach catches edge cases and validates mathematical correctness across the entire input space.

## Framework

We use [`proptest`](https://docs.rs/proptest/) which provides:
- Automatic random input generation
- Shrinking (minimizing failing test cases)
- Deterministic replay with seeds
- Integration with standard `cargo test`

## Properties Tested

### expr_core (Fundamental Properties)

**Hash-Consing:**
- `prop_int_stable`: Same integer produces same ID
- `prop_rational_normalized`: Rationals are properly normalized

**Algebraic Properties:**
- `prop_add_commutative`: `a + b == b + a`
- `prop_add_associative`: `(a + b) + c == a + (b + c)`
- `prop_mul_commutative`: `a * b == b * a`
- `prop_mul_associative`: `(a * b) * c == a * (b * c)`
- `prop_distributive`: `a * (b + c) == a*b + a*c`

**Power Laws:**
- `prop_pow_zero`: `x^0 == 1`
- `prop_pow_one`: `x^1 == x`

**Robustness:**
- `prop_to_string_works`: String conversion never panics
- `prop_simplify_idempotent`: `simplify(simplify(e)) == simplify(e)`

### simplify (Simplification Properties)

**Idempotence:**
- `prop_simplify_idempotent`: Repeated simplification produces same result
- Applies to complex expressions with multiple terms

**Identity Laws:**
- `prop_simplify_add_zero`: `a + 0 == a`
- `prop_simplify_mul_one`: `a * 1 == a`
- `prop_simplify_mul_zero`: `a * 0 == 0`

**Algebraic Correctness:**
- `prop_collect_like_terms`: `a*x + a*x` simplifies correctly
- `prop_cancel_same_terms`: `x - x == 0`
- `prop_double_negation`: `-(-a) == a`

**Exactness:**
- `prop_rational_addition`: Rational arithmetic preserves exactness
- No `NaN` or `inf` values produced

**Non-Expansion:**
- `prop_simplify_not_expanding`: Simplification doesn't make expressions significantly larger

### calculus (Calculus Properties)

**Differentiation Rules:**
- `prop_diff_constant`: `d/dx(c) = 0`
- `prop_diff_variable`: `d/dx(x) = 1`
- `prop_diff_linear`: `d/dx(a*x) = a`
- `prop_power_rule`: `d/dx(x^n) = n*x^(n-1)`

**Linearity:**
- `prop_diff_linear_sum`: `d/dx(f + g) = d/dx(f) + d/dx(g)`
- `prop_diff_preserves_sum_structure`: Differentiation preserves sum structure

**Function Derivatives:**
- `prop_diff_sin`: `d/dx(sin(x)) = cos(x)`
- `prop_diff_cos`: `d/dx(cos(x)) = -sin(x)`
- `prop_diff_exp`: `d/dx(exp(x)) = exp(x)`

**Higher-Order:**
- `prop_second_derivative`: Second derivatives work correctly

**Calculus Fundamental Theorem:**
- `prop_integrate_diff_inverse`: Integration and differentiation are inverses
- `prop_integrate_constant`: `∫c dx = c*x`

## Running Property Tests

**All property tests:**
```bash
cargo test --workspace
```

**Specific crate:**
```bash
cargo test -p expr_core proptests
cargo test -p simplify proptests
cargo test -p calculus proptests
```

**With more cases (default is 256):**
```bash
PROPTEST_CASES=1000 cargo test proptests
```

**With specific seed (for reproducibility):**
```bash
PROPTEST_SEED=12345 cargo test proptests
```

## Writing Property Tests

**Basic structure:**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_my_property(input in strategy()) {
        // Test code
        prop_assert!(condition);
    }
}
```

**Example - Commutativity:**
```rust
proptest! {
    #[test]
    fn prop_add_commutative(a in -100i64..=100, b in -100i64..=100) {
        let mut st = Store::new();
        let ea = st.int(a);
        let eb = st.int(b);
        
        let sum1 = st.add(vec![ea, eb]);
        let sum2 = st.add(vec![eb, ea]);
        
        prop_assert_eq!(st.get(sum1).digest, st.get(sum2).digest);
    }
}
```

## Best Practices

### 1. Focus on Properties, Not Examples
✅ **Good**: "Addition is commutative for all integers"  
❌ **Bad**: "2 + 3 equals 5"

### 2. Use Appropriate Ranges
```rust
// Too large - may cause overflow
fn bad_range() -> impl Strategy<Value = i64> {
    any::<i64>()
}

// Better - bounded range
fn good_range() -> impl Strategy<Value = i64> {
    -1000i64..=1000
}
```

### 3. Test Mathematical Properties
- **Algebraic**: Commutative, associative, distributive, identity laws
- **Calculus**: Linearity, chain rule, fundamental theorem
- **Equivalence**: Canonical forms, simplification idempotence
- **Robustness**: No panics, valid outputs

### 4. Keep Tests Fast
- Use small ranges for expensive operations
- Limit recursion depth in generated expressions
- Use `#[cfg(test)]` modules for helper functions

### 5. Handle Shrinking
When a property fails, proptest **shrinks** the input to find the minimal failing case:
```
Test failed for (a: 42, b: -17)
Shrinking to (a: 1, b: -1)
```

## Debugging Failed Tests

**Reproduce a failure:**
```bash
# Proptest prints the seed when a test fails
PROPTEST_SEED=1234567890 cargo test prop_my_test
```

**Add debug output:**
```rust
proptest! {
    #[test]
    fn prop_debug(a in small_int()) {
        eprintln!("Testing with a = {}", a);
        // test code
    }
}
```

**Use `prop_assume!` to filter inputs:**
```rust
proptest! {
    #[test]
    fn prop_division(a in small_int(), b in small_int()) {
        prop_assume!(b != 0); // Skip when b is zero
        // test division
    }
}
```

## Integration with CI

Property tests run in CI like regular tests:
```bash
cargo test --workspace  # Runs property tests automatically
```

For deeper fuzzing in CI:
```bash
PROPTEST_CASES=10000 cargo test proptests
```

## Comparison with Fuzzing

| Aspect | Property Tests | Fuzz Tests |
|--------|---------------|------------|
| Tool | proptest | cargo-fuzz |
| Focus | Mathematical properties | Crashes/panics |
| Speed | Fast (256-1000 cases) | Slow (millions of cases) |
| Shrinking | Automatic | Manual (cmin/tmin) |
| CI Integration | Built-in | Requires special setup |
| Use Case | Correctness verification | Robustness testing |

**Best Practice**: Use **both**
- Property tests for mathematical correctness
- Fuzz tests for finding crashes and edge cases

## Coverage

Property tests significantly increase code coverage by exploring:
- Edge cases (0, negative, boundary values)
- Unusual combinations
- Different orderings
- Random expressions

## Roadmap Alignment

Property testing implements **Phase L: Hardening, Fuzzing, Differential Testing** deliverables:
- ✅ Property tests for core algebraic laws
- ✅ Property tests for calculus operations
- ✅ Idempotence verification
- ✅ Mathematical correctness validation
- 🔲 Differential testing vs reference CAS (future)

## See Also

- [proptest documentation](https://docs.rs/proptest/)
- [Fuzzing](fuzzing.md) - Complementary testing approach
- [Testing Rust](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [QuickCheck (Haskell inspiration)](https://hackage.haskell.org/package/QuickCheck)
