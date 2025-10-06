# Symmetrica Development Roadmap

**Current Version:** 1.0.0-rc.1  
**Last Updated:** October 2025  
**Status:** Release Candidate - Preparing for 1.0.0 stable release

---

## Vision

Transform Symmetrica into a comprehensive Computer Algebra System (CAS) comparable to Mathematica, while maintaining core principles of lightweight design, modularity, performance, and embeddability.

---

## Current Status: Advanced Integration & Simplification In Progress 🚀

Symmetrica has completed its foundational phase and is actively developing advanced integration and symbolic simplification capabilities. The system is production-ready with **201 calculus tests** (up from initial baseline) and **84.62% code coverage**.

### 📍 Active Development Summary (October 2025)

**Current Version:** 1.1-rc (Release Candidate)  
**Active Phases:** Phase 2 (Integration) + Phase 2.5 (Simplification)  
**Recent Commits:** 11 commits in Phase 2/2.5 (c41f81f → 85b3683)

#### What's Working Now ✅
- **Advanced Trigonometric Integration:** `∫ sin^m(x) cos^n(x) dx` for all cases (odd/even powers)
- **Weierstrass Substitution:** General form for rational trigonometric integrals
- **Risch Algorithm Foundation:** Exponential and logarithmic extensions
- **Integration by Parts:** Standalone inverse functions (ln(x), atan(x)) with LIATE heuristic
- **Definite Integrals:** Finite and infinite bounds with limit evaluation
- **Symbolic Simplification:**
  - Perfect square roots: √4 → 2, √(4/9) → 2/3
  - Exponential/log identities: ln(e^x) → x, e^(ln x) → x
  - Pythagorean identity: sin²x + cos²x → 1
  - Double-angle formulas: 2sin(x)cos(x) → sin(2x), cos²x - sin²x → cos(2x)

#### What's Next 🎯
- **Immediate (Next Session):** Pythagorean variants (1 + tan²x → sec²x)
- **Short-term (This Week):** Complete remaining trigonometric identities
- **Medium-term (This Month):** Hyperbolic identities, advanced u-substitution patterns

#### Test & Quality Metrics
- **Total Tests:** 201 (28 in symbolic_simplify, 173+ in integration/calculus)
- **Coverage:** 84.62% overall, 87.9% in symbolic_simplify.rs
- **Pass Rate:** 100% (all tests passing)
- **CI Status:** ✅ Green (fmt, clippy, tests, docs, coverage)

---

### ✅ Implemented Features (Current Release)

#### Core Infrastructure
- [x] **Expression Kernel** (`expr_core`)
  - Immutable DAG with hash-consing
  - Canonical forms for Add, Mul, Pow
  - Structural sharing and memoization
  - Expression digests for equality testing
  
- [x] **Rational Arithmetic** (`arith`)
  - Exact rational numbers (i64 numerator/denominator)
  - GCD/LCM algorithms
  - Fraction normalization
  
- [x] **Simplification** (`simplify`)
  - Like-term collection
  - Power merging and expansion
  - Rational function simplification
  - Automatic canonical form maintenance

#### Mathematical Capabilities

- [x] **Calculus** (`calculus`) **[ACTIVELY DEVELOPING]**
  - **Differentiation:** All standard rules (power, product, quotient, chain), hyperbolic, inverse trig ✅
  - **Integration:** Polynomial, rational functions, trigonometric patterns, Weierstrass substitution, Risch algorithm foundation, integration by parts (LIATE), definite integrals with infinite bounds 🚧
  - **Limits:** Algebraic limits, limit evaluation for improper integrals ✅
  - **Series Expansion:** Taylor/Maclaurin series ✅
  - **Symbolic Simplification:** Perfect squares, exp/log identities, Pythagorean identity, double-angle formulas 🚧

- [x] **Polynomial Algebra** (`polys`)
  - Univariate polynomials over ℚ
  - Multivariate polynomial operations
  - GCD computation (Euclidean algorithm)
  - Partial fraction decomposition
  - Resultants and discriminants

- [x] **Linear Algebra** (`matrix`)
  - Exact matrix operations over ℚ
  - Determinant (Bareiss algorithm)
  - Row reduction (RREF)
  - Matrix inversion
  - Rank computation

