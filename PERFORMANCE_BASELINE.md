# Performance Baseline - v1.0.0-rc.1

**Established:** October 5, 2025  
**Version:** v1.0.0-rc.1  
**Purpose:** Track performance metrics and identify optimization opportunities

## Benchmark Infrastructure

Symmetrica includes comprehensive benchmarks using Criterion.rs across 6 core crates:

- **expr_core** (8 benchmarks) - Expression building and hash-consing
- **simplify** (7 benchmarks) - Simplification operations
- **calculus** (8 benchmarks) - Differentiation and integration
- **solver** (6 benchmarks) - Equation solving
- **polys** (16 benchmarks) - Polynomial operations
- **matrix** (23 benchmarks) - Linear algebra

**Total:** 68 benchmark functions

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench --workspace

# Run specific crate benchmarks
cargo bench -p expr_core
cargo bench -p simplify
cargo bench -p calculus
cargo bench -p solver
cargo bench -p polys
cargo bench -p matrix

# View HTML reports
open target/criterion/report/index.html
```

## Performance Characteristics

### Expression Building (expr_core)
- **Hash-consing:** O(1) lookup via HashMap
- **Canonical constructors:** O(n log n) for sorting operands
- **Memory:** Sub-linear growth via DAG sharing

### Simplification (simplify)
- **Idempotent:** `simplify(simplify(e)) == simplify(e)`
- **Like-term collection:** O(n) for n terms
- **Power merging:** O(n) for n factors

### Calculus Operations
- **Differentiation:** O(n) tree traversal
- **Integration:** Conservative rules, pattern matching
- **Series expansion:** O(k) for k terms

### Polynomial Operations (polys)
- **GCD:** O(n²) Euclidean algorithm
- **Multiplication:** O(n·m) for degrees n, m
- **Factorization:** Rational root theorem + trial division

### Matrix Operations (matrix)
- **Determinant:** O(n³) Bareiss algorithm (fraction-free)
- **Linear solve:** O(n³) Gaussian elimination
- **LU decomposition:** O(n³) with partial pivoting

## Known Performance Characteristics

### Strengths ✅
1. **Hash-consing efficiency:** Structural sharing prevents duplication
2. **Exact arithmetic:** No floating-point errors (uses rationals)
3. **Deterministic:** Stable ordering and reproducible results
4. **Fraction-free algorithms:** Controls intermediate expression swell

### Areas for Future Optimization 🔄
1. **Integration:** Currently conservative, could expand techniques
2. **Polynomial factorization:** Could add more sophisticated algorithms
3. **Parallel operations:** Large Add/Mul could benefit from parallelization
4. **Caching:** More aggressive memoization for expensive operations

## Baseline Metrics (v1.0.0-rc.1)

### Test Suite Performance
- **Total tests:** 704
- **Test execution time:** ~2-3 seconds (full workspace)
- **Coverage:** 81.91% (1354/1653 lines)

### Build Performance
- **Clean build:** ~10-15 seconds (debug)
- **Incremental build:** ~1-2 seconds
- **Release build:** ~20-30 seconds

### Memory Characteristics
- **Expression DAG:** Shared structure, O(unique nodes)
- **Store overhead:** HashMap + Vec storage
- **Cache size:** Bounded by unique expressions

## Performance Testing Strategy

### 1. Micro-benchmarks (Criterion)
- Individual operation performance
- Regression detection
- HTML reports for tracking

### 2. Macro-benchmarks (Integration)
- End-to-end workflows
- Real-world expression sizes
- Memory profiling

### 3. Scalability Tests
- Large expression handling (10⁴+ nodes)
- Deep nesting (100+ levels)
- Wide operations (1000+ children)

## Optimization Opportunities (Post-1.0)

### High Priority
1. **Memoization expansion**
   - Cache more expensive operations
   - Implement cache eviction strategies
   - Profile cache hit rates

2. **Parallel simplification**
   - Parallelize large Add/Mul operations
   - Work-stealing for independent subexpressions
   - Maintain deterministic ordering

3. **Integration improvements**
   - Implement more heuristic Risch fragments
   - Add integration by parts orchestrator
   - Table lookup with validation

### Medium Priority
4. **Special function library**
   - Extend elementary functions
   - Add special function derivatives
   - Implement more series expansions

5. **Polynomial optimizations**
   - FFT-based multiplication for large degrees
   - More sophisticated factorization
   - Gröbner basis (behind feature flag)

### Low Priority
6. **Optional bignum support**
   - Feature flag for arbitrary precision
   - GMP/MPFR integration
   - Maintain exact arithmetic guarantees

## Regression Prevention

### CI Performance Checks
- Benchmark smoke tests in CI
- Alert on significant regressions (>10%)
- Track performance trends over releases

### Local Development
```bash
# Before optimization
cargo bench -p <crate> -- --save-baseline before

# After optimization
cargo bench -p <crate> -- --baseline before

# Compare results
```

## Performance Goals (1.x Series)

### 1.1.0 Goals
- [ ] Establish automated performance tracking
- [ ] Identify top 3 bottlenecks via profiling
- [ ] Document performance best practices

### 1.2.0 Goals
- [ ] Implement memoization improvements
- [ ] Optimize hot paths identified in 1.1.0
- [ ] Add parallel simplification (optional feature)

### 1.3.0 Goals
- [ ] Integration technique expansion
- [ ] Special function library growth
- [ ] Memory usage optimization

## Profiling Tools

### Recommended Tools
- **Criterion.rs:** Micro-benchmarking (already integrated)
- **cargo-flamegraph:** CPU profiling
- **heaptrack:** Memory profiling
- **perf:** Linux performance analysis
- **Instruments:** macOS profiling

### Profiling Commands
```bash
# CPU profiling
cargo install flamegraph
cargo flamegraph --bench <bench_name>

# Memory profiling
valgrind --tool=massif target/release/<binary>

# Benchmark with profiling
cargo bench --profile-time 10
```

## Notes

- All optimizations must maintain mathematical correctness
- Performance improvements are non-breaking changes (minor version bumps)
- Benchmarks are part of the test suite and run in CI
- HTML reports provide detailed performance analysis

## References

- Criterion.rs documentation: https://bheisler.github.io/criterion.rs/
- Rust performance book: https://nnethercote.github.io/perf-book/
- Benchmark results: `target/criterion/report/index.html`

---

**Last Updated:** 2025-10-05  
**Version:** v1.0.0-rc.1  
**Status:** Baseline established, ready for optimization tracking
