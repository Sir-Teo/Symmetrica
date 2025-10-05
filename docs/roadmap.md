# Symmetrica Development Roadmap

**Vision:** Transform Symmetrica from a solid symbolic computation engine into a comprehensive Computer Algebra System (CAS) comparable to Mathematica, while maintaining our core principles of lightweight design, modularity, and performance.

**Last Updated:** October 2025

---

## Table of Contents

1. [Original Roadmap (Pre-1.0) - COMPLETED](#original-roadmap-pre-10---completed-)
2. [Post-1.0 Roadmap: Toward a Full-Featured CAS](#post-10-roadmap-toward-a-full-featured-cas)
3. [Phase Overview & Timeline](#phase-overview--timeline)
4. [Comparison with Other CAS Systems](#comparison-with-other-cas-systems)

---

## Original Roadmap (Pre-1.0) - COMPLETED ✅

Below is the master implementation roadmap that guided Symmetrica's development from inception to v1.0. This roadmap has been **fully completed** as of October 2025.

⸻

### 0) Project Vision & Guardrails

Primary goal: A compact, fast, embeddable symbolic engine covering:
	•	symbolic expressions (algebra)
	•	simplification & rewriting
	•	calculus (diff, basic limits/series, staged integration)
	•	equation solving (linear, polynomial first; staged transcendental)
	•	symbolic matrices/linear algebra
	•	numeric evaluation (arbitrary precision)

Non-goals (for v1): notebooks/plots/UI, PDEs, full Risch, full Gröbner bases for large multivariate ideals, computer algebra for noncommutative structures, comprehensive special‑function encyclopedias.

Principles:
	•	Lightweight: minimal deps; modular crates; build-time feature flags; WASM target.
	•	Deterministic: canonical forms; reproducible outputs.
	•	Safe & scalable: immutability, DAG sharing, resource guards, linear/multithread scalability for large operand sets.
	•	Extensible: plugin-like modules; registry for functions/rules; stable public API.

Initial target budgets (soft):
	•	Core static lib (release, stripped): < 10–15 MB (Rust; WASM < ~2–5 MB gzipped with features pruned).
	•	Typical operations on expressions with ~10⁴ nodes complete in sub-second on commodity CPU (after warm caches).
	•	Memory overhead: sub‑linear growth via DAG/consing; avoid quadratic blowups by design.

⸻

1) High-Level Architecture (modules/crates)

/engine
  /crates
    expr_core         # expression kernel, stores, interning, hashing, printing, serialization
    arith             # integers, rationals; numeric kernels; big-int/MPFR bridge
    polys             # uni-/multivariate polynomial types, GCD, factor skeleton, resultants
    simplify          # canonicalization + algebraic normal forms + rewrite scheduler
    pattern           # substitution + (later) AC pattern matching + rule registry
    calculus          # diff, limits, series; staged integration
    solver            # linear systems, univariate roots; staged transcendental solve
    matrix            # exact linear algebra, fraction-free methods
    assumptions       # property lattice, domains, context handling
    evalf             # arbitrary-precision numeric eval; interval arithmetic (optional)
    api               # stable FFI, C ABI, and Python (pyo3) bindings
    io                # parser (lightweight), pretty/LaTeX printers, S-expr/JSON serialization
    cli               # minimal REPL (dev convenience)
    wasm              # wasm-pack target with small API surface
    benches           # micro/macro benchmarks
    tests             # e2e, property, fuzz, differential


⸻

2) Core Data Model & Performance Design

2.1 Expression kernel (immutable DAG + hash‑consing)
	•	Node = { OpTag, smallvec<ChildId>, Payload } where Payload holds immediates (e.g., small ints) or function symbols.
	•	Arena Store per “session”:
	•	Hash‑consing: unique table keyed by (OpTag, normalized children, payload) → NodeId to guarantee structural sharing.
	•	Refcounts (Arc) or generation indices to reclaim unreachable subgraphs.
	•	Canonicalization-in-construction: flatten associative ops (Add, Mul), sort operands (deterministic), separate numeric part (coefficient) from symbolic monomials.
	•	Atom interning: Symbols, small integers, common constants (0,1,π,e,i) are interned.
	•	Equality: pointer equality ⇒ structural equality (thanks to consing); deep equals only on store boundary.
	•	Hashing: stable, cached per node; use strong hasher; memoized.

2.2 Types (first wave)
Integer, Rational, Symbol, Constant, Add, Mul, Pow, Function, Piecewise, Relational(=,≠,<,≤,>).
Later: AlgebraicNumber, Complex literal, MatrixExpr.

2.3 Assumptions system (skeleton)
	•	3-valued lattice per property: {True, False, Unknown} for real, positive, integer, nonzero, finite, ...;
	•	Scoped context; rewrite honors assumptions (no unsafe simplifications).

⸻

3) Simplification & Rewriting Strategy

3.1 Default simplifier (fast, safe, local)
	•	Constant folding; unit/zero elimination; combine like terms/factors; rational normalization with polynomial GCD.
	•	Canonical ordering of operands and terms; fraction normalization (cancel GCD numerator/denominator).
	•	No expansion/factorization by default (avoid expression swell).

3.2 Rewrite system (explicit)
	•	Substitution v1: symbol→expr; exact head matches.
	•	Rules registry: (pattern, constraints) -> replacement; rule families keyed by OpTag.
	•	AC-aware matching v2 (Add/Mul commutativity/associativity) with multiset matching; cost-based chooser.
	•	Strategy: bounded, confluent-ish pipelines: normalize → cheap rules → domain-aware rules → (optional) heavy transforms with iteration caps and size guards.
	•	Optional advanced: e-graph-based saturation behind a feature flag (kept out of default build for lightness).

⸻

4) Algebra & Polynomials

