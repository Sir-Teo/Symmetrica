# Migration Guide

This guide helps you migrate from Symmetrica 0.1.x pre-releases to 1.0.0 and beyond.

## What's New in 1.0.0

- Stable core APIs across crates: `expr_core`, `simplify`, `polys`, `calculus`, `matrix`, `solver`, `pattern`, `assumptions`, `io`, `evalf`
- Deterministic canonical constructors and digests
- Comprehensive testing, fuzzing, and differential test harnesses
- Documentation overhaul and roadmap consolidation

## API Changes

- Canonical constructors remain stable: `Store::add`, `Store::mul`, `Store::pow`
- Integration API: `calculus::integrate` returns `Option<ExprId>` (returns `None` when unsupported)
- Polynomial conversions stabilized: `expr_to_unipoly`, `unipoly_to_expr`
- Solver: explicit return types for roots; improved error handling via `Result`

### Breaking Changes Since 0.1.x

- Function names and module paths normalized for consistency
- Some error types consolidated into unified error enums
- CLI flags renamed for clarity (see `docs/cli.md`)

## Migration Checklist

- [ ] Review your usage of `integrate()`: handle `None` explicitly
- [ ] Replace any deprecated CLI flags following `docs/cli.md`
- [ ] Update imports to new module paths where applicable
- [ ] Rebuild and run tests locally (`cargo test --workspace`)
- [ ] Regenerate docs (`cargo doc --workspace --no-deps`)

## Troubleshooting

- Integration returns `None` unexpectedly
  - Ensure input is simplified (`simplify`), check for unsupported patterns (Risch not yet implemented)
- Different results after upgrade
  - Verify assumptions context; guarded simplifications may change outputs
- Performance regressions
  - Run benchmarks; open an issue with minimal repro and expression size

## Code Examples

```rust
use expr_core::Store;
use calculus::{diff, integrate};

let mut st = Store::new();
let x = st.sym("x");
let expr = st.pow(x, st.int(3));

let df = diff(&mut st, expr, "x"); // 3x^2
let maybe_int = integrate(&mut st, df, "x");
assert!(maybe_int.is_some());
```

```rust
use polys::{expr_to_unipoly, unipoly_to_expr};
use expr_core::Store;

let mut st = Store::new();
let x = st.sym("x");
let poly_expr = st.add(vec![st.pow(x, st.int(2)), st.mul(vec![st.int(3), x]), st.int(2)]);
let p = expr_to_unipoly(&st, poly_expr, "x").unwrap();
let back = unipoly_to_expr(&mut st, &p);
```

## Version References

This guide covers migration from 0.1.x to 1.0.0.
