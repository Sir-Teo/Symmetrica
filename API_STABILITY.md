# API Stability and 1.0 Commitment

This document defines the API stability guarantees for Symmetrica 1.0 and beyond.

## Semantic Versioning

Symmetrica follows [Semantic Versioning 2.0.0](https://semver.org/):

- **MAJOR version** (X.0.0) - Incompatible API changes
- **MINOR version** (0.X.0) - Backwards-compatible functionality additions
- **PATCH version** (0.0.X) - Backwards-compatible bug fixes

## 1.0 Stability Guarantees

### Public API Stability

Once Symmetrica reaches 1.0.0, the following are **guaranteed stable**:

#### Core Types (`expr_core`)
- ✅ `Store` - Expression store API
- ✅ `ExprId` - Expression identifier (opaque handle)
- ✅ `Op` - Operation enum variants
- ✅ `Payload` - Payload enum variants

**Stable Methods:**
```rust
// Store construction and basic operations
Store::new()
Store::sym(name)
Store::int(value)
Store::rat(num, den)
Store::add(children)
Store::mul(children)
Store::pow(base, exp)
Store::func(name, args)
Store::get(id)
```

#### Simplification (`simplify`)
- ✅ `simplify(store, expr)` - Main simplification entry point
- ✅ Idempotent behavior guarantee
- ✅ Algebraic correctness guarantees

#### Calculus (`calculus`)
- ✅ `diff(store, expr, var)` - Differentiation
- ✅ `integrate(store, expr, var)` - Integration (partial)
- ✅ Derivative rules (power, product, chain)

#### Polynomials (`polys`)
- ✅ `UniPoly` - Univariate polynomial type
- ✅ `MultiPoly` - Multivariate polynomial type
- ✅ `expr_to_unipoly` - Conversion from expressions
- ✅ `unipoly_to_expr` - Conversion to expressions
- ✅ Arithmetic operations (add, mul, div_rem, gcd)
- ✅ Advanced operations (factor, resultant, discriminant)

#### Linear Algebra (`matrix`)
- ✅ `MatrixQ` - Rational matrix type
- ✅ `MatrixQ::new`, `from_i64`, `identity`
- ✅ Arithmetic (add, sub, mul, transpose, scalar_mul)
- ✅ `det_bareiss` - Determinant computation
- ✅ `solve_bareiss`, `solve_lu` - System solving
- ✅ `lu_decompose`, `inverse`
- ✅ `rank`, `nullspace`, `columnspace`

#### Solver (`solver`)
- ✅ `solve(store, expr, var)` - Main solving interface
- ✅ Support for polynomial equations (linear through quartic)
- ✅ Basic exponential equation solving

#### I/O (`io`)
- ✅ `to_sexpr(store, expr)` - S-expression serialization
- ✅ `from_sexpr(store, input)` - S-expression parsing
- ✅ `to_json(store, expr)` - JSON serialization
- ✅ `from_json(store, input)` - JSON parsing
- ✅ `to_latex(store, expr)` - LaTeX output

#### Pattern Matching (`pattern`)
- ✅ Pattern matching and substitution API
- ✅ Wild pattern support
- ✅ Multi-pattern matching

#### Assumptions (`assumptions`)
- ✅ `AssumeCtx` - Assumption context
- ✅ Assumption types (Positive, Nonnegative, Real, Integer, etc.)
- ✅ Three-valued logic (True/False/Unknown)

### Stability Levels

#### Stable ✅
- Core expression building (Store API)
- Simplification (idempotent, algebraic correctness)
- Differentiation (all standard rules)
- Polynomial operations (univariate and multivariate)
- Matrix operations (exact arithmetic over ℚ)
- Basic solving (polynomial equations)
- I/O formats (S-expr, JSON, LaTeX)
- Pattern matching v1
- Assumptions v1

#### Evolving 🔄
- Integration (partial implementation, will expand)
- Series expansion (stub, will be enhanced)
- Special functions (limited set, will expand)
- Solver (will add more equation types)

#### Experimental 🧪
- Python bindings (`api`) - API may evolve
- WASM bindings (`wasm`) - API may evolve
- Plotting (`plot`) - May add more backends

## Breaking Change Policy

### What Constitutes a Breaking Change

Breaking changes require a **major version bump**:

1. **Removing public types, traits, or functions**
2. **Changing function signatures** (parameters, return types)
3. **Changing behavior** that users depend on
4. **Removing or renaming public fields**
5. **Changing error types** in non-`Result`-returning functions

### What is NOT a Breaking Change

The following do **not** constitute breaking changes:

1. **Adding new public APIs** (functions, methods, types)
2. **Adding new trait implementations** to existing types
3. **Fixing bugs** that result in incorrect mathematical results
4. **Performance improvements** (even if they change complexity)
5. **Adding new optional parameters** via builder patterns
6. **Internal implementation changes** that don't affect public API
7. **Documentation improvements**

### Deprecation Policy

Before removing APIs in a major version:

1. Mark as `#[deprecated]` in a minor release
2. Provide migration path in deprecation message
3. Keep deprecated API for at least **2 minor versions** or **6 months**
4. Remove in next major version

Example:
```rust
#[deprecated(since = "1.2.0", note = "Use `new_api()` instead")]
pub fn old_api() { ... }
```

## Mathematical Correctness Guarantees

### Core Guarantees (1.0+)

1. **Simplification is idempotent**
   - `simplify(simplify(expr)) == simplify(expr)`

2. **Algebraic equivalence preservation**
   - Simplification never changes mathematical meaning
   - Transformations are sound

3. **Derivative correctness**
   - All derivative rules are mathematically correct
   - Chain rule, product rule, quotient rule verified

4. **Polynomial arithmetic**
   - GCD algorithm is correct (Euclidean)
   - Factorization finds all rational roots
   - Resultant and discriminant formulas are correct

5. **Matrix operations**
   - Determinant computation is exact
   - Linear system solving is exact (no floating-point errors)
   - All algorithms are fraction-free or use exact rationals

### Bug Fix Exception

If a bug is found that violates mathematical correctness:
- **It will be fixed immediately**, even if it changes behavior
- This is **not considered a breaking change**
- Users should not depend on incorrect mathematical results

## Feature Flags

### Stable Feature Flags (1.0+)

- `default` - Default feature set
- `python` - Python bindings (evolving)
- `wasm` - WebAssembly support (evolving)

### Adding New Features

New feature flags can be added in **minor versions**.
Existing feature flags won't be removed without deprecation.

## Minimum Supported Rust Version (MSRV)

- **Current MSRV:** Rust 1.70.0
- **MSRV Policy:** 
  - Can be increased in **minor versions**
  - Will maintain MSRV for at least **6 months** after a Rust release
  - MSRV increases will be documented in CHANGELOG

## Crate Structure Stability

### Workspace Crates

The following crate structure is **stable** for 1.0:

- `expr_core` - Core expression system
- `arith` - Rational arithmetic
- `simplify` - Simplification
- `calculus` - Differentiation and integration
- `polys` - Polynomial operations
- `matrix` - Linear algebra
- `solver` - Equation solving
- `pattern` - Pattern matching
- `assumptions` - Assumption system
- `io` - Serialization and parsing
- `evalf` - Numeric evaluation
- `plot` - Plotting utilities

### Internal Crates

The following are **internal** and may change:

- `tests_e2e` - Integration tests
- Individual benchmark crates

## Commitment to Users

### What We Promise

1. **Stability** - Your code won't break on minor/patch updates
2. **Correctness** - Mathematical operations are correct
3. **Performance** - We won't regress performance without cause
4. **Documentation** - All public APIs are documented
5. **Migration Help** - Clear upgrade paths for major versions

### What We Ask

1. **Test your code** - Run tests before upgrading
2. **Report issues** - Help us find bugs early
3. **Use stable APIs** - Avoid depending on internal implementation details
4. **Read changelogs** - Stay informed about changes

## Version Timeline

### Current: 0.1.0 (Pre-1.0)

- All APIs are **subject to change**
- Breaking changes can occur in minor versions
- Use for evaluation and testing

### Planned: 1.0.0-rc.1

- Feature freeze for 1.0 APIs
- Only bug fixes and documentation
- Minimum 4-week RC period
- Community feedback integration

### Target: 1.0.0

- Full API stability commitment
- Semantic versioning enforcement
- Long-term support

## Contact and Feedback

- **Issues:** https://github.com/Sir-Teo/Symmetrica/issues
- **Discussions:** https://github.com/Sir-Teo/Symmetrica/discussions
- **Security:** See SECURITY.md (to be created)

## References

- [Semantic Versioning 2.0.0](https://semver.org/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Cargo SemVer Compatibility](https://doc.rust-lang.org/cargo/reference/semver.html)