4.1 Arithmetic kernel
	•	BigInt/Rational via a single trait; implement with rug (GMP/MPFR) or pure-Rust bigints behind feature flags.
	•	Exact arithmetic; rational canonicalization everywhere.

4.2 Polynomial module
	•	Univariate dense for small degree; sparse (hashmap) for multi-variate.
	•	Algorithms:
	•	Euclidean division; subresultant PRS; GCD; square‑free decomposition.
	•	Factorization skeleton: trial division; Berlekamp/EDF over finite fields + rational reconstruction (staged in later phase).
	•	Resultants; discriminants; partial fractions (proper/improper).
	•	Tight conversion: Expr ⟷ Poly when heads permit.

⸻

5) Calculus

5.1 Differentiation (v1)
	•	Structural rules for Add/Mul/Pow/Function with chain/product/quotient; derivative tables for common functions; simplify-on-the-fly.

5.2 Limits & series (v1)
	•	Heuristic limits (polynomial dominance, L’Hôpital for simple forms).
	•	Series: univariate Taylor/Maclaurin for standard functions; operations on series (add/mul/compose/truncate).

5.3 Integration (phased)
	•	v1: rational functions via partial fractions; linear change-of-variable detection; basic ∫f’/f forms; common trig/exponential patterns.
	•	v2: heuristic Risch fragments (exp/log towers); integration by parts orchestrator; table lookup with validation.
	•	v3 (optional): full transcendental Risch subsets; algebraic function handling.

⸻

6) Solvers

6.1 Linear algebraic solve
	•	Exact linear system solving (fraction‑free Gaussian elimination / Bareiss), LU for rationals.
	•	Determinant, rank, nullspace (symbolic).

6.2 Polynomial equations
	•	Univariate: factor → roots (closed forms up to quartic; otherwise root objects / isolating intervals).
	•	Systems (phased): resultants/elimination; Gröbner bases (Buchberger) behind a feature flag (later).

6.3 Transcendental equations (v1)
	•	Patterned forms (e.g., a*exp(bx)=c → LambertW); fallback: numeric root-finding under evalf with certification if intervals enabled.

⸻

7) Matrix / Linear Algebra
	•	Symbolic matrices with exact entries; lazy simplification of elements.
	•	Determinant via Bareiss; adjugate/inverse when feasible; block operations.
	•	Polynomial matrices (Smith normal form later).

⸻

8) Numeric Evaluation (evalf)
	•	Arbitrary precision floats (MPFR via rug), complex support; round-half-even; controllable precision context.
	•	Interval arithmetic (optional feature) for certified bounds in numeric solve/integrate.

⸻

9) API, Interop & Tooling

9.1 Public API
	•	Rust crate API (Expr builder, visitors, simplify, diff, integrate, solve, matrix ops).
	•	C FFI + stable ABI for other languages.
	•	Python bindings (pyo3) exposing idiomatic names; install via pip.

9.2 Serialization & I/O
	•	Parser for a compact math syntax (operators, function calls, piecewise).
	•	Printers: pretty text, Unicode math, LaTeX; machine formats: S‑expr, JSON/CBOR (schema versioned).

