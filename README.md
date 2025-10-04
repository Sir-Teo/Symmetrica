# Symmetrica
[![CI](https://github.com/Sir-Teo/Symmetrica/actions/workflows/ci.yml/badge.svg)](https://github.com/Sir-Teo/Symmetrica/actions/workflows/ci.yml)

A lightweight, embeddable symbolic computation engine (CAS) in Rust for symbolic mathematics, calculus, and equation solving.

## Key Features

- **Symbolic Computation**: Expression manipulation with automatic simplification
- **Calculus**: Differentiation, integration, series expansion, and limits
- **Equation Solving**: Univariate polynomial solving with exact rational and symbolic roots
- **Linear Algebra**: Exact matrix operations over rational numbers
- **Plotting**: SVG visualization of mathematical functions
- **Multiple Formats**: LaTeX, JSON, and S-expression I/O

## Core Principles

- **Immutable DAG**: Hash-consed expression trees for structural sharing
- **Canonical Forms**: Automatic normalization of `Add`, `Mul`, and `Pow`
- **Deterministic**: Stable ordering and digests for reproducible results
- **Exact Arithmetic**: Rational numbers (no floating-point errors)
- **Modular Design**: Minimal kernel with optional feature crates
- **Zero Dependencies**: Core functionality uses only Rust stdlib

## Workspace Layout

### Core Modules

- **`expr_core`**: Expression kernel with immutable DAG, hash-consing, and canonical constructors
- **`arith`**: Small rational arithmetic (i64) and GCD utilities
- **`simplify`**: Algebraic simplification passes (like-term collection, power merging)
- **`pattern`**: Symbol substitution and pattern matching

### Mathematical Modules

- **`calculus`**: Differentiation, integration, Maclaurin series, and limits
- **`polys`**: Univariate polynomials over Q with division, GCD, and partial fractions
- **`solver`**: Univariate polynomial equation solving
- **`matrix`**: Exact linear algebra over rationals (determinants, linear systems)
- **`assumptions`**: Tri-valued logic for assumption-guarded transformations

### I/O and Applications

- **`io`**: LaTeX, JSON, and S-expression serialization/parsing
- **`evalf`**: Numeric evaluation with f64 (arbitrary precision planned)
- **`plot`**: SVG plotting with numerical evaluation
- **`cli`**: Command-line interface (matika_cli)
- **`api`**: Python bindings via PyO3 with feature flag
- **`wasm`**: WebAssembly bindings via wasm-bindgen for browser/Node.js
- **`tests_e2e`**: End-to-end integration tests

## Quickstart

 Build everything:

 ```bash
 cargo build --workspace
 ```

 Run tests:

 ```bash
 cargo test --workspace
 ```

 Run the demo CLI:

 ```bash
 cargo run -p matika_cli
 ```

## Features

 - Immutable DAG with hash-consing in `crates/expr_core` for structural sharing.
 - Canonical constructors for `Add`/`Mul`/`Pow`; deterministic ordering and stable digests.
 - Small rational arithmetic (i64) and precedence-aware pretty printer via `Store::to_string()`.
 - Simplifier (`crates/simplify`): constant folding, like-term/factor collection, rational normalization, and power-merge in products.
 - Calculus (`crates/calculus`):
   - `diff()` for `Add`/`Mul`/`Pow` with integer exponents, common funcs (`sin`/`cos`/`exp`/`ln`) with chain rule, and general power rule `d(u^v)`.
   - `integrate()` conservative rules: constants/symbols, power rule (incl. `1/x = ln(x)`), linear `exp(ax+b)`/`sin(ax+b)`/`cos(ax+b)`, and `u'/u -> ln(u)`.
   - Maclaurin series for `exp`/`sin`/`cos`/`ln(1+z)` with composition; simple polynomial limits at `0` and `+∞`.
 - Pattern/substitution (`crates/pattern`): `subst_symbol()` for safe symbol replacement.
 - Polynomials (`crates/polys`): univariate dense over Q; division, GCD; `expr_to_unipoly()` and `unipoly_to_expr()` conversions.
## Usage Examples

### Build and Simplify Expressions

```rust
use expr_core::Store;
use simplify::simplify;

let mut st = Store::new();
let x = st.sym("x");

// Build expression: x^2 + 3x + 1
let expr = st.add(vec![
    st.pow(x, st.int(2)),
    st.mul(vec![st.int(3), x]),
    st.int(1),
]);

// Simplify and print
let simplified = simplify(&mut st, expr);
println!("{}", st.to_string(simplified));  // Output: 1 + 3 * x + x^2
```

### Differentiation

```rust
use calculus::diff;

// Differentiate x^3 + 2x
let expr = st.add(vec![
    st.pow(x, st.int(3)),
    st.mul(vec![st.int(2), x]),
]);

let derivative = diff(&mut st, expr, "x");
let simplified = simplify(&mut st, derivative);
println!("{}", st.to_string(simplified));  // Output: 2 + 3 * x^2
```

### Integration

```rust
use calculus::integrate;

// Integrate x^2
let x2 = st.pow(x, st.int(2));
let integral = integrate(&mut st, x2, "x").unwrap();
println!("{}", st.to_string(integral));  // Output: 1/3 * x^3

// Integrate 1/x
let inv_x = st.pow(x, st.int(-1));
let ln_x = integrate(&mut st, inv_x, "x").unwrap();
println!("{}", st.to_string(ln_x));  // Output: ln(x)
```

### Equation Solving

```rust
use solver::solve_univariate;

// Solve x^2 + 3x + 2 = 0
let eq = st.add(vec![
    st.pow(x, st.int(2)),
    st.mul(vec![st.int(3), x]),
    st.int(2),
]);

let roots = solve_univariate(&mut st, eq, "x").unwrap();
for root in roots {
    println!("x = {}", st.to_string(root));
}
// Output: x = -1
//         x = -2
```

### Pattern Substitution

```rust
use pattern::subst_symbol;

// Substitute x -> (y + 1) in x^2
let y = st.sym("y");
let y_plus_1 = st.add(vec![y, st.int(1)]);
let x2 = st.pow(x, st.int(2));

let result = subst_symbol(&mut st, x2, "x", y_plus_1);
let simplified = simplify(&mut st, result);
println!("{}", st.to_string(simplified));  // Output: (1 + y)^2
```

### Polynomial Conversion

```rust
use polys::{expr_to_unipoly, unipoly_to_expr};

// Convert expression to polynomial
let poly = expr_to_unipoly(&st, expr, "x").expect("valid polynomial");
println!("Degree: {:?}", poly.degree());

// Convert back to expression
let expr_back = unipoly_to_expr(&mut st, &poly);
assert_eq!(st.get(expr).digest, st.get(expr_back).digest);
```

### LaTeX Output

```rust
use io::to_latex;

let expr = st.pow(st.add(vec![x, st.int(1)]), st.int(2));
let latex = to_latex(&st, expr);
println!("{}", latex);  // Output: (1 + x)^{2}
```

For complete examples, see the `examples/` directory and `crates/cli/src/main.rs`.

## Local quality gates (what CI enforces)

CI is defined in `.github/workflows/ci.yml` and runs on Ubuntu/macOS/Windows:
 - Format: `cargo fmt --all -- --check`
 - Clippy: `cargo clippy --workspace --all-targets -- -D warnings`
 - Build: `cargo build --workspace --all-features`
 - Tests: `cargo test --workspace --all-features`
 - Docs: `cargo doc --workspace --no-deps`
 - Audit: `cargo audit`
 - Deny: `cargo deny check`
 - Coverage: `cargo tarpaulin -p expr_core -p simplify --engine llvm --fail-under 80`

Recommended to run locally before pushing:

```bash
rustup component add rustfmt clippy
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace --all-features
cargo test --workspace --all-features
cargo doc --workspace --no-deps
cargo install cargo-audit cargo-deny cargo-tarpaulin --locked
cargo audit
cargo deny check
cargo tarpaulin -p expr_core -p simplify --engine llvm --fail-under 80
```

Pre-commit hooks are configured in `.pre-commit-config.yaml`:

```bash
pipx install pre-commit  # or: pip install --user pre-commit
pre-commit install
pre-commit run --all-files
```

## Benchmarks

Micro-benchmarks live in `crates/expr_core/benches/expr_benches.rs` (Criterion).

```bash
cargo bench -p expr_core
# HTML report: target/criterion/report/index.html
```

## Documentation

### Module Documentation

Comprehensive guides for each crate in `docs/`:

- **[expr_core.md](docs/expr_core.md)**: Expression kernel, DAG, hash-consing
- **[arith.md](docs/arith.md)**: Rational arithmetic and GCD
- **[simplify.md](docs/simplify.md)**: Algebraic simplification passes
- **[calculus.md](docs/calculus.md)**: Differentiation, integration, series
- **[polys.md](docs/polys.md)**: Polynomial operations and conversions
- **[solver.md](docs/solver.md)**: Equation solving algorithms
- **[matrix.md](docs/matrix.md)**: Linear algebra over rationals
- **[pattern.md](docs/pattern.md)**: Pattern matching and substitution
- **[assumptions.md](docs/assumptions.md)**: Assumption system
- **[io.md](docs/io.md)**: Serialization formats (LaTeX, JSON, S-expr)
- **[evalf.md](docs/evalf.md)**: Numeric evaluation
- **[plot.md](docs/plot.md)**: SVG plotting
- **[cli.md](docs/cli.md)**: Command-line interface
- **[api.md](docs/api.md)**: Python bindings (PyO3)
- **[wasm.md](docs/wasm.md)**: WebAssembly bindings

### Design Documentation

- **[roadmap.md](docs/roadmap.md)**: Future development plans
- **[research.md](docs/research.md)**: Design notes and prior art
- **[skeleton.md](docs/skeleton.md)**: Initial architecture rationale

### Quality Assurance

- **[fuzzing.md](docs/fuzzing.md)**: Fuzz testing infrastructure (Phase L)
- **[property_testing.md](docs/property_testing.md)**: Property-based testing (Phase L)

### API Documentation

Generate API docs:
```bash
cargo doc --workspace --no-deps --open
```

## Release

Tagging `v*.*.*` triggers `.github/workflows/release.yml` to build and upload artifacts. If `matika_cli` is present it is packaged per-OS.

To cut a release locally:

```bash
git tag -a v0.1.0 -m "v0.1.0"
git push origin v0.1.0
```

## Contributing

- Follow the PR template in `.github/PULL_REQUEST_TEMPLATE.md`.
- Keep determinism: canonical forms and stable prints matter. Prefer adding tests in `expr_core`, `simplify`, and integration tests in `tests_e2e`.
- Run the local quality gates above before opening a PR.
- Licensing: dual MIT/Apache-2.0; contributions are accepted under the same terms.

## Licensing

Licensed under either of
 - Apache License, Version 2.0 (``LICENSE-APACHE``)
 - MIT license (``LICENSE-MIT``)

 at your option.
