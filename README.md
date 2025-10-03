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
- `crates/cli`: tiny demonstration CLI.
- Stubs: `polys`, `calculus`, `solver`, `matrix`, `assumptions`, `api`, `io`.

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

## Licensing

Licensed under either of
- Apache License, Version 2.0 (``LICENSE-APACHE``)
- MIT license (``LICENSE-MIT``)

at your option.
