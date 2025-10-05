# Changelog

All notable changes to Symmetrica will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0-rc.1] - 2025-10-05

### Release Candidate 1

This is the first release candidate for Symmetrica 1.0.0. All core features are complete and stable.

#### What's New
- **API Stability Guarantees**: Semantic versioning commitment documented in `API_STABILITY.md`
- **Migration Guide**: Complete migration documentation in `MIGRATION.md`
- **Security Policy**: Vulnerability reporting process in `SECURITY.md`
- **Production Ready**: All 704 tests passing, 81.91% code coverage

#### Version Bump
- All crates bumped from `0.1.0` to `1.0.0-rc.1`

#### RC Period
- **Duration**: Minimum 4 weeks
- **Focus**: Bug fixes only, no new features
- **Goal**: Community feedback and real-world validation

#### Next Steps
- Community testing and feedback
- Bug fixes if needed
- Final 1.0.0 release after RC period

## [0.1.0] - 2024-10-05

### Added - Core Expression System (Phase A)

#### `expr_core` - Expression DAG with Hash Consing
- Immutable expression DAG with arena allocation
- Hash-consing for structural sharing and O(1) equality
- Canonical constructors for Add/Mul/Pow operations
- Support for:
  - Integer and Rational constants
  - Symbols and variables
  - n-ary Add and Mul operations
  - Binary Pow operations
  - Functions (sin, cos, exp, ln, etc.)
- Expression digests for fast equality checks
- Comprehensive test coverage (100+ unit tests)
- Performance benchmarks for all core operations

#### `arith` - Rational Arithmetic
- Small rationals over i64 with automatic normalization
- GCD-based canonical form
- Zero-copy operations where possible
- Overflow detection and handling

### Added - Algebraic Simplification (Phase B)

#### `simplify` - Algebraic Simplification Engine
- Combine like terms in Add/Mul
- Rational arithmetic normalization
- Power law simplification (x^a * x^b → x^(a+b))
- Idempotent simplification guaranteed
- Identity elimination (x+0 → x, x*1 → x, x^1 → x)
- Zero propagation (x*0 → 0)
- Domain-aware logarithm and exponential rules
- Support for assumptions in simplification
- Property-based tests for algebraic laws
- Comprehensive benchmarks

### Added - Polynomials (Phase C)

