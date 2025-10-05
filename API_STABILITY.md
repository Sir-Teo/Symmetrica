# API Stability Policy

This document defines Symmetrica's stability guarantees and the process for introducing changes to the public API.

## Semantic Versioning

Symmetrica follows [Semantic Versioning](https://semver.org/) strictly:
- MAJOR version (X.0.0) for incompatible API changes
- MINOR version (0.Y.0) for added functionality in a backward compatible manner
- PATCH version (0.0.Z) for backward compatible bug fixes

## 1.0 Stability Guarantees

Starting from 1.0.0, the following are guaranteed stable within a major version:
- Public types and functions in `expr_core`, `simplify`, `polys`, `calculus`, `matrix`, `solver`, `pattern`, `assumptions`, `io`, `evalf`
- Behavior of canonical constructors (Add/Mul/Pow) and digests
- Deterministic output for equivalent expressions
- Panic-free operation on valid inputs

## Breaking Change Policy

- Breaking changes are only allowed in MAJOR releases
- Deprecations will be announced and maintained for at least two MINOR versions before removal
- API changes require an RFC and two approvals
- Migration guides will be provided for breaking changes

## Mathematical Correctness Guarantees

- Deterministic canonical forms for algebraic operations
- Verified differentiation rules (power, product, chain)
- Conservative integration (returns `None` if not confidently supported)
- Exact rational arithmetic in `arith`
- Verified polynomial operations (GCD, resultants, discriminants)

## Backward Compatibility Commitments

- Minor/patch releases will not break existing public APIs
- Feature flags will not change default behavior in a breaking way
- Internal refactors must preserve public API contracts