9.3 WASM
	•	Small surface: parse → expr; evaluate/simplify/diff/solve; print.
	•	Resource caps enforced (time/steps/memory).

⸻

10) Resource Governance, Security & Observability
	•	Guards: max nodes, max depth, max rewrite steps, max expansion, timeouts (cooperative), numeric precision cap.
	•	Determinism: canonical ordering; seeded but stable tie‑breakers.
	•	Metrics: op counters, node counts, cache hit rates; optional tracing spans.
	•	Errors: rich diagnostics (why a simplification didn’t apply; which rule matched).

⸻

11) Testing & Quality
	•	Unit tests per module with table-driven cases.
	•	Property tests (e.g., d/dx ∫ f dx ~ f on safe classes; expand(f*g)==expand(f)*expand(g) within degree bounds).
	•	Golden tests for printers/serialization.
	•	Differential testing: randomly generate expressions; compare certain outcomes with a reference engine (when behavior is mathematically unambiguous).
	•	Fuzzing of parser and simplifier.
	•	Benchmarks: micro (Add/Mul combine; GCD; partial fractions) and macro (symbolic determinant for k×k, expand (x+1)^n under guard).

⸻

12) Documentation & Governance
	•	Architecture docs (module responsibilities, invariants, data model).
	•	Contribution guide (rule authoring, function registration, test patterns).
	•	API docs with runnable examples.
	•	Release policy (semver; feature flags listed).
	•	License: MIT or Apache‑2.0 (permissive); call out dependency licenses; optional CLA.

⸻

13) Phased Roadmap (deliverables & acceptance criteria)

Phase A — Foundations (Repo, Infra, Core)

Deliverables
	•	Repo scaffolding; CI (build + tests + lints); formatter; pre-commit hooks.
	•	expr_core: arena store, hash‑consing, interning, canonical Add/Mul/Pow, equality & hashing, basic printer.
	•	arith: BigInt/Rational with canonicalization.
	•	io: S‑expr/JSON serializer; minimal parser for literals, symbols, Add/Mul/Pow, function calls.

Acceptance
	•	Can construct and serialize expressions; pointer equality implies structural equality; canonical ordering verified; 95% unit coverage in core.
	•	Bench: create/simplify 10⁵ small nodes within resource budget.

⸻

Phase B — Baseline Algebra & Simplify v1

Deliverables
	•	Combine like terms/factors; rational normal form (cancel, together).
	•	Substitution v1 (exact head/symbol replacement).
	•	Assumptions skeleton (real/positive/integer flags, 3‑valued).
	•	Printers: pretty + LaTeX.

Acceptance
	•	simplify idempotent under guard (simplify(simplify(e)) == simplify(e)).
	•	Rational normalization: cancel((x^2-1)/(x-1)) = x+1 guarded when x≠1.
	•	Resource guards on.

⸻

Phase C — Polynomials v1

Deliverables
	•	Univariate dense; multivariate sparse; Euclidean division, PRS GCD; square‑free; partial fractions.
	•	Tight conversions Expr↔Poly.

Acceptance
	•	GCD/partial fractions pass canonical algebra tests; round-trip conversions preserve value; microbench shows sub-linear scaling wrt term count.

⸻

Phase D — Calculus I (Diff, Limits, Series)

Deliverables
	•	diff for core ops & standard functions; simplify-on-derive.
	•	Limits heuristics; series expansions for elementary functions with compose/multiply.

Acceptance
	•	d/dx (x^n) = n*x^(n-1); chain/product/quotient tests; basic limit identities; series comparisons with numeric checks.

⸻

Phase E — Linear Algebra I

Deliverables
	•	Exact matrices; fraction‑free Gaussian elimination; determinant; solve linear systems.

Acceptance
	•	Determinant equals product of eigenvalues for numeric specializations; exact solve on integer/rational matrices; performance within budget for k≤20.

⸻

Phase F — Solve I (Univariate)

Deliverables
	•	Factor‑then‑solve for univariate polynomials; root objects for irreducible factors; quadratic–quartic closed forms; numeric isolation fallback.

Acceptance
	•	Verified root substitution back to polynomial (exact/numeric); robust handling of multiplicities; guarded behavior for large degree.

⸻

Phase G — Integration I (Rational + Patterns)