#### `polys` - Polynomial Operations
- **Univariate Dense Polynomials:**
  - Arithmetic (add, sub, mul, div_rem)
  - Euclidean GCD algorithm
  - Derivative computation
  - Evaluation (Horner's method)
  - Monic normalization
- **Advanced Polynomial Algorithms:**
  - Factorization (rational root theorem)
  - Square-free decomposition
  - Resultant computation (Sylvester matrix)
  - Discriminant calculation
  - Partial fractions (simple cases)
- **Multivariate Sparse Polynomials:**
  - Monomial representation (BTreeMap)
  - Add, sub, mul operations
  - Total degree computation
  - Evaluation at points
- **Expr ⟷ Poly Conversions:**
  - Expression to polynomial conversion
  - Polynomial to expression conversion
  - Roundtrip preservation
- 93 unit tests + 23 benchmark correctness tests
- Comprehensive performance benchmarks (16 benchmark functions)

### Added - Calculus (Phase D)

#### `calculus` - Differentiation and Integration
- **Differentiation:**
  - Power rule, product rule, chain rule
  - Trigonometric derivatives (sin, cos)
  - Exponential and logarithm derivatives
  - Automatic simplification of results
- **Integration:**
  - Power rule for polynomials
  - Exponential integration
  - Trigonometric integrals (sin, cos)
  - f'/f pattern recognition
  - Rational function integration
- **Series Expansion:**
  - Taylor series (stub)
  - Limit computation (stub)
- 50+ unit tests
- Property tests for calculus rules
- Performance benchmarks

### Added - Linear Algebra (Phase E)

#### `matrix` - Exact Linear Algebra over ℚ
- **Matrix Arithmetic:**
  - Addition, subtraction, multiplication
  - Transpose, scalar multiplication
  - Trace computation
- **Determinant:**
  - Bareiss fraction-free algorithm
  - Efficient for exact rational computation
- **Linear Systems:**
  - Cramer's rule (solve_bareiss)
  - LU decomposition with partial pivoting
  - LU-based solving (solve_lu)
- **Matrix Decomposition:**
  - LU factorization
  - Matrix inverse (Gauss-Jordan)
- **Subspace Computations:**
  - Rank computation
  - Nullspace basis
  - Column space basis
- 122 unit tests + 35 benchmark correctness tests
- Comprehensive performance benchmarks (23 benchmark functions)

### Added - Equation Solving (Phase F)

#### `solver` - Algebraic Equation Solving
- **Polynomial Solving:**
  - Linear equations
  - Quadratic formula
  - Cubic formula (Cardano)
  - Quartic formula (Ferrari)
  - Polynomial factorization approach
- **Exponential Equations:**
  - Basic exponential solving (a*e^(bx) = c)
  - Pattern matching for solvable forms
- 19 unit tests
- Property tests for solution verification
- Performance benchmarks

### Added - Pattern Matching & Substitution (Phase H)

#### `pattern` - Pattern Matching Engine
- Wild pattern matching with variables
- Expression substitution
- Pattern-based rewriting
- Alpha-equivalence support
- Multi-pattern matching
- Comprehensive test coverage

### Added - Assumptions System (Phase I)

#### `assumptions` - Symbolic Assumptions
- Three-valued logic (True/False/Unknown)
- Assumption types:
  - Positivity (positive, nonnegative)
  - Sign (negative, zero, nonzero)
  - Domain (real, integer)
- Assumption propagation through expressions
- Integration with simplifier
- Scoped assumption contexts
- 30+ unit tests

### Added - I/O and Serialization

#### `io` - Expression I/O
- **S-Expression Format:**
  - Parser and printer
  - Canonical representation
  - Used in differential testing and fuzzing
- **JSON Format:**
  - Full expression serialization
  - Structured format for interop
  - Roundtrip preservation
- **LaTeX Output:**
  - Mathematical typesetting
  - Precedence-aware parenthesization
  - Support for all expression types
- 100+ unit tests

### Added - Numeric Evaluation

#### `evalf` - Arbitrary-Precision Evaluation
- Numeric evaluation of symbolic expressions
- Support for all standard functions
- Variable substitution
- Floating-point computation

### Added - Plotting

#### `plot` - SVG Function Plotting
- SVG-based function plotting
- Configurable sampling and dimensions
- Support for sin, cos, exp, ln
- Customizable plot ranges and sizes
- 20+ unit tests

### Added - Bindings & Interfaces

#### `api` - Python Bindings (PyO3)
- Python interface to core functionality
- Expression creation and manipulation
- Symbolic operations
- Feature-gated for optional Python support

#### `wasm` - WebAssembly Bindings
- WASM interface for web applications
- Browser-compatible symbolic computation
- Lightweight deployment

#### `cli` - Command-Line Interface
- Interactive symbolic computation
- Expression evaluation
- File-based processing

### Added - Quality Assurance (Phase L) ✅

#### Fuzzing Infrastructure
- 4 fuzz targets (cargo-fuzz):
  - `fuzz_diff` - Differentiation fuzzing
  - `fuzz_simplify` - Simplification fuzzing
  - `fuzz_expr_ops` - Expression operations
  - `fuzz_sexpr_parse` - Parser fuzzing
- Crash-free validation suite (10 tests)
- Deterministic behavior verification
- Malformed input handling tests

#### Property-Based Testing
- `proptest` integration across all crates
- Algebraic law verification:
  - Commutativity, associativity
  - Distributivity
  - Identity and zero properties
- Calculus property tests:
  - Linearity of differentiation
  - Product/chain rule verification
- Idempotence checks for simplification
- GCD/polynomial property tests

#### Differential Testing
- SymPy reference comparison
- Test categories:
  - Differentiation rules
  - Algebraic simplification
  - Integration patterns
  - Fundamental theorem of calculus
- Automatic validation pipeline
- 10 differential test cases

#### Performance Benchmarking
- Criterion.rs benchmarks for all 6 core crates:
  - `expr_core` - Expression building (8 benchmarks)
  - `simplify` - Simplification operations (7 benchmarks)
  - `calculus` - Diff/integrate (8 benchmarks)
  - `solver` - Equation solving (6 benchmarks)
  - `polys` - Polynomial operations (16 benchmarks)
  - `matrix` - Linear algebra (23 benchmarks)
- HTML reports for performance tracking
- Regression detection

### Testing Infrastructure

- **Total Test Count:** 465+ tests
  - Unit tests: 400+
  - Integration tests: 25 (tests_e2e)
  - Property tests: 30+
  - Fuzz validation: 10
- **Coverage:** 81.91% (via tarpaulin)
- **CI/CD:** GitHub Actions
  - Formatting (rustfmt)
  - Linting (clippy -D warnings)
  - All tests
  - Documentation build
  - Dependency audit
  - Security checks (cargo-deny)
  - Coverage reporting

### Documentation

- Comprehensive module documentation
- Algorithm explanations
- Usage examples in `/examples`
- API documentation with rustdoc
- Roadmap and architecture docs
- Testing strategy documentation:
  - `docs/fuzzing.md`
  - `docs/property_testing.md`
  - `docs/differential_testing.md`
  - `docs/benchmarking.md`

### Infrastructure

- Multi-crate workspace structure
- Feature flags for optional dependencies
- Consistent licensing (MIT OR Apache-2.0)
- Pre-commit hooks
- Local quality gates
- Reproducible builds

## Phase Completion Status

- ✅ **Phase A** - Foundations (expr_core, arith)
- ✅ **Phase B** - Baseline Algebra & Simplify v1
- ✅ **Phase C** - Polynomials v1
- ✅ **Phase D** - Calculus (diff, limits, series stub, integrate v1)
- ✅ **Phase E** - Matrices & Linear Algebra
- ✅ **Phase F** - Univariate Solver v1
- ✅ **Phase G** - Integration v1 (partial)
- ✅ **Phase H** - Pattern Matching v1
- ✅ **Phase I** - Assumptions v1
- ✅ **Phase J** - I/O & Bindings
- ✅ **Phase K** - WASM & Python
- ✅ **Phase L** - Hardening, Fuzzing, Differential Testing

## Next Steps Toward 1.0

- [x] API stability review and finalization ✅
- [x] Migration guide for 1.0 ✅
- [x] Complete documentation audit ✅
- [ ] Performance optimization pass (optional)
- [ ] 1.0.0-rc.1 release
- [ ] Community feedback period
- [ ] 1.0.0 release

## [0.0.0] - Initial Development

Project initialization and architecture design.

[Unreleased]: https://github.com/Sir-Teo/Symmetrica/compare/v1.0.0-rc.1...HEAD
[1.0.0-rc.1]: https://github.com/Sir-Teo/Symmetrica/compare/v0.1.0...v1.0.0-rc.1
[0.1.0]: https://github.com/Sir-Teo/Symmetrica/releases/tag/v0.1.0
[0.0.0]: https://github.com/Sir-Teo/Symmetrica/commits/main
