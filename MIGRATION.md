# Migration Guide

This guide helps you migrate from 0.1.x to 1.1.x.

## What's New in 1.0.0 / 1.1.x
- Phase 2 advanced integration and Phase 2.5 symbolic simplification.
- Phase 6 advanced simplifier wiring.
- Foundations for special functions, Gröbner, summation.

## API Changes
- `simplify()` now wires in advanced trig/log/radical passes.
- Calculus tests compare simplified digests when appropriate.

## Migration Checklist
- Run the full CI gate locally.
- Update any tests that assert exact string forms of logs to compare simplified forms.

## Troubleshooting
- PyO3 linking: the `api` crate now builds as `rlib` by default. Enable the `python` feature and `cdylib` mode in downstream packaging when needed.

```rust
// Before
let result = integrate(&mut st, f, "x");
assert_eq!(st.to_string(result.unwrap()), "ln(2+x) + ln(1+x)");

// After (simplifier may contract logs)
let int = integrate(&mut st, f, "x").unwrap();
let int_s = simplify(&mut st, int);
let expected = simplify(&mut st, st.add(vec![ln1x, ln2x]));
assert_eq!(st.get(int_s).digest, st.get(expected).digest);
```