Deliverables
	•	Rational integration via partial fractions; ∫ f’/f; linear u‑sub detection; initial trig/exp patterns.

Acceptance
	•	Canonical antiderivatives for rational functions (derivative check simplifies back); pattern hits validated by differentiation.

⸻

Phase H — Pattern Matching v2 & Rewrite Strategy

Deliverables
	•	AC-aware pattern matching for Add/Mul; constraint predicates; rule registry DSL.
	•	Rewrite scheduler: cost model; termination caps; metrics.

Acceptance
	•	Rules for common identities (e.g., collect like terms; simple trig/log identities) apply deterministically; measured rewrite step bounds respected.

⸻

Phase I — Assumptions v2 & Domain-Aware Simplify

Deliverables
	•	Scoped context; propagate properties; conditionally correct rewrites (e.g., sqrt(x^2) -> |x| unless x≥0).
	•	Piecewise propagation.

Acceptance
	•	Test matrix where domain changes alter simplification; no unsafe rewrites under default (complex) domain.

⸻

Phase J — Integration II & Solve II (Staged)

Deliverables
	•	Heuristic Risch fragments for exp/log towers; integration by parts orchestrator.
	•	Transcendental solves for select classes (LambertW forms), numeric fallback with intervals (if enabled).

Acceptance
	•	Extensive diff-check harness for integrals; certified numeric solve on guarded domains.

⸻

Phase K — WASM & Python Packaging

Deliverables
	•	WASM build with tiny API; Python wheels (manylinux, macOS, Windows).
	•	Docs + examples for embedding.

Acceptance
	•	WASM demo (parse→simplify→print) within size budget; pip install works; API stability documented.

⸻

Phase L — Hardening, Fuzzing, Differential Testing & 1.0

Deliverables
	•	Fuzzing on parser/simplifier/rewriter; differential tests against a reference for well-defined identities; metrics dashboards.
	•	Finalize semver 1.0 API; write migration/compat notes.

Acceptance
	•	Crash-free fuzzing over threshold corpus; rule regressions gated; docs complete; tagged release.

⸻

14) Rule & Function Extension Model
	•	Function registry: to add a function F, provide metadata: arity/variadic, algebraic properties (odd/even, monotonic, branch), derivative rule, evalf routine, simplification hooks.
	•	Rule registry DSL: let contributors define pattern→rewrite with guards (e.g., (a_*x + b_*x) -> (a+b)*x with type/assumption checks).
	•	Module isolation: heavy domains (e.g., special functions, Gröbner) live behind feature flags.

⸻

15) Performance Tactics (throughout)
	•	Hash‑consing + memoization (per‑store caches for expand, gcd, diff).
	•	Avoid expansion by default; provide expand/factor explicitly.
	•	Fraction‑free algorithms (Bareiss) to control intermediate swell.
	•	Parallelizable operations (e.g., map over large Add/Mul child lists) using work‑stealing, with deterministic reduction order.
	•	CSE for output (optional) to shrink printed/generated code.
	•	Bench harness that tracks node counts, rewrite steps, allocation rates, peak RSS.

⸻

16) Risk Register & Mitigations
	•	Expression swell → strict defaults (no auto expand), fraction‑free methods, guards, user‑requested heavy ops only.
	•	Pattern matcher blowups → strategy caps, cost heuristics, rule grouping by head, optional e‑graph only behind flag.
	•	Numeric/library size → feature flags (bigint=on/off, mpfr=on/off), dual backends (pure-Rust vs GMP) to fit environments.
	•	Branch cuts/domain correctness → assumptions default to complex; require guards for real‑only rewrites; add diagnostics.
	•	API churn → public API review gates; semver discipline; deprecation paths.

⸻

17) Initial Backlog (actionable tickets)
	1.	expr_core: Arena + consing table + Node layout + canonical Add/Mul/Pow.
	2.	arith: BigInt/Rational traits + canonicalization; Constant atoms.
	3.	io: S‑expr/JSON serializer + minimal parser; pretty printer.
	4.	simplify: combine-like terms/factors; rational normalization; idempotence checks.
	5.	pattern v1: substitution engine, alpha‑equivalence where relevant.
	6.	polys v1: uni dense + multi sparse; GCD; partial fractions.
	7.	calculus: diff rules; simplify-on-derive.
	8.	matrix: fraction‑free Gaussian elimination; determinant.
	9.	solver v1: univariate solve via factor; quadratic–quartic formulas.
	10.	integration v1: rationals + f’/f + simple trig/exp patterns.
	11.	assumptions v1: 3‑valued lattice; scope; hooks in simplifier.
	12.	bindings: C ABI + Python (pyo3) minimal surface.
	13.	wasm: build + tiny demo.
	14.	QA: property tests; fuzzing seeds; micro/macro benches.

