# Migration Guide to Symmetrica 1.0

This guide helps you upgrade from Symmetrica 0.1.x to 1.0.0.

## Overview

Symmetrica 1.0.0 is the first stable release with **API stability guarantees**. The good news: **there are no breaking changes from 0.1.0 to 1.0.0**. This is a stabilization release.

## Quick Start

### For Rust Users

Update your `Cargo.toml`:

```toml
[dependencies]
expr_core = "1.0"
simplify = "1.0"
calculus = "1.0"
# ... other crates as needed
```

Or if using the workspace:

```toml
[dependencies]
symmetrica = "1.0"  # If we publish a unified crate
```

Then run:
```bash
cargo update
cargo test  # Verify everything works
```

### For Python Users

```bash
pip install --upgrade symmetrica>=1.0.0
```

### For JavaScript/WASM Users

```bash
npm install symmetrica-wasm@^1.0.0
```

## What's New in 1.0.0

### Stability Guarantees

1. **Semantic Versioning**: All public APIs follow semver strictly
2. **No Breaking Changes**: Until 2.0.0, the API remains stable
3. **Deprecation Policy**: At least 2 minor versions (6 months) notice
4. **Mathematical Correctness**: Bug fixes won't be considered breaking changes

See [API_STABILITY.md](API_STABILITY.md) for full details.

### Documentation Improvements

- ✅ **SECURITY.md**: Security policy and vulnerability reporting
- ✅ **API_STABILITY.md**: Detailed stability guarantees
- ✅ **Comprehensive module docs**: All 23 module guides complete
- ✅ **Migration guide**: This document

### Quality Improvements

- ✅ **714 tests**: 176% over target (400+ unit, 35+ integration, 30+ property)
- ✅ **81.91% code coverage**: Above 80% threshold
- ✅ **Fuzz testing**: 4 fuzz targets with crash-free validation
- ✅ **Differential testing**: SymPy validation for correctness
- ✅ **Performance benchmarks**: 68 benchmarks across 6 crates

## API Changes from 0.1.0

**Good news: There are NO breaking API changes.** All code that worked in 0.1.0 will work in 1.0.0.

### No Changes Required

The following APIs are **100% compatible**:

#### Core Expression Building
```rust
use expr_core::Store;

let mut st = Store::new();
let x = st.sym("x");
let expr = st.add(vec![st.pow(x, st.int(2)), st.int(1)]);
// ✅ Works identically in 0.1.0 and 1.0.0
```

#### Simplification
```rust
use simplify::simplify;

let simplified = simplify(&mut st, expr);
// ✅ No changes needed
```

#### Calculus
```rust
use calculus::{diff, integrate};

let derivative = diff(&mut st, expr, "x");
let integral = integrate(&mut st, expr, "x");
// ✅ No changes needed
```

#### Polynomial Operations
```rust
use polys::{expr_to_unipoly, unipoly_to_expr};

let poly = expr_to_unipoly(&st, expr, "x").unwrap();
let back = unipoly_to_expr(&mut st, &poly);
// ✅ No changes needed
```

#### Matrix Operations
```rust
use matrix::MatrixQ;

let m = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
let det = m.det_bareiss();
// ✅ No changes needed
```

#### Equation Solving
```rust
use solver::solve_univariate;

let roots = solve_univariate(&mut st, expr, "x").unwrap();
// ✅ No changes needed
```

#### I/O Operations
```rust
use io::{to_sexpr, from_sexpr, to_json, to_latex};

let sexpr = to_sexpr(&st, expr);
let json = to_json(&st, expr);
let latex = to_latex(&st, expr);
// ✅ No changes needed
```

## Behavioral Changes

### None

There are **no intentional behavioral changes** from 0.1.0 to 1.0.0, except:

- Bug fixes that produce more correct mathematical results
- Performance improvements (same results, faster execution)

