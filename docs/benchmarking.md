# Benchmarking (Phase L)

Performance benchmarking infrastructure for Symmetrica using Criterion.rs.

## Overview

Benchmarks track performance of critical operations to:
- Detect performance regressions
- Guide optimization efforts
- Validate algorithmic complexity
- Ensure production readiness for 1.0

## Running Benchmarks

**All benchmarks:**
```bash
cargo bench --workspace
```

**Specific crate:**
```bash
cargo bench -p expr_core
cargo bench -p simplify
cargo bench -p calculus
cargo bench -p solver
```

**Specific benchmark:**
```bash
cargo bench -p expr_core -- hash_consing
```

**Quick test (faster):**
```bash
cargo bench --workspace -- --quick
```

## Benchmark Suites

### expr_core Benchmarks

Tests fundamental expression operations:

**`bench_build_atoms`** - Atom creation throughput
- Tests: 10K, 50K, 100K symbols
- Measures hash-consing efficiency

**`bench_hash_consing`** - Deduplication performance
- 1000 identical expressions
- Validates O(1) lookup

**`bench_add_chain`** - Addition performance
- Tests: 1K, 5K, 10K terms
- Validates canonical ordering

**`bench_mul_chain`** - Multiplication performance
- 100 factors
- Tests canonicalization overhead

**`bench_pow_operations`** - Power operations
- Symbolic x^n for n=1..20
- Tests power simplification

**`bench_rational_ops`** - Rational normalization
- 100×100 rational creation
- Tests GCD performance

**`bench_deep_expr_tree`** - Nested expression depth
- Depth 10 expression tree
- Tests recursive operations

### simplify Benchmarks

Tests algebraic simplification:

**`bench_simplify_idempotence`** - Idempotent simplification
- Multiple simplify passes
- Should show constant time

**`bench_collect_like_terms`** - Like-term collection
- 10 terms with same variable
- Tests coefficient merging

**`bench_distributive_law`** - Distributive expansion
- `(x+y)(x+z)` expansion
- Tests term collection

**`bench_rational_arithmetic`** - Rational simplification
- Sum of 20 fractions
- Tests fraction normalization

**`bench_polynomial_simplify`** - Polynomial simplification
- 5th degree polynomial
- Tests power combining

**`bench_cancel_terms`** - Term cancellation
- `x - x = 0`
- Tests zero detection

**`bench_nested_simplify`** - Nested identity simplification
- `((x+0)*1) + ((y*1)+0)`
- Tests multi-level simplification

### calculus Benchmarks

Tests differentiation and integration:

**`bench_diff_polynomial`** - Polynomial differentiation
- 5th degree polynomial
- Tests power rule

**`bench_diff_product_rule`** - Product rule
- `d/dx(x * x^2)`
- Tests product differentiation

**`bench_diff_chain_rule`** - Chain rule
- `d/dx(sin(x^2))`
- Tests nested differentiation

**`bench_diff_trig_functions`** - Trigonometric functions
- `d/dx(sin(x) + cos(x))`
- Tests function derivatives

**`bench_diff_nested`** - Repeated differentiation
- 10 successive derivatives
- Tests accumulation overhead

**`bench_integrate_polynomial`** - Polynomial integration
- `∫x^3 dx`
- Tests basic integration

**`bench_integrate_sum`** - Sum integration
- Polynomial sum integration
- Tests linearity

### solver Benchmarks

Tests equation solving:

**`bench_solve_linear`** - Linear solving
- `2x + 3 = 0`
- Baseline solver performance

**`bench_solve_quadratic`** - Quadratic solving
- `x^2 - 5x + 6 = 0`
- Tests discriminant computation

**`bench_solve_quadratic_rational`** - Rational coefficients
- `2x^2 + 3x - 6 = 0`
- Tests rational arithmetic in solving

**`bench_solve_cubic`** - Cubic solving
- `x^3 - 6x^2 + 11x - 6 = 0`
- Tests rational root theorem

**`bench_solve_perfect_square`** - Perfect square
- `x^2 - 4 = 0`
- Tests factorization

## Understanding Results

**Sample output:**
```
build_atoms/10000      time:   [1.234 ms 1.256 ms 1.278 ms]
                       thrpt:  [7.82 Kelem/s 7.96 Kelem/s 8.11 Kelem/s]
```

