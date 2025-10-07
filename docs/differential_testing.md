# Differential Testing

This document describes our approach to differential testing across calculus and simplification features.

- We validate integrals by differentiating results and comparing against the original integrand (`crates/calculus/src/lib.rs` tests).
- Trigonometric and hyperbolic identities are verified by structural simplification comparisons.
- We cross-compare results for selected expressions against reference CAS outputs in fuzzing targets.

See also:
- `docs/fuzzing.md`
- `docs/property_testing.md`
- `crates/tests_e2e/tests/differential_tests.rs`
