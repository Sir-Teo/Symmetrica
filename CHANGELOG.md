# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Gröbner basis computation with Buchberger's algorithm
- Symbolic tensor operations with Einstein notation
- Special functions (Bessel, Gamma, Error functions)
- Radical denesting for algebraic simplifications
- Number theory primitives (GCD, primitive roots, Diophantine equations)

### Fixed
- Gröbner solver performance issues with simplification
- O(n) performance bug in primitive_root factorization

## [1.0.0-rc.2] - 2025-10-10

### Added
- Complete Phase 9 (Algebraic) to 60% with radical denesting
- Complete Phase 8 (Tensor) to 60% with symbolic tensors
- Complete Phase 4 (ODE) to 80% with exact equations

### Changed
- Improved Gröbner basis algorithm with better zero detection
- Enhanced simplification throughout algebraic operations

### Fixed
- Solve_system performance and correctness issues
- Buchberger algorithm infinite loop prevention

## [1.0.0-rc.1] - Initial Release Candidate

### Added
- Core expression system with hash consing
- Symbolic differentiation and integration
- Matrix operations and linear algebra
- Polynomial operations and GCD
- Pattern matching and rewriting
- LaTeX and JSON output formats
- Assumptions system for symbolic reasoning
- Plotting capabilities
- Comprehensive test suite

[Unreleased]: https://github.com/Sir-Teo/Symmetrica/compare/v1.0.0-rc.2...HEAD
[1.0.0-rc.2]: https://github.com/Sir-Teo/Symmetrica/compare/v1.0.0-rc.1...v1.0.0-rc.2
[1.0.0-rc.1]: https://github.com/Sir-Teo/Symmetrica/releases/tag/v1.0.0-rc.1