- [x] **Equation Solving** (`solver`)
  - Linear equations
  - Quadratic equations (exact symbolic roots)
  - Cubic equations (Cardano's formula)
  - Quartic equations (Ferrari's method)
  - Univariate polynomial solving

- [x] **Pattern Matching** (`pattern`)
  - Basic substitution rules
  - Pattern variables
  - Wildcard matching
  - Rule application

- [x] **Assumptions** (`assumptions`)
  - Variable domain constraints (real, positive, integer)
  - Assumption propagation
  - Simplification with assumptions

#### I/O & Interfaces

- [x] **Input/Output** (`io`)
  - S-expression parser and serializer
  - JSON serialization
  - LaTeX output
  - Pretty printing

- [x] **Numeric Evaluation** (`evalf`)
  - Floating-point evaluation
  - Configurable precision
  - Numeric approximation

- [x] **Visualization** (`plot`)
  - SVG function plotting
  - 2D graphs
  - Customizable styling

- [x] **Command-Line Interface** (`cli`)
  - Interactive REPL
  - Expression evaluation
  - File processing

- [x] **Python Bindings** (`api`)
  - PyO3-based Python API
  - NumPy integration
  - Pythonic interface

- [x] **WebAssembly** (`wasm`)
  - Browser deployment
  - JavaScript bindings
  - Web playground

#### Quality Assurance

- [x] **Testing Infrastructure** (`tests_e2e`)
  - 704 unit and integration tests
  - 87.98% code coverage
  - Property-based testing (proptest)
  - Differential testing framework
  - Fuzz testing harness
  - Benchmark suite

---

## Development Phases

### Phase 1: Foundation (COMPLETED ✅)

**Status:** 100% Complete  
**Timeline:** 2024-2025  
**Version:** 1.0.0-rc.1

All foundational features implemented and tested. System is production-ready.

#### Checklist (Complete)
- [x] Expression kernel (`expr_core`): immutable DAG, hash-consing, canonical `Add/Mul/Pow`
- [x] Rational arithmetic (`arith`): exact rationals, GCD/LCM, normalization
- [x] Simplification (`simplify`): like-term collection, power merging, guarded log/exp
- [x] Calculus (`calculus`): differentiation, conservative integration, limits, series
- [x] Polynomial algebra (`polys`): univariate/multivariate basics, GCD, resultants, partial fractions
- [x] Linear algebra (`matrix`): determinant (Bareiss), RREF, inversion, rank
- [x] Equation solving (`solver`): linear → quartic exact solvers
- [x] Pattern matching (`pattern`): basic substitution, wildcards, rule application
- [x] Assumptions (`assumptions`): domains (real/positive/integer), propagation
- [x] I/O (`io`): S-expr, JSON, LaTeX, pretty printing
- [x] Eval (`evalf`): numeric evaluation with precision control
- [x] Plotting (`plot`): 2D SVG plotting
- [x] CLI (`cli`): REPL, file processing
- [x] Bindings (`api`, `wasm`): Python (PyO3) and WebAssembly
- [x] QA: tests (704), coverage (87.98%), fuzzing, property tests, benchmarks

---

### Phase 2: Advanced Integration (v1.1)

**Status:** ✅ 80% Complete (IN PROGRESS)  
**Timeline:** October 2025 - December 2025  
**Goal:** Expand integration beyond rational functions  
**Current Progress:**
- ✅ Risch algorithm foundation (exponential/logarithmic extensions)
- ✅ Trigonometric integration (odd/even power products, reduction formulas)
- ✅ Weierstrass substitution (general form for rational trig integrals)
- ✅ Integration by parts for standalone inverse functions (ln(x), atan(x))
- ✅ Inverse trigonometric patterns (1/(a+x²) → atan)
- ✅ Definite integrals with finite and infinite bounds
- ✅ Limit evaluation for improper integrals
- ✅ Constant folding and evaluation
- 🔄 Advanced u-substitution heuristics (partial)
- 🔄 Non-elementary integral detection (partial)

#### Checklist

**Week 1-2: Risch Algorithm Foundation** ✅ COMPLETE
- [x] Implement differential field tower representation
- [x] Add logarithmic derivative computation
- [x] Create tower extension detection (exp/log)
- [x] Test with simple exponential integrals: `∫ e^x dx`, `∫ e^(2x) dx`
- [x] Verify differential check: `diff(integrate(f, x), x) = f`
- [x] Integration of logarithmic extensions (`∫ 1/x dx = ln(x)`)

**Week 3-4: Trigonometric Integration** ✅ COMPLETE
- [x] Implement Weierstrass substitution (tangent half-angle)
- [x] Weierstrass general form: `∫ 1/(a+b cos x) dx`
- [x] Add trigonometric reduction formulas (odd/even powers)
- [x] Pattern matching for `∫ sin^m(x) cos^n(x) dx` (all cases)
- [x] Hyperbolic function integration rules (sinh, cosh, tanh)
- [x] Property-based testing (18 trig integration tests)
- [x] Fundamental theorem verification by differentiation

**Week 5-6: Substitution Detection** ✅ MOSTLY COMPLETE
- [x] Basic u-substitution with pattern detection
- [x] Chain rule pattern recognition
- [x] Inverse trig substitution detection (atan patterns)
- [x] Integration by parts orchestrator with LIATE heuristic
- [x] Standalone inverse function integration (ln, atan)
- [ ] Advanced composite function integrals (remaining edge cases)

**Week 7-8: Definite Integrals & Limits** ✅ COMPLETE
- [x] Definite integral framework (finite bounds)
- [x] Improper integrals (infinite bounds)
- [x] Limit evaluation for boundary computation
- [x] Constant folding and evaluation
- [x] 27+ new test cases for advanced features
- [x] Documentation with examples (CALCULUS_PROGRESS.md)
- [ ] Performance profiling and optimization (deferred)
- [ ] Benchmark against SymPy (deferred)

**Acceptance Criteria:**
- ✅ 50+ new integration test cases pass (achieved: 201 total calculus tests)
- ✅ Differential check passes for all supported classes (verified)
- ✅ Performance: sub-second for expressions with <100 nodes (verified)
- ✅ No false positives (incorrect integrals) (all tests verify by differentiation)
- 🔄 Coverage maintained >85% (current: 84.62%, slightly below target)

**Design Considerations:**
- Risch algorithm with tower-based approach
- Heuristic ordering: try simple patterns first
- Memoization at expression subtree level
- Return `None` for non-elementary integrals

---

### Phase 2.5: Symbolic Simplification (v1.1.5) **[NEW PHASE]**

**Status:** ✅ 60% Complete (IN PROGRESS)  
**Timeline:** October 2025 - November 2025  
**Goal:** Intelligent calculus-aware simplification  
**Current Progress:**
- ✅ Perfect square root simplification (√4 → 2, √(4/9) → 2/3)
- ✅ Exponential/logarithmic identities (ln(e^x) → x, e^(ln x) → x, e^0 → 1)
- ✅ Inverse trigonometric identities (atan(tan x) → x, atan(0) → 0)
- ✅ Pythagorean identity (sin²x + cos²x → 1)
- ✅ Double-angle formula for sine (2sin(x)cos(x) → sin(2x))
- ✅ Double-angle formula for cosine (cos²x - sin²x → cos(2x))
- 🔄 Pythagorean variants (1 + tan²x → sec²x)
- 🔄 Identity rearrangements (1 - sin²x → cos²x)
- 🔄 Hyperbolic identities (cosh²x - sinh²x → 1)

#### Checklist

**Week 1-2: Foundation & Perfect Squares** ✅ COMPLETE
- [x] Create `symbolic_simplify.rs` module
- [x] Implement perfect square root detection
- [x] Recursive simplification pipeline
- [x] Integration with main simplifier
- [x] 10 initial tests

**Week 3-4: Trigonometric Identities** ✅ COMPLETE
- [x] Pythagorean identity: sin²x + cos²x → 1
- [x] Double-angle sine: 2sin(x)cos(x) → sin(2x)
- [x] Double-angle cosine: cos²x - sin²x → cos(2x)
- [x] Order-independent pattern matching
- [x] Argument-independent detection (works with any expression)
- [x] 12 comprehensive tests
- [x] Interactive examples demonstration

**Week 5: Remaining Identities** 🔄 IN PROGRESS
- [ ] Pythagorean variants (tan/sec, cot/csc)
- [ ] Identity rearrangements
- [ ] Half-angle formulas (optional)
- [ ] Hyperbolic identities

**Acceptance Criteria:**
- ✅ Pythagorean identity and double-angle formulas working
- ✅ Recursive simplification with multiple rule types
- ✅ 28 tests passing (current: 28/28 = 100%)
- ✅ Coverage: 87.9% for symbolic_simplify.rs
- ✅ Interactive examples (17 demonstrations)
- 🔄 Full identity suite (60% complete, 6/10 features)

---

### Phase 3: Special Functions Library (v1.2)

**Status:** 🔄 Not Started  
**Timeline:** Q2 2026 (12-16 weeks)  
**Goal:** Comprehensive special function support
**Current Coverage Snapshot:**
- No `crates/special` present; no special functions (Gamma/Bessel/Hypergeometric) in current APIs
- `evalf` provides numeric evaluation for elementary functions only; no special-function eval or series

#### Checklist

**Week 1-3: Infrastructure**
- [ ] Create `crates/special` module
- [ ] Design `SpecialFunction` trait (derivative, series, evalf methods)
- [ ] Implement function registry with metadata
- [ ] Add precision context for numerical evaluation
- [ ] Set up property-based testing framework

**Week 4-6: Gamma/Beta/Error Functions**
- [ ] Gamma function: `Γ(x)` with reflection formula
- [ ] Gamma function: duplication formula
- [ ] Incomplete Gamma: `Γ(x, a)` and `γ(x, a)`
- [ ] Digamma: `ψ(x)` with series expansion
- [ ] Beta function: `B(x, y)` and incomplete beta
- [ ] Error functions: `erf(x)`, `erfc(x)`, `erfi(x)`
- [ ] Exponential integrals: `Ei(x)`, `E_n(x)`
- [ ] Symbolic differentiation rules for all
- [ ] Numeric evaluation integration

**Week 7-10: Bessel Functions**
- [ ] Bessel J: `J_ν(x)` with series expansion
- [ ] Bessel J: asymptotic expansions
- [ ] Bessel Y: `Y_ν(x)` (Neumann function)
- [ ] Modified Bessel I: `I_ν(x)`
- [ ] Modified Bessel K: `K_ν(x)`
- [ ] Implement stable recurrence relations
- [ ] Wronskian identities for validation
- [ ] Test with DLMF reference values

**Week 11-13: Orthogonal Polynomials & Hypergeometric**
- [ ] Legendre: `P_n(x)` with Rodrigues' formula
- [ ] Chebyshev: `T_n(x)`, `U_n(x)` with trig representation
- [ ] Hermite: `H_n(x)` (physicist's and probabilist's)
- [ ] Laguerre: `L_n(x)` and associated Laguerre
- [ ] Jacobi: `P_n^(α,β)(x)`
- [ ] Hypergeometric: `₁F₁(a; b; z)` (Kummer's confluent)
- [ ] Hypergeometric: `₂F₁(a, b; c; z)` (Gauss)
- [ ] Transformation formulas between forms
- [ ] Connection formulas to other special functions

**Week 14-16: Testing & Integration**
- [ ] Property-based tests for recurrence relations
- [ ] Differential tests against SymPy/Mathematica
- [ ] Integration with calculus module (derivatives)
- [ ] Integration with evalf module (numerical evaluation)
- [ ] Comprehensive documentation with DLMF references
- [ ] Performance benchmarks
- [ ] Example gallery

**Acceptance Criteria:**
- ✅ 50+ special functions implemented
- ✅ Symbolic differentiation rules for all functions
- ✅ Numeric evaluation with configurable precision
- ✅ Series expansions at key points (0, ∞, singularities)
- ✅ Property tests verify recurrence relations
- ✅ Performance: O(1) function creation, lazy series computation
- ✅ Coverage >85%

**Design Considerations:**
- Function registry pattern for extensibility
- Lazy evaluation: compute series only when needed
- DLMF compliance for correctness
- Stable recurrence directions to avoid numerical instability

---

### Phase 4: Advanced Equation Solving (v1.3)

**Status:** 🔄 Not Started  
**Timeline:** Q3 2026 (16-20 weeks)  
**Goal:** Multivariate systems, transcendental equations, and ODEs
**Current Coverage Snapshot:**
- `solver`: linear → quartic univariate solving implemented (Cardano, Ferrari). No Lambert W or transcendental solvers
- `polys`: univariate GCD, resultants, discriminants; multivariate sparse polynomials exist, but no Gröbner bases
- No ODE solving framework in current codebase

#### Checklist

**Week 1-4: Gröbner Bases Foundation**
- [ ] Implement monomial ordering (lex, grlex, grevlex)
- [ ] Buchberger's algorithm with sugar cube optimization
- [ ] S-polynomial computation and reduction
- [ ] Criterion for detecting zero remainders
- [ ] Test with 2-3 variable systems
- [ ] Performance benchmarks

**Week 5-7: Gröbner Optimization & Elimination**
- [ ] F4 algorithm for matrix-based reduction (feature flag)
- [ ] Elimination theory via variable ordering
- [ ] Triangular decomposition for solving
- [ ] Resultant-based elimination as fallback
- [ ] Compare F4 vs Buchberger performance
- [ ] Test with 3-5 variable systems

**Week 8-10: Transcendental Equations**
- [ ] Lambert W function implementation (principal branch)
- [ ] Lambert W: multiple branches handling
- [ ] Pattern matching for `a·e^(bx) = c·x + d` forms
- [ ] Inverse trig function solving
- [ ] Inverse hyperbolic function solving
- [ ] Logarithmic equation solving
- [ ] Numeric fallback with Newton-Raphson
- [ ] Interval arithmetic for certified bounds

**Week 11-14: First-Order ODEs**
- [ ] Separable equations: `dy/dx = f(x)g(y)`
- [ ] Linear ODEs: `y' + p(x)y = q(x)` with integrating factor
- [ ] Exact equations: `M(x,y)dx + N(x,y)dy = 0`
- [ ] Exactness test implementation
- [ ] Bernoulli equations: `y' + p(x)y = q(x)y^n`
- [ ] Homogeneous equations with substitution
- [ ] Initial value problem (IVP) support
- [ ] Solution verification by substitution
- [ ] 50+ test cases from textbooks

**Week 15-17: Second-Order ODEs**
- [ ] Constant coefficients: characteristic equation method
- [ ] Cauchy-Euler equations: `x²y'' + axy' + by = 0`
- [ ] Reduction of order for known solution
- [ ] Series solutions (Frobenius method)
- [ ] Regular singular points handling
- [ ] Variation of parameters for non-homogeneous
- [ ] 30+ test cases

**Week 18-20: Systems of ODEs & Testing**
- [ ] Matrix exponential method: `X' = AX`
- [ ] Eigenvalue/eigenvector approach
- [ ] Phase plane analysis (optional)
- [ ] 100+ total ODE test cases
- [ ] Solution verification by substitution
- [ ] Performance profiling and optimization
- [ ] Comprehensive documentation
- [ ] Example gallery

**Acceptance Criteria:**
- ✅ Solve systems with 3-5 variables, degree ≤4 in <10s
- ✅ 100+ ODE test cases pass with verified solutions
- ✅ Correctness validated by substitution
- ✅ IVP support with symbolic constants
- ✅ Graceful failure for unsolvable cases
- ✅ Coverage >85%

**Design Considerations:**
- Gröbner basis with F4 optimization for large systems
- ODE classification via pattern matching
- Solution representation: explicit when possible, implicit otherwise
- Always verify solutions by substitution

---

### Phase 5: Symbolic Summation (v1.4)

**Status:** 🔄 Not Started  
**Timeline:** Q4 2026 (10-12 weeks)  
**Goal:** Closed-form summation and product evaluation
**Current Coverage Snapshot:**
- No dedicated summation module; no Gosper/Zeilberger; series support limited to Maclaurin in `calculus`

#### Checklist

**Week 1-3: Gosper's Algorithm**
- [ ] Hypergeometric term recognition
- [ ] Rational certificate computation
- [ ] Closed-form detection
- [ ] Test with standard hypergeometric sums
- [ ] Verify results symbolically

**Week 4-6: Zeilberger's Algorithm**
- [ ] Creative telescoping implementation
- [ ] Recurrence relation generation
- [ ] Certificate computation
- [ ] Test with binomial sums
- [ ] Integration with Gosper's algorithm

**Week 7-9: Basic Sums & Series**
- [ ] Arithmetic series: `∑ (a + kd)`
- [ ] Geometric series: `∑ ar^k`
- [ ] Power sums: `∑ k^n`
- [ ] Binomial sums: `∑ C(n,k) f(k)`
- [ ] Convergence tests (ratio, root, integral)
- [ ] Series acceleration (Shanks, Wynn-ε)

**Week 10-12: Products & Testing**
- [ ] Infinite products
- [ ] Pochhammer symbol: `(x)_n`
- [ ] Connection to Gamma function
- [ ] 50+ summation test cases
- [ ] Performance benchmarks
- [ ] Documentation and examples

**Acceptance Criteria:**
- ✅ Gosper's algorithm handles hypergeometric terms
- ✅ Zeilberger's algorithm generates recurrences
- ✅ Basic sum formulas work correctly
- ✅ Convergence tests implemented
- ✅ 50+ test cases pass
- ✅ Coverage >85%

---

### Phase 6: Enhanced Simplification (v2.0)

**Status:** 🔄 Not Started  
**Timeline:** Q1 2027 (12-14 weeks)  
**Goal:** Advanced simplification and rewriting
**Current Coverage Snapshot:**
- `simplify`: like-term collection, power merging, guarded log/exp rules via `assumptions`
- Missing: trigonometric identities, radical denesting, logarithm branch-cut aware rules, e-graph equality saturation

#### Checklist

**Week 1-4: Trigonometric Simplification**
- [ ] Pythagorean identities: `sin²(x) + cos²(x) = 1`
- [ ] Double angle formulas
- [ ] Half angle formulas
- [ ] Sum-to-product identities
- [ ] Product-to-sum identities
- [ ] Trigonometric reduction to canonical form
- [ ] Test with 30+ trig expressions

**Week 5-7: Radical Simplification**
- [ ] Denesting: `√(a + b√c)` → `√d + √e`
- [ ] Ramanujan's denesting algorithm
- [ ] Denominator rationalization
- [ ] Conjugate multiplication
- [ ] Test with nested radicals

**Week 8-10: Logarithm & Exponential Rules**
- [ ] `log(a·b) → log(a) + log(b)` with assumptions
- [ ] `log(a^n) → n·log(a)` with branch cuts
- [ ] `exp(log(x)) → x` simplification
- [ ] Multi-valued function handling
- [ ] Branch cut detection

**Week 11-14: E-Graph Rewriting (Optional)**
- [ ] Equality saturation implementation (feature flag)
- [ ] Integration with `egg` crate or custom
- [ ] Rule set for algebraic identities
- [ ] Performance comparison with current simplifier
- [ ] Documentation and examples

**Acceptance Criteria:**
- ✅ Trig identities applied automatically
- ✅ Radical denesting when possible
- ✅ Log/exp rules with correct branch handling
- ✅ E-graph feature optional and performant
- ✅ Coverage >85%

---

### Phase 7: Number Theory (v2.1)

**Status:** 🔄 Not Started  
**Timeline:** Q2 2027 (8-10 weeks)  
**Goal:** Computational number theory capabilities
**Current Coverage Snapshot:**
- No `number_theory` crate; `arith` provides rational arithmetic only
- No primality, factorization beyond polynomial contexts, or modular arithmetic utilities

#### Checklist

**Week 1-3: Integer Arithmetic**
- [ ] Miller-Rabin primality test
- [ ] AKS deterministic test (optional)
- [ ] Trial division factorization
- [ ] Pollard's rho factorization
- [ ] Quadratic sieve (for large integers)
- [ ] Extended Euclidean algorithm
- [ ] Batch GCD computation

**Week 4-6: Modular Arithmetic**
- [ ] Modular exponentiation
- [ ] Chinese Remainder Theorem
- [ ] Modular inverse
- [ ] Discrete logarithm (baby-step giant-step)
- [ ] Test with cryptographic examples

**Week 7-8: Diophantine Equations**
- [ ] Linear Diophantine equations
- [ ] Pell's equation solver
- [ ] Pythagorean triples generation
- [ ] Test with classical problems

**Week 9-10: Partition Functions & Testing**
- [ ] Integer partitions
- [ ] Partition generating functions
- [ ] Restricted partitions
- [ ] 50+ number theory test cases
- [ ] Documentation and examples

**Acceptance Criteria:**
- ✅ Fast primality testing
- ✅ Factorization for large integers
- ✅ CRT implementation
- ✅ Diophantine solver
- ✅ Coverage >85%

---

### Phase 8: Tensor Algebra (v2.2)

**Status:** 🔄 Not Started  
**Timeline:** Q3 2027 (16-20 weeks)  
**Goal:** Tensor computations and differential geometry
**Current Coverage Snapshot:**
- No tensor types or differential geometry utilities in current crates

#### Checklist

**Week 1-5: Tensor Basics**
- [ ] Tensor type with arbitrary rank
- [ ] Index notation (Einstein summation)
- [ ] Covariant/contravariant indices
- [ ] Tensor contraction
- [ ] Tensor product
- [ ] Test with simple tensors

**Week 6-10: Differential Geometry**
- [ ] Metric tensor operations
- [ ] Christoffel symbols computation
- [ ] Riemann curvature tensor
- [ ] Ricci tensor and scalar
- [ ] Geodesic equations
- [ ] Test with standard metrics (Minkowski, Schwarzschild)

**Week 11-15: Exterior Calculus**
- [ ] Differential forms
- [ ] Wedge product
- [ ] Exterior derivative
- [ ] Hodge star operator
- [ ] Test with Maxwell's equations

**Week 16-20: Applications & Testing**
- [ ] General relativity examples
- [ ] Classical mechanics (Lagrangian/Hamiltonian)
- [ ] Electromagnetism applications
- [ ] 50+ tensor test cases
- [ ] Documentation and examples

**Acceptance Criteria:**
- ✅ Arbitrary rank tensors
- ✅ Einstein summation
- ✅ Differential geometry operations
- ✅ Exterior calculus
- ✅ Coverage >85%

---

### Phase 9: Algebraic Extensions (v2.3)

**Status:** 🔄 Not Started  
**Timeline:** Q4 2027 (14-18 weeks)  
**Goal:** Exact computation with algebraic numbers
**Current Coverage Snapshot:**
- No algebraic number or field extension support; all arithmetic over ℚ

#### Checklist

**Week 1-4: Algebraic Numbers**
- [ ] Representation as polynomial roots
- [ ] Arithmetic operations
- [ ] Minimal polynomial computation
- [ ] Algebraic number recognition
- [ ] Test with quadratic extensions

**Week 5-9: Field Extensions**
- [ ] `ℚ(√2)` implementation
- [ ] `ℚ(i)` implementation
- [ ] `ℚ(ζ_n)` (cyclotomic fields)
- [ ] Tower of extensions
- [ ] Galois theory computations
- [ ] Test with classical examples

**Week 10-14: Algebraic Simplification**
- [ ] Simplify expressions in extensions
- [ ] Norm computations
- [ ] Trace computations
- [ ] Conjugate elements
- [ ] Test with algebraic identities

**Week 15-18: Testing & Integration**
- [ ] 50+ algebraic number test cases
- [ ] Integration with solver
- [ ] Integration with simplifier
- [ ] Documentation and examples

**Acceptance Criteria:**
- ✅ Arithmetic in field extensions
- ✅ Tower of extensions
- ✅ Galois theory basics
- ✅ Coverage >85%

---

### Phase 10: Code Generation (v2.4)

**Status:** 🔄 Not Started  
**Timeline:** Q1 2028 (10-12 weeks)  
**Goal:** Generate optimized code from symbolic expressions
**Current Coverage Snapshot:**
- No code generation backends in the repository; examples and CLI focus on evaluation/plotting

#### Checklist

**Week 1-3: C Code Generation**
- [ ] Function generation with proper types
- [ ] Loop optimization
- [ ] SIMD hints
- [ ] Test compilation and execution

**Week 4-6: Fortran & Julia Generation**
- [ ] Fortran array operations
- [ ] Scientific computing conventions
- [ ] Julia native syntax
- [ ] Type annotations
- [ ] Test with numerical codes

**Week 7-9: Common Subexpression Elimination**
- [ ] Identify repeated subexpressions
- [ ] Generate intermediate variables
- [ ] Minimize computation count
- [ ] Test with large expressions

**Week 10-12: Automatic Differentiation & Testing**
- [ ] Generate derivative functions
- [ ] Forward mode AD
- [ ] Reverse mode AD
- [ ] Jacobian/Hessian generation
- [ ] 30+ code generation test cases
- [ ] Documentation and examples

**Acceptance Criteria:**
- ✅ Valid C/Fortran/Julia code generation
- ✅ CSE optimization
- ✅ AD code generation
- ✅ Performance benchmarks
- ✅ Coverage >85%

---

### Phase 11: Interactive Computing (v2.5)

**Status:** 🔄 Not Started  
**Timeline:** Q2 2028 (8-10 weeks)  
**Goal:** Enhanced user interaction and visualization
**Current Coverage Snapshot:**
- `wasm` crate and web playground exist; `plot` supports 2D SVG plots
- No Jupyter kernel; advanced/3D plotting and interactive sliders not implemented

#### Checklist

**Week 1-3: Jupyter Kernel**
- [ ] Jupyter protocol implementation
- [ ] Cell-based evaluation
- [ ] Rich output (LaTeX, plots, tables)
- [ ] Markdown documentation cells
- [ ] Test with Jupyter notebook

**Week 4-6: Advanced Plotting**
- [ ] Multiple functions on same axes
- [ ] Parametric plots
- [ ] Polar plots
- [ ] Contour plots
- [ ] Vector field plots
- [ ] 3D surface plots
- [ ] Parametric surfaces

**Week 7-8: Interactive Features**
- [ ] Pan, zoom, rotate
- [ ] Parameter sliders
- [ ] Animation support
- [ ] Test in browser

**Week 9-10: Pretty Printing & Testing**
- [ ] Unicode math rendering in terminal
- [ ] HTML/MathML output
- [ ] Syntax highlighting
- [ ] Expression tree visualization
- [ ] Documentation and examples

**Acceptance Criteria:**
- ✅ Jupyter kernel functional
- ✅ Advanced 2D/3D plotting
- ✅ Interactive visualizations
- ✅ Pretty printing enhancements
- ✅ Coverage >85%

---

### Phase 12: Advanced Pattern Matching (v2.6)

**Status:** 🔄 Not Started  
**Timeline:** Q3 2028 (14-16 weeks)  
**Goal:** Mathematica-level pattern matching
**Current Coverage Snapshot:**
- `pattern`: basic substitution, wildcards, rule application
- Missing: AC-matching, sequence/conditional patterns, rule strategies/priorities

#### Checklist

**Week 1-4: Advanced Pattern Syntax**
- [ ] Sequence patterns: `a___`, `b__`, `c_`
- [ ] Conditional patterns: `x_ /; x > 0`
- [ ] Pattern alternatives: `x_ | y_`
- [ ] Named patterns with constraints
- [ ] Test with complex patterns

**Week 5-8: AC-Matching**
- [ ] Efficient AC-matching algorithm
- [ ] Multiset matching for Add/Mul
- [ ] Orderless pattern matching
- [ ] Test with commutative operations

**Week 9-12: Transformation Rules**
- [ ] Rule application strategies (innermost, outermost)
- [ ] Repeated rule application with termination
- [ ] Rule priority and ordering
- [ ] Conditional rewriting
- [ ] Test with rewrite systems

**Week 13-16: Pattern-Based Simplification & Testing**
- [ ] User-defined simplification rules
- [ ] Rule sets for specific domains
- [ ] Automatic rule conflict detection
- [ ] 50+ pattern matching test cases
- [ ] Documentation and examples

**Acceptance Criteria:**
- ✅ Full pattern syntax
- ✅ Efficient AC-matching
- ✅ Rule application strategies
- ✅ User-defined rules
- ✅ Coverage >85%

---

### Phase 13: PDEs (v3.0)

**Status:** 🔄 Not Started  
**Timeline:** Q4 2028 (20-24 weeks)  
**Goal:** Symbolic PDE solving
**Current Coverage Snapshot:**
- No PDE support; only basic limits/series and integrator in `calculus`

#### Checklist

**Week 1-6: Classification & Canonical Forms**
- [ ] First-order PDEs (method of characteristics)
- [ ] Second-order linear PDEs classification
- [ ] Canonical form reduction
- [ ] Test with standard PDEs

**Week 7-12: Separation of Variables**
- [ ] Cartesian coordinates
- [ ] Cylindrical coordinates
- [ ] Spherical coordinates
- [ ] Eigenfunction expansions
- [ ] Fourier series solutions
- [ ] Test with boundary value problems

**Week 13-18: Transform Methods**
- [ ] Fourier transform
- [ ] Laplace transform
- [ ] Hankel transform
- [ ] Test with classical PDEs

**Week 19-24: Special Cases & Testing**
- [ ] Heat equation
- [ ] Wave equation
- [ ] Laplace equation
- [ ] Schrödinger equation
- [ ] 50+ PDE test cases
- [ ] Documentation and examples

**Acceptance Criteria:**
- ✅ PDE classification
- ✅ Separation of variables
- ✅ Transform methods
- ✅ Special cases solved
- ✅ Coverage >85%

---

### Phase 14: Probability & Statistics (v3.1)

**Status:** 🔄 Not Started  
**Timeline:** Q1 2029 (8-10 weeks)  
**Goal:** Symbolic probability and statistics
**Current Coverage Snapshot:**
- No probability distributions or random variable algebra modules present

#### Checklist

**Week 1-3: Probability Distributions**
- [ ] Discrete: Binomial, Poisson, Geometric
- [ ] Continuous: Normal, Exponential, Gamma, Beta
- [ ] PDF, CDF, moments, MGF
- [ ] Test with standard distributions

**Week 4-6: Random Variables**
- [ ] Symbolic random variable algebra
- [ ] Expectation and variance
- [ ] Moment generating functions
- [ ] Characteristic functions
- [ ] Test with transformations

**Week 7-8: Statistical Tests**
- [ ] Hypothesis testing (symbolic)
- [ ] Confidence intervals
- [ ] Distribution fitting
- [ ] Test with examples

**Week 9-10: Testing & Documentation**
- [ ] 30+ probability test cases
- [ ] Documentation and examples

**Acceptance Criteria:**
- ✅ Probability distributions
- ✅ Random variable algebra
- ✅ Statistical tests
- ✅ Coverage >85%

---

## Architectural Principles

### 1. Incremental Complexity Management
- Layered architecture: each phase builds on previous
- Feature flags for heavy algorithms
- Graceful degradation: return partial results or `None`
- Complexity budgets: max expression size, depth, time

### 2. Correctness-First Development
- Differential verification for integration
- Property-based testing for algebraic laws
- Cross-validation with SymPy/Mathematica
- Proof obligations documented

### 3. Performance by Design
- Algorithmic selection: best asymptotic complexity
- Memoization at expression subtree level
- Lazy evaluation: defer until needed
- Parallel opportunities identified

### 4. Extensibility Through Composition
- Trait-based design for common operations
- Registry pattern for functions/rules
- Visitor pattern for tree traversal
- Plugin architecture

### 5. Error Handling Philosophy
- Explicit failure: `Option<T>` or `Result<T, E>`
- Partial results when full solution not found
- Rich error messages
- Timeout guards

### 6. Testing Strategy
- Unit tests: known inputs/outputs
- Property tests: algebraic laws
- Differential tests: compare with references
- Regression tests: capture bugs
- Benchmark tests: performance tracking

### 7. Documentation as Code
- Algorithm documentation with math background
- Complexity analysis for each operation
- Runnable examples for every public API
- Research references to papers/textbooks

### 8. Backward Compatibility
- Semantic versioning strictly followed
- Deprecation policy: 2+ minor versions before removal
- API stability for core operations
- Migration guides for breaking changes

---

## Cross-Cutting Concerns

### Performance Monitoring
- Comprehensive benchmark suite
- CI fails if performance degrades >10%
- Regular profiling with flamegraph
- Memory tracking (arena allocator, hash-consing)

### Security & Resource Management
```rust
pub struct ResourceLimits {
    max_expression_size: usize,      // Default: 1_000_000 nodes
    max_expression_depth: usize,     // Default: 10_000
    max_computation_time: Duration,  // Default: 60s
    max_memory_usage: usize,         // Default: 1GB
}
```

### Code Quality Standards
**Pre-Commit Checklist:**
- ✅ `cargo fmt --all -- --check`
- ✅ `cargo clippy --workspace --all-targets -- -D warnings`
- ✅ `cargo test --workspace --all-features`
- ✅ `cargo doc --workspace --no-deps`
- ✅ Coverage maintained >85%

**Code Review Requirements:**
- Two approving reviews
- Test coverage for new code
- Documentation for public APIs
- Performance benchmarks for critical changes

### Dependency Management
- Minimize dependencies (prefer std)
- Run `cargo audit` on every commit
- Only MIT/Apache-2.0 compatible licenses
- Optional dependencies behind feature flags

---

## Success Metrics

### Technical Metrics
- **Performance:** Match or exceed SymPy on standard benchmarks
- **Coverage:** Maintain >85% code coverage
- **Correctness:** 100% pass rate on differential tests
- **Size:** Core library <50MB (with feature flags)

### Adoption Metrics
- **Users:** 10,000+ active users by v2.0
- **Contributors:** 50+ contributors by v2.0
- **Packages:** 20+ third-party packages by v3.0
- **Citations:** 50+ academic citations by v3.0

### Capability Metrics
- **Integration:** 90% success rate on standard integral tables by v2.0
- **Solving:** Handle 95% of undergraduate-level problems by v2.0
- **Special Functions:** 50+ functions with full support by v2.0
- **Performance:** <1s for typical symbolic computations

---

## Feature Parity Roadmap

| Feature | Mathematica | Current (1.0) | v2.0 Target | v3.0 Target |
|---------|-------------|---------------|-------------|-------------|
| Basic Algebra | ✅ | ✅ 100% | ✅ 100% | ✅ 100% |
| Differentiation | ✅ | ✅ 100% | ✅ 100% | ✅ 100% |
| Integration | ✅✅✅ | ✅✅ 70% | ✅✅ 80% | ✅✅✅ 95% |
| Special Functions | ✅✅✅ | ❌ 0% | ✅✅ 60% | ✅✅ 80% |
| Polynomial Algebra | ✅ | ✅ 90% | ✅ 95% | ✅ 100% |
| Linear Algebra | ✅ | ✅ 90% | ✅ 95% | ✅ 100% |
| Equation Solving | ✅✅ | ✅ 50% | ✅✅ 75% | ✅✅ 85% |
| Pattern Matching | ✅✅✅ | ✅ 30% | ✅✅ 60% | ✅✅✅ 95% |
| ODEs | ✅✅✅ | ❌ 0% | ✅ 50% | ✅✅ 80% |
| PDEs | ✅✅ | ❌ 0% | ❌ 0% | ✅ 60% |
| Number Theory | ✅✅ | ❌ 0% | ✅✅ 80% | ✅✅ 90% |
| Tensor Algebra | ✅✅ | ❌ 0% | ✅ 50% | ✅ 70% |
| Code Generation | ✅✅ | ❌ 0% | ✅✅ 80% | ✅✅ 90% |
| Visualization | ✅✅✅ | ✅ 20% | ✅✅ 50% | ✅✅ 75% |

**Overall Capability:**
- **Current (1.1 RC):** ~55% of Mathematica (up from 45% in v1.0)
- **v2.0 Target:** ~70% of Mathematica
- **v3.0 Target:** ~85% of Mathematica

---

## Timeline Summary

| Version | Timeline | Focus | Status |
|---------|----------|-------|--------|
| 1.0 | 2024-2025 | Foundation | ✅ Complete |
| 1.1 | Oct-Dec 2025 | Integration v2 | ✅ 80% Complete (IN PROGRESS) |
| 1.1.5 | Oct-Nov 2025 | Symbolic Simplification | ✅ 60% Complete (IN PROGRESS) |
| 1.2 | Q2 2026 | Special Functions | 🔄 Not Started |
| 1.3 | Q3 2026 | Advanced Solving | 🔄 Not Started |
| 1.4 | Q4 2026 | Summation | 🔄 Not Started |
| 2.0 | Q1 2027 | Simplification | 🔄 Not Started |
| 2.1 | Q2 2027 | Number Theory | 🔄 Not Started |
| 2.2 | Q3 2027 | Tensor Algebra | 🔄 Not Started |
| 2.3 | Q4 2027 | Algebraic Extensions | 🔄 Not Started |
| 2.4 | Q1 2028 | Code Generation | 🔄 Not Started |
| 2.5 | Q2 2028 | Interactive Computing | 🔄 Not Started |
| 2.6 | Q3 2028 | Advanced Patterns | 🔄 Not Started |
| 3.0 | Q4 2028 | PDEs | 🔄 Not Started |
| 3.1 | Q1 2029 | Probability | 🔄 Not Started |

**Total Timeline:** ~3.5-4 years from 1.0 to 3.1

---

## Contributing

### How to Get Involved

1. **Pick a Phase:** Choose a phase that interests you
2. **Study Algorithms:** Review academic literature (see [research.md](research.md))
3. **Prototype:** Create proof-of-concept implementations
4. **Test:** Write comprehensive tests (unit + property + differential)
5. **Document:** Explain design choices and algorithms
6. **Submit:** Open PRs with clear descriptions

### Resources
- **Research Notes:** [research.md](research.md) - Algorithm references
- **API Documentation:** `cargo doc --workspace --no-deps --open`
- **Examples:** See `examples/` directory
- **Discussions:** https://github.com/Sir-Teo/Symmetrica/discussions

---

## References

### Academic Literature
- Bronstein, M. (2005). *Symbolic Integration I: Transcendental Functions*
- Geddes, K. O., Czapor, S. R., & Labahn, G. (1992). *Algorithms for Computer Algebra*
- von zur Gathen, J., & Gerhard, J. (2013). *Modern Computer Algebra*
- Davenport, J. H., Siret, Y., & Tournier, E. (1988). *Computer Algebra: Systems and Algorithms*

### CAS Systems (Prior Art)
- SymPy: https://www.sympy.org/
- Maxima: https://maxima.sourceforge.io/
- GiNaC: https://www.ginac.de/
- SymEngine: https://github.com/symengine/symengine
- SageMath: https://www.sagemath.org/

### Algorithm Resources
- DLMF: https://dlmf.nist.gov/ (Digital Library of Mathematical Functions)
- OEIS: https://oeis.org/ (Online Encyclopedia of Integer Sequences)
- Wolfram Functions: https://functions.wolfram.com/

---

**Document Version:** 3.0 (Complete Rewrite)  
**Last Updated:** October 2025  
**Status:** Living Document
