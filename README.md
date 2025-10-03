# Symmetrica

A lightweight, embeddable symbolic computation engine (CAS) in Rust.
Core principles:
- Immutable DAG + hash-consing; canonical `Add`/`Mul`/`Pow`.
- Default simplifier: constant folding, like-term/factor collection, rational normalization.
- Deterministic ordering and stable digests; resource guards (planned).

- Minimal kernel; heavier features live in separate crates.

## Workspace layout

- `crates/expr_core`: expression kernel (immutable DAG + interner) with canonical constructors and small rational helpers.
- `crates/simplify`: explicit simplification passes; `simplify::simplify(&mut Store, id)`.
- `crates/pattern`: basic substitution utilities.
- `crates/polys`: univariate dense polynomials (Q), division/GCD, expr↔poly conversions.
- `crates/calculus`: differentiation for Add/Mul/Pow with integer exponents.
- `crates/cli`: tiny demonstration CLI.
- Skeleton/stubs: `solver`, `matrix`, `assumptions`, `api`, `io`.

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
 - Calculus (`crates/calculus`): `diff()` for `Add`/`Mul`/`Pow` with integer exponents, plus `sin`/`cos`/`exp`/`ln` with chain rule; product rule; auto-simplifies results.
 - Pattern/substitution (`crates/pattern`): `subst_symbol()` for safe symbol replacement.
 - Polynomials (`crates/polys`): univariate dense over Q; division, GCD; `expr_to_unipoly()` and `unipoly_to_expr()` conversions.
 - Modular crates for future domains: `matrix`, `solver`, `assumptions`, `api`, `io`.
 - Cross-platform CI with fmt, clippy, build, tests, docs, audit, deny, and coverage.

## Architecture & crates

 - `crates/expr_core` — expression kernel: `Store`, `ExprId`, `Op`, `Payload`, canonical `add()/mul()/pow()`, `to_string()`.
 - `crates/simplify` — `simplify()` and `simplify_with()` using `assumptions::Context`; collects like terms and merges powers.
 - `crates/calculus` — `diff(&mut Store, ExprId, &str)` implements linear, product, and integer power rules.
 - `crates/pattern` — `subst_symbol(&mut Store, ExprId, sym, with)` tree substitution.
 - `crates/polys` — `UniPoly`, `expr_to_unipoly()`, `unipoly_to_expr()`; Euclidean `div_rem()` and `gcd()`.
 - `crates/assumptions` — 3‑valued `Truth` and `Context` (skeleton) used by `simplify`.
 - `crates/matrix`, `crates/solver`, `crates/api`, `crates/io` — stubs for upcoming features.
 - `crates/cli` — demo binary `matika_cli` showing building/printing/simplifying expressions.
 - `crates/tests_e2e` — integration tests scaffold.

## Usage examples

 Build and simplify an expression:

 ```rust
 use expr_core::Store;
 use simplify::simplify;

 let mut st = Store::new();
 let x = st.sym("x");
 let expr = st.add(vec![
     st.pow(x, st.int(2)),
     st.mul(vec![st.int(3), x]),
     st.int(1),
 ]);
 let s = simplify(&mut st, expr);
 println!("{}", st.to_string(s));
 ```

 Differentiate with respect to `x`:

 ```rust
 use calculus::diff;
 let df = diff(&mut st, s, "x");
 println!("{}", st.to_string(df));
 ```

 Substitute `x -> (y+1)`:

 ```rust
 use pattern::subst_symbol;
 let y = st.sym("y");
 let y1 = st.add(vec![y, st.int(1)]);
 let replaced = subst_symbol(&mut st, s, "x", y1);
 println!("{}", st.to_string(replaced));
 ```

 Convert to/from univariate polynomial over Q:

 ```rust
 use polys::{expr_to_unipoly, unipoly_to_expr};
 let p = expr_to_unipoly(&st, s, "x").expect("convertible to UniPoly");
 let s_back = unipoly_to_expr(&mut st, &p);
 assert_eq!(st.get(s_back).digest, st.get(s).digest);
 ```

 See `crates/cli/src/main.rs` for a runnable end-to-end example.

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

## Docs & roadmap

- API docs: `cargo doc --workspace --no-deps --open`
- roadmap: `docs/roadmap.md`
- Design notes and prior art: `docs/research.md`
- Initial skeleton and rationale: `docs/skeleton.md`

## Release

Tagging `v*.*.*` triggers `.github/workflows/release.yml` to build and upload artifacts. If `matika_cli` is present it is packaged per-OS.

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