⸻

18) Release Train (capability slices)
	•	Alpha: A–C (core + simplify + polynomials)
	•	Beta: + D–G (diff/limits/series, matrices, univariate solving, rational integration)
	•	RC: + H–I (pattern v2, assumptions v2)
	•	1.0: + J–K–L (integration/solve staging, WASM/Python packaging, hardening)

⸻

19) What “Lightweight & Scalable” means concretely here
	•	Lightweight: small, dependency-lean builds; feature flags to disable heavy algebra (e.g., Groebner, MPFR); no background daemons; pure library + small CLI.
	•	Scalable:
	•	Handles many small subterms efficiently (flat n‑ary Add/Mul, SIMD‑friendly loops, parallel map/reduce over children).
	•	Avoids super‑linear blowups via canonical forms and fraction‑free algorithms.
	•	Stable memory behavior via DAG sharing; expression metrics and guards prevent pathological workloads.

⸻

Done right, this plan yields a compact, safe, high‑performance core you can embed anywhere (CLI, services, browsers via WASM, Python). You get immediate utility at Beta (symbolic algebra, diff/limits/series, linear solve, rational integration), and a clear path to deepen capabilities without compromising the lightweight core.

---

## Post-1.0 Roadmap: Toward a Full-Featured CAS

### Executive Summary

Symmetrica 1.0 provides a **solid foundation** with all core features complete. To reach Mathematica-level capabilities, we need to add:

1. **Advanced Integration** - Full Risch algorithm, special function integrals
2. **Special Functions** - Comprehensive library (Bessel, Gamma, Hypergeometric, etc.)
3. **Advanced Solving** - Multivariate systems, transcendental equations, ODEs/PDEs
4. **Symbolic Summation** - Gosper's algorithm, hypergeometric summation
5. **Number Theory** - Modular arithmetic, primality, factorization
6. **Enhanced Simplification** - Trigonometric identities, radical denesting
7. **Tensor Algebra** - Einstein notation, index manipulation
8. **Algebraic Extensions** - Algebraic numbers, field extensions
9. **Code Generation** - C/Fortran/Julia code output
10. **Interactive Features** - Notebook interface, advanced visualization

### Design Philosophy (Maintained from 1.0)

- **Lightweight:** Modular crates, feature flags, minimal dependencies
- **Deterministic:** Canonical forms, reproducible outputs
- **Safe:** Immutability, resource guards, no unsafe unless necessary
- **Extensible:** Plugin architecture, registry-based functions/rules
- **Fast:** Hash-consing, memoization, fraction-free algorithms
- **Embeddable:** Library-first, multiple language bindings

---

## Phase Overview & Timeline

### v1.x Series (2025-2026) - Consolidation

#### Phase M: Integration v2 (v1.1 - Q1 2026, 6-8 weeks)
**Goal:** Advanced integration techniques

**Deliverables:**
- Risch algorithm for exp/log towers
- Trigonometric integration (Weierstrass substitution)
- Automatic u-substitution detection
- Integration by parts orchestrator

**Dependencies:** polys, calculus  
**Complexity:** Medium

#### Phase N: Special Functions Library (v1.2 - Q2 2026, 12-16 weeks)
**Goal:** Comprehensive special function support

**Deliverables:**
- Gamma, Beta, Error functions with derivatives and series
- Bessel functions (J, Y, I, K) with recurrence relations
- Orthogonal polynomials (Legendre, Chebyshev, Hermite, Laguerre, Jacobi)
- Hypergeometric functions (₁F₁, ₂F₁)

**New Crate:** `crates/special`  
**Dependencies:** calculus, evalf  
**Complexity:** High

#### Phase O: Advanced Equation Solving (v1.3 - Q3 2026, 16-20 weeks)
**Goal:** Solve multivariate systems and transcendental equations

