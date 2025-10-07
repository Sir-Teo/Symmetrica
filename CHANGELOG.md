# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Pending

### Changed
- Pending

### Fixed
- Pending
## [1.0.0-rc.2] - 2025-10-06
### Added
- Phase 3 numerics: `Gamma` (Lanczos + reflection) and `Ei` (series) with unit tests in `crates/special/`.
- Phase 4 Gröbner: `s_polynomial()` and a naive `reduce()` with unit tests in `crates/grobner/`.
- Phase 6 E2E tests: trig identities (product-to-sum, half-angle), log quotient expansion, radical rationalization.
- Phase 7 number theory: `is_prime_u64`, `mod_inverse`, `crt_pair`, `crt` APIs; experimental factorization (Pollard's rho Brent variant) behind feature.
- Phase 8 tensor algebra: `crates/tensor/` with `Tensor<T>` supporting reshape, permute_axes, transpose, matmul, contract, trace, sum_axis, outer product, and elementwise ops.
- Phase 9 scaffold: `crates/algebraic/` with `Quad` for quadratic extensions Q(√d) supporting Add/Sub/Mul/Neg, conjugate, and norm.
- Docs: per-crate READMEs for `number_theory/`, `special/`, `grobner/`, `tensor/`, and `algebraic/`; doc examples for public APIs.

### Changed
- Promoted `crt_pair()` and `crt()` to stable in `crates/number_theory/` (no feature required).
- Satisfied `cargo-deny` by adding `license`/`description` to `special/` and `summation/`.
- Increased coverage (~79.11%); HTML at `tarpaulin-report.html`.
### Fixed
- Clippy: precision/grouping in `Gamma` coefficients, boolean comparisons, misplaced test attributes, and minor style issues.
- Gröbner `reduce()`: nested `Add` term handling and monomial divisor special-case to ensure structural remainder.

## [1.1.0] - 2025-10-06
### Added
- Phase 2.5 symbolic simplification
- Phase 2 advanced integrators

[Unreleased]: https://keepachangelog.com/en/1.0.0/
