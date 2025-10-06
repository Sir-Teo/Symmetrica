# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog (https://keepachangelog.com/en/1.0.0/) and this project adheres to Semantic Versioning.

## [Unreleased]

## [1.1.0] - In Progress
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
