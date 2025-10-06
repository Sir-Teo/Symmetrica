# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.0.0] - 2025-10-06
### Added - Phase 6: Enhanced Simplification
- **Trigonometric Identities Module** (`simplify/src/trig_identities.rs`):
  - Product-to-sum: `sin(A)cos(B) → [sin(A+B) + sin(A-B)]/2`
  - Sum-to-product: `sin(A) + sin(B) → 2sin((A+B)/2)cos((A-B)/2)`
  - Half-angle formulas: `sin²(x/2) → (1 - cos(x))/2`
  - 30 comprehensive tests (14 unit + 16 integration)
- **Radical Simplification Module** (`simplify/src/radical_simplify.rs`):
  - Perfect square detection: `√4 → 2`, `√(4/9) → 2/3`
  - Perfect power simplification: `√(x⁴) → x²`
  - Factoring: `√(4x) → 2√x`
  - Ramanujan's denesting algorithm for nested radicals
  - Denominator rationalization: `1/√x → √x/x`
  - 17 comprehensive tests (5 unit + 12 integration)
- **Logarithm Expansion Module** (`simplify/src/log_simplify.rs`):
  - Product rule: `log(xy) → log(x) + log(y)` [assumption-guarded]
  - Power rule: `log(x^n) → n·log(x)` [assumption-guarded]
  - Quotient rule: `log(x/y) → log(x) - log(y)` [assumption-guarded]
  - Logarithm contraction: `log(x) + log(y) → log(xy)`
  - Branch-cut aware transformations with assumptions context
  - 18 comprehensive tests (5 unit + 13 integration)
- **Phase 6 Summary**:
  - 3 new modules (~1200 lines production code)
  - 104 total tests in simplify crate (100% passing)
  - Public API: `simplify_trig`, `simplify_radicals`, `simplify_logarithms`, `contract_logarithms`

### Added - Phase 5: Symbolic Summation (Completed)
- **Infinite Products Module** (`summation/src/products.rs`):
  - Finite product evaluation: `∏(k=1..n) k = n!`
  - Geometric products: `∏(k=0..n) r^k = r^(n(n+1)/2)`
  - Gamma function connection: `∏(k=0..n-1) (x+k) = Γ(x+n)/Γ(x)`
  - 3 comprehensive tests

## [1.4.0] - 2025-10-06 - Phase 5 Complete
### Added
- **Symbolic Summation Complete** (`crates/summation`):
  - Gosper's algorithm for hypergeometric summation
  - Zeilberger's algorithm for creative telescoping
  - Basic sums: arithmetic, geometric, power sums
  - Convergence tests: ratio test
  - Pochhammer symbols and factorial operations
  - Infinite products with Gamma function connections
  - 59 comprehensive tests across all modules

## [1.1.0] - 2025-10-05
### Added
- **Hyperbolic Functions (v1.1):**
  - Differentiation: `d/dx sinh(u) = cosh(u) * u'`, `d/dx cosh(u) = sinh(u) * u'`, `d/dx tanh(u) = (1 - tanh²(u)) * u'`
  - Integration: `∫ sinh(ax+b) dx`, `∫ cosh(ax+b) dx`, `∫ tanh(ax+b) dx = ln(cosh(ax+b))/a`
  - Comprehensive test suite with differential verification
- **Trigonometric Power Patterns (v1.1):**
  - `∫ sin(x)cos(x) dx = -cos(2x)/4` using product-to-sum identity
  - `∫ sin²(x) dx = x/2 - sin(2x)/4` using double-angle formula
  - `∫ cos²(x) dx = x/2 + sin(2x)/4` using double-angle formula
  - Pattern recognition for trigonometric products in integration
- **U-Substitution Pattern Detection (v1.1):**
  - Automatic detection of `∫ f(g(x)) * g'(x) dx` patterns
  - Handles `∫ u^n * u' dx` with rational coefficient adjustment
  - Supports nested polynomials, composite expressions, and negative powers
  - Verified with differential check for all test cases
- **Risch Algorithm Foundation (v1.1):**
  - New `risch` module with differential field tower representation
  - Tower extension detection for exp/log structures (`ExtensionType` enum)
  - Logarithmic derivative computation: `d/dx(ln(f)) = f'/f`
  - Helper functions: `is_exponential()`, `is_logarithm()`, `detect_extension()`
  - Enhanced exponential integration via Risch framework
  - Lays groundwork for full Risch algorithm implementation
- **Weierstrass Substitution (v1.1):**
  - Tangent half-angle substitution for rational trigonometric integrals
  - `∫ 1/(1 + cos(x)) dx = tan(x/2) + C`
  - `∫ 1/(1 - cos(x)) dx = -cot(x/2) + C`
  - Pattern recognition for denominators with cosine terms
  - Helper functions: `is_simple_cos()`, `is_negative_cos()`
  - Handles coefficient scaling automatically
- **Comprehensive Test Suite:**
  - 144 total tests in calculus crate (121 lib + 19 integration_v1_1 + 4 prop)
  - 12 new Risch algorithm tests for tower detection and logarithmic derivatives
  - 5 new Weierstrass substitution tests for rational trig integrals
  - 19 integration_v1_1 tests covering standard integrals, hyperbolic functions, trigonometric patterns, u-substitution, mixed patterns, and edge cases
  - All new tests verified via differential check or structural validation

### Changed
- Integration engine priority: u-substitution → trig patterns → integration by parts → partial fractions
- Documentation extensively updated with v1.1 feature examples and usage patterns in `docs/calculus.md`

### Fixed
- Borrow checker issues in trigonometric pattern matching
- Clippy warning for needless range loop in u-substitution

## [1.0.0-rc.1] - 2025-10-01
### Added
- Roadmap consolidated and enhanced with detailed design principles and checklists.

### Changed
- Documentation cleanup and reorganization.

### Fixed
- Documentation tests updated to reflect cleaned files.

## [1.0.0-rc.1] - 2025-10-01
### Added
- Foundation complete: expression kernel, simplification, calculus, polys, matrix, solver, pattern, assumptions, io, evalf, plot, cli, api, wasm.
- Testing, fuzzing, and benchmark infrastructure.