**Metrics:**
- **time**: Mean execution time with confidence interval
- **thrpt**: Throughput (operations per second)
- **change**: Comparison with previous run (if available)

**Regression detection:**
```
Performance has regressed:
  time:   [+15.23% +18.45% +21.67%] (p = 0.00 < 0.05)
```

This indicates a statistically significant performance regression.

## HTML Reports

Criterion generates detailed HTML reports in `target/criterion/`:

```bash
# Run benchmarks and open report
cargo bench --workspace
open target/criterion/report/index.html
```

Reports include:
- Performance plots
- Statistical analysis
- Regression detection
- Historical comparison

## Baseline Comparisons

**Save current performance as baseline:**
```bash
cargo bench --workspace -- --save-baseline main
```

**Compare against baseline:**
```bash
cargo bench --workspace -- --baseline main
```

**Example workflow:**
```bash
# On main branch
git checkout main
cargo bench -- --save-baseline main

# On feature branch
git checkout feature/optimization
cargo bench -- --baseline main
```

## CI Integration

Run benchmarks in CI to track performance over time:

```yaml
- name: Run benchmarks
  run: cargo bench --workspace --no-fail-fast
  
- name: Archive benchmark results
  uses: actions/upload-artifact@v3
  with:
    name: benchmark-results
    path: target/criterion/
```

For PR checks (quick mode):
```bash
cargo bench --workspace -- --quick --noplot
```

## Profiling

For deeper performance analysis:

**Flamegraph:**
```bash
cargo install flamegraph
cargo flamegraph --bench expr_benches
```

**Perf (Linux):**
```bash
perf record --call-graph dwarf cargo bench -p expr_core
perf report
```

**Instruments (macOS):**
```bash
cargo bench -p expr_core -- --profile-time=5
```

## Performance Goals

Target performance for 1.0 release:

| Operation | Target | Current | Status |
|-----------|--------|---------|--------|
| Hash-consing lookup | < 100ns | TBD | 🔄 |
| Add 1000 terms | < 1ms | TBD | 🔄 |
| Simplify polynomial | < 100μs | TBD | 🔄 |
| Differentiate | < 10μs | TBD | 🔄 |
| Solve quadratic | < 50μs | TBD | 🔄 |

## Best Practices

### 1. Use `black_box` for Optimization Prevention
```rust
use criterion::black_box;

b.iter(|| {
    let result = expensive_operation();
    black_box(result); // Prevent dead code elimination
});
```

### 2. Setup Outside Timing Loop
```rust
c.bench_function("my_bench", |b| {
    let data = setup_data(); // Outside iter()
    b.iter(|| {
        process(black_box(&data))
    });
});
```

### 3. Use Throughput for Scaling Tests
```rust
group.throughput(Throughput::Elements(n as u64));
```

### 4. Warm-up Iterations
Criterion automatically handles warm-up, but adjust if needed:
```rust
group.warm_up_time(Duration::from_secs(3));
```

### 5. Measurement Time
Increase for stable results:
```rust
group.measurement_time(Duration::from_secs(10));
```

## Optimization Workflow

1. **Identify bottleneck** - Run benchmarks to find slow operations
2. **Profile** - Use flamegraph/perf to find hotspots
3. **Optimize** - Make targeted improvements
4. **Measure** - Re-run benchmarks to verify improvement
5. **Baseline** - Save as new baseline if improved

## Common Issues

**High variance:**
- Close other applications
- Disable CPU frequency scaling
- Increase measurement time

**Benchmarks take too long:**
- Use `--quick` flag
- Reduce iteration counts
- Focus on specific benchmarks

**Memory issues:**
- Reduce problem sizes
- Use `cargo bench --release`
- Clear criterion cache: `rm -rf target/criterion`

## Roadmap Alignment

Benchmarking implements **Phase L: Hardening, Fuzzing, Differential Testing** deliverables:
- ✅ Performance regression detection
- ✅ Optimization guidance
- ✅ Production readiness validation
- 🔲 Continuous performance tracking in CI
- 🔲 Performance comparison dashboard

## See Also

- [Criterion.rs documentation](https://bheisler.github.io/criterion.rs/book/)
- [Fuzzing](fuzzing.md) - Complementary robustness testing
- [Property testing](property_testing.md) - Correctness verification
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