If you observe different behavior, please [report it as a bug](https://github.com/Sir-Teo/Symmetrica/issues).

## Deprecations

**None.** No APIs are deprecated in 1.0.0.

## New Features (Non-Breaking)

While these are technically available in 0.1.0, they are now **stable** in 1.0.0:

### 1. Security Policy (NEW)
- See [SECURITY.md](SECURITY.md) for vulnerability reporting
- Security best practices documented

### 2. Documentation Tests (NEW)
- Comprehensive documentation validation
- Ensures all required docs are present and complete

### 3. API Stability Guarantees (NEW)
- Clear semver commitment
- Deprecation policy defined
- Mathematical correctness guarantees

## Migration Checklist

Even though no code changes are required, we recommend:

- [ ] **Update dependencies** to 1.0.0 versions
- [ ] **Run your test suite** to verify compatibility
- [ ] **Review [API_STABILITY.md](API_STABILITY.md)** to understand guarantees
- [ ] **Review [SECURITY.md](SECURITY.md)** for security best practices
- [ ] **Update CI/CD** to use 1.0.0 (if pinned)
- [ ] **Update documentation** to reference 1.0.0

## Platform-Specific Notes

### Rust

**MSRV (Minimum Supported Rust Version)**: 1.70.0

No changes from 0.1.0. If your code compiled with 0.1.0, it will compile with 1.0.0.

### Python Bindings

**Python Version**: 3.8+

```python
from symmetrica import Store, simplify, diff

st = Store()
x = st.sym("x")
expr = st.add([st.pow(x, 2), st.int(1)])
simplified = simplify(st, expr)
# ✅ Same API as 0.1.0
```

### WebAssembly

**Node.js**: 14+
**Browsers**: Modern browsers with WASM support

```javascript
import init, { Store } from 'symmetrica-wasm';

await init();
const st = new Store();
// ✅ Same API as 0.1.0
```

## Updating from Pre-0.1.0 Versions

If you're upgrading from a version before 0.1.0 (e.g., development snapshots):

1. **Review [CHANGELOG.md](CHANGELOG.md)** for all changes since your version
2. **Check for renamed crates**: Some crates were consolidated in 0.1.0
3. **Update imports**: Some module paths may have changed
4. **Run tests extensively**: Pre-0.1.0 had no stability guarantees

## Common Patterns

### Pattern 1: Expression Building and Simplification

```rust
use expr_core::Store;
use simplify::simplify;

let mut st = Store::new();
let x = st.sym("x");
let expr = st.add(vec![
    st.mul(vec![st.int(2), x]),
    st.mul(vec![st.int(3), x]),
]);
let simplified = simplify(&mut st, expr);  // Result: 5*x
```

✅ **Status**: Works identically in 0.1.0 and 1.0.0

### Pattern 2: Differentiation Workflow

```rust
use calculus::diff;
use simplify::simplify;

let derivative = diff(&mut st, expr, "x");
let simplified = simplify(&mut st, derivative);
```

✅ **Status**: Works identically in 0.1.0 and 1.0.0

### Pattern 3: Integration with Verification

```rust
use calculus::{diff, integrate};

let integral = integrate(&mut st, expr, "x").unwrap();
let verify = diff(&mut st, integral, "x");
let verified = simplify(&mut st, verify);
// verified should equal original expr
```

✅ **Status**: Works identically in 0.1.0 and 1.0.0

### Pattern 4: Equation Solving

```rust
use solver::solve_univariate;

let equation = st.add(vec![
    st.pow(x, st.int(2)),
    st.mul(vec![st.int(-3), x]),
    st.int(2),
]);
let roots = solve_univariate(&mut st, equation, "x").unwrap();
```

✅ **Status**: Works identically in 0.1.0 and 1.0.0

### Pattern 5: Matrix Linear Algebra

```rust
use matrix::MatrixQ;

let a = MatrixQ::from_i64(2, 2, vec![1, 2, 3, 4]);
let b = MatrixQ::from_i64(2, 1, vec![5, 6]);
let x = a.solve_bareiss(&b).unwrap();
```

✅ **Status**: Works identically in 0.1.0 and 1.0.0

## Performance Considerations

### What's the Same

- Expression building complexity: O(1) due to hash-consing
- Simplification: Idempotent, same algorithmic complexity
- Memory usage: DAG sharing, same memory characteristics

### What Might Be Better

1.0.0 includes various performance improvements, but the algorithmic complexity remains the same. Your code should run at similar or better speeds.

## Testing Your Migration

### Recommended Test Strategy

1. **Update dependencies** to 1.0.0
2. **Run existing tests**: All should pass
3. **Run new tests** (if you added 1.0-specific features)
4. **Performance tests**: Verify no regressions
5. **Integration tests**: Test full workflows

### Example Test

```rust
#[test]
fn test_migration_compatibility() {
    use expr_core::Store;
    use simplify::simplify;
    use calculus::diff;
    
    let mut st = Store::new();
    let x = st.sym("x");
    let expr = st.pow(x, st.int(2));
    
    // Should work identically to 0.1.0
    let derivative = diff(&mut st, expr, "x");
    let simplified = simplify(&mut st, derivative);
    
    // Result: 2*x
    assert_eq!(st.to_string(simplified), "2 * x");
}
```

## Troubleshooting

### Issue: "Cargo can't find version 1.0.0"

**Solution**: Make sure you've run `cargo update` and that 1.0.0 is actually released:

```bash
cargo update
cargo clean
cargo build
```

### Issue: "Tests fail after upgrading"

**Solution**: 
1. Check if you're using internal/unstable APIs (not recommended)
2. Verify you're using compatible versions of all Symmetrica crates
3. Run `cargo clean` and rebuild
4. [Report as a bug](https://github.com/Sir-Teo/Symmetrica/issues) if issue persists

### Issue: "Performance regression"

**Solution**:
1. Benchmark with `cargo bench` to quantify
2. [Report as a bug](https://github.com/Sir-Teo/Symmetrica/issues) with benchmark details
3. We take performance seriously and will investigate

### Issue: "Different mathematical results"

**Solution**:
1. Verify the result is actually incorrect (not just differently formatted)
2. This is a **high-priority bug** if true
3. [Report immediately](https://github.com/Sir-Teo/Symmetrica/issues) with:
   - Input expression
   - Expected result (from 0.1.0)
   - Actual result (from 1.0.0)
   - Steps to reproduce

## Future Plans

After 1.0.0, we plan:

### 1.x Series (Minor Releases)

- **1.1.0**: Enhanced integration techniques (non-breaking)
- **1.2.0**: Extended special functions (non-breaking)
- **1.3.0**: Performance optimizations (non-breaking)
- **1.x.0**: Additional features as needed (all non-breaking)

### 2.0.0 (Major Release)

Only if we need breaking changes:
- Potential: Multivariate solver system
- Potential: Optional bignum support (different backend)
- Potential: Major API improvements based on community feedback

**Timeline**: No earlier than **12 months after 1.0.0 release**

## Getting Help

### Documentation

- **API Docs**: `cargo doc --open` or https://docs.rs/symmetrica
- **Module Guides**: See [docs/](docs/) directory
- **Examples**: See [examples/](examples/) directory

### Community

- **Issues**: https://github.com/Sir-Teo/Symmetrica/issues
- **Discussions**: https://github.com/Sir-Teo/Symmetrica/discussions
- **Security**: See [SECURITY.md](SECURITY.md)

### Reporting Issues

When reporting migration issues, include:

1. **Version info**: Both old and new versions
2. **Minimal reproduction**: Smallest code that shows the issue
3. **Expected behavior**: What worked in 0.1.0
4. **Actual behavior**: What happens in 1.0.0
5. **Environment**: OS, Rust version, etc.

## Summary

**Migration from 0.1.0 to 1.0.0 is straightforward:**

✅ **No breaking changes**
✅ **Update dependencies and run tests**
✅ **Review new stability guarantees**
✅ **Enjoy long-term API stability**

The primary difference is that 1.0.0 comes with **stability commitments** that 0.1.0 didn't have. Your code will work the same, but now you have guarantees about future compatibility.

---

**Questions?** Open an issue or discussion on GitHub!

**Found a problem?** See [SECURITY.md](SECURITY.md) for security issues or [Issues](https://github.com/Sir-Teo/Symmetrica/issues) for bugs.

**Last Updated**: 2025-10-05
**Version**: 1.0.0
