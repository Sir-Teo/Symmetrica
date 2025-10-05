# Coverage Improvements Plan

This document tracks test coverage status and planned improvements across Symmetrica crates.

## Current Snapshot

- Overall coverage: 87.98%
- Target minimum (CI): 80% (tarpaulin --fail-under 80)
- Target goal (project): ≥ 85% for all core crates

## Priorities

- calculus: increase tests for integration edge cases and series expansion
- polys: extend tests for resultants/discriminants corner cases
- matrix: add property tests for RREF and determinant invariants
- solver: add more quartic edge cases and failure modes
- simplify: expand assumption-guarded transformations (ln rules, positivity)

## Action Items

- [ ] Add property-based tests (proptest) for polynomial GCD roundtrips
- [ ] Add differential tests comparing integrate+diff identities
- [ ] Fuzz parsers and simplifier inputs with AFL/libFuzzer harnesses
- [ ] Expand examples into doc-tests to boost coverage passively

## Methodology

- Use `cargo tarpaulin` in CI with LLVM engine
- Track per-crate coverage deltas on PRs
- Block merges that reduce coverage by >2% without justification

## Notes

- See `.github/workflows/ci.yml` for coverage configuration
- Benchmarks are not counted in coverage but must not regress