**Deliverables:**
- Gröbner bases (Buchberger's algorithm with optimizations)
- Lambert W function for transcendental equations
- First & second-order ODEs (separable, linear, Bernoulli, constant coefficients)
- Systems of ODEs (matrix exponential method)

**Dependencies:** polys, solver, matrix  
**Complexity:** Very High

#### Phase P: Symbolic Summation & Products (v1.4 - Q4 2026, 10-12 weeks)
**Goal:** Closed-form summation and product evaluation

**Deliverables:**
- Gosper's algorithm for hypergeometric term summation
- Zeilberger's algorithm for creative telescoping
- Basic sums (arithmetic/geometric series, power sums, binomial sums)
- Infinite series convergence tests

**Dependencies:** calculus, special  
**Complexity:** High

### v2.x Series (2026-2028) - Advanced Features

#### Phase Q: Enhanced Simplification (v2.0 - Q1 2027, 12-14 weeks)
**Goal:** Advanced simplification and rewriting

**Deliverables:**
- Trigonometric identities (Pythagorean, double/half angle, sum-to-product)
- Radical denesting (Ramanujan's algorithm)
- Logarithm & exponential rules with branch cut handling
- E-graph based rewriting (optional, behind feature flag)

**Dependencies:** simplify, assumptions  
**Complexity:** High

#### Phase R: Number Theory Module (v2.1 - Q2 2027, 8-10 weeks)
**Goal:** Computational number theory capabilities

**Deliverables:**
- Primality testing (Miller-Rabin, AKS)
- Integer factorization (Pollard's rho, quadratic sieve)
- Modular arithmetic (exponentiation, CRT, discrete logarithm)
- Diophantine equations (linear, Pell's equation)

**New Crate:** `crates/number_theory`  
**Dependencies:** arith, polys  
**Complexity:** Medium

#### Phase S: Tensor Algebra & Differential Geometry (v2.2 - Q3 2027, 16-20 weeks)
**Goal:** Support for tensor computations

**Deliverables:**
- Tensor type with arbitrary rank and Einstein summation
- Metric tensor operations, Christoffel symbols
- Riemann/Ricci tensors, geodesic equations
- Exterior calculus (differential forms, wedge product, exterior derivative)

**New Crate:** `crates/tensor`  
**Dependencies:** matrix, calculus  
**Complexity:** Very High

#### Phase T: Algebraic Extensions & Number Fields (v2.3 - Q4 2027, 14-18 weeks)
**Goal:** Exact computation with algebraic numbers

**Deliverables:**
- Algebraic number representation as polynomial roots
- Field extensions (ℚ(√2), ℚ(i), ℚ(ζ_n))
- Tower of extensions and Galois theory computations
- Algebraic simplification (norm, trace, conjugates)

**Dependencies:** polys, arith  
**Complexity:** Very High

#### Phase U: Code Generation & Compilation (v2.4 - Q1 2028, 10-12 weeks)
**Goal:** Generate optimized code from symbolic expressions

**Deliverables:**
- C, Fortran, Julia code generation backends
- Common subexpression elimination (CSE)
- Automatic differentiation code generation
- LLVM IR (optional, behind feature flag)

**Dependencies:** expr_core, io  
**Complexity:** High

#### Phase V: Interactive Computing & Visualization (v2.5 - Q2 2028, 8-10 weeks)
**Goal:** Enhanced user interaction and visualization

**Deliverables:**
- Jupyter kernel for Symmetrica
- Advanced 2D/3D plotting (parametric, polar, contour, surface plots)
- Interactive visualizations with parameter sliders
- Animation support

**Dependencies:** plot, io, cli  
**Complexity:** Medium

#### Phase W: Performance Optimization (Ongoing)
**Goal:** Continuous performance improvements

**Focus Areas:**
- Parallel polynomial GCD, FFT for polynomial multiplication
- Memory optimization (arena tuning, garbage collection)
- Memoization expansion to more operations
- SIMD & parallelization, GPU acceleration (experimental)

**Timeline:** Continuous

#### Phase X: Advanced Pattern Matching (v2.6 - Q3 2028, 14-16 weeks)
**Goal:** Mathematica-level pattern matching and rewriting

**Deliverables:**
- Advanced pattern syntax (sequence patterns, conditional patterns, alternatives)
- Associative-commutative matching for Add/Mul
- Transformation rules with strategies (innermost, outermost)
- User-defined simplification rules with conflict detection

**Dependencies:** pattern, simplify  
**Complexity:** Very High

### v3.x Series (2028+) - Research Features

#### Phase Y: Partial Differential Equations (v3.0 - Q4 2028, 20-24 weeks)
**Goal:** Symbolic PDE solving

**Deliverables:**
- First & second-order PDEs classification and canonical forms
- Separation of variables (Cartesian, cylindrical, spherical)
- Transform methods (Fourier, Laplace, Hankel)
- Special cases (heat, wave, Laplace, Schrödinger equations)

**Dependencies:** calculus, solver, special  
**Complexity:** Very High

#### Phase Z: Probability & Statistics (v3.1 - Q1 2029, 8-10 weeks)
**Goal:** Symbolic probability and statistics

**Deliverables:**
- Probability distributions (discrete & continuous)
- Random variable algebra (expectation, variance, MGF)
- Statistical tests (hypothesis testing, confidence intervals)
- Distribution fitting

**Dependencies:** special, calculus  
**Complexity:** Medium

---

## Comparison with Other CAS Systems

### Feature Parity Roadmap

| Feature | Mathematica | Symmetrica 1.0 | Symmetrica 3.0 (Target) |
|---------|-------------|----------------|-------------------------|
| Basic Algebra | ✅ | ✅ | ✅ |
| Differentiation | ✅ | ✅ | ✅ |
| Integration | ✅✅✅ | ✅ (40%) | ✅✅✅ (95%) |
| Special Functions | ✅✅✅ | ❌ (0%) | ✅✅ (80%) |
| Polynomial Algebra | ✅ | ✅ (90%) | ✅ (100%) |
| Linear Algebra | ✅ | ✅ (90%) | ✅ (100%) |
| Equation Solving | ✅✅ | ✅ (50%) | ✅✅ (85%) |
| Pattern Matching | ✅✅✅ | ✅ (30%) | ✅✅✅ (95%) |
| ODEs | ✅✅✅ | ❌ (0%) | ✅✅ (80%) |
| PDEs | ✅✅ | ❌ (0%) | ✅ (60%) |
| Number Theory | ✅✅ | ❌ (0%) | ✅✅ (90%) |
| Tensor Algebra | ✅✅ | ❌ (0%) | ✅ (70%) |
| Code Generation | ✅✅ | ❌ (0%) | ✅✅ (90%) |
| Visualization | ✅✅✅ | ✅ (20%) | ✅✅ (75%) |

**Overall Capability:**
- **v1.0:** ~45% of Mathematica
- **v2.0:** ~70% of Mathematica  
- **v3.0:** ~85% of Mathematica

### Key Differentiators

**Symmetrica's Advantages:**
1. ✅ **Free & Open Source** (MIT/Apache vs $1,000+/year)
2. ✅ **Embeddable & Lightweight** (15MB vs 5GB)
3. ✅ **Memory Safe** (Rust guarantees)
4. ✅ **Native WASM Support** (browser deployment)
5. ✅ **High Performance** (10-100x faster than SymPy)
6. ✅ **Modern Language & Tools** (Rust ecosystem)

**vs. SymPy:**
- 10-100x faster performance
- Memory safety guarantees
- Native WASM support
- Smaller deployment size

**vs. Maxima:**
- Modern language (Rust vs Lisp)
- Active development
- Better documentation
- WASM support

**vs. Commercial CAS:**
- Free and open source
- Embeddable in applications
- Community-driven development
- No vendor lock-in

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

## Resource Requirements

### Development Team
- **Core Team:** 3-5 full-time developers
- **Contributors:** 20-50 part-time contributors
- **Reviewers:** 5-10 experienced reviewers
- **Documentation:** 2-3 technical writers

### Timeline Summary
- **v1.x Series:** 12-18 months (2025-2026)
- **v2.x Series:** 18-24 months (2026-2028)
- **v3.x Series:** 12+ months (2028+)
- **Total to v3.0:** ~3.5-4 years

---

## Contributing

### How to Get Involved

1. **Pick a Phase:** Choose a phase that interests you from the roadmap above
2. **Study Algorithms:** Review academic literature and prior art (see [research.md](research.md))
3. **Prototype:** Create proof-of-concept implementations
4. **Test:** Write comprehensive tests with property-based testing
5. **Document:** Explain design choices and algorithms
6. **Submit:** Open PRs with clear descriptions

### Resources
- **Research Notes:** [research.md](research.md) - Algorithm references and design notes
- **API Documentation:** Generate with `cargo doc --workspace --no-deps --open`
- **Examples:** See `examples/` directory for usage patterns
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

**Document Version:** 2.0 (Consolidated)  
**Last Updated:** October 2025  
**Status:** Living Document