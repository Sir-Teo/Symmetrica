# Implementation Plan: Phases 4, 7-9

**Goal:** Complete Phase 4 (50% → 100%) and expand Phases 7-9 (30% → 80%)  
**Timeline:** 4-6 weeks of focused development  
**Current Status:** Phase 4 at 50%, Phase 7 at 30%, Phase 8 at 20%, Phase 9 at 15%

---

## Phase 4: Advanced Equation Solving (50% → 100%)

### Current State ✅
- Gröbner basis infrastructure (monomial ordering, S-polynomials, reduction)
- Lambert W function for transcendental equations
- First-order ODE patterns (separable, linear)

### Remaining Work 🎯

#### 4.1: Complete Gröbner Basis Solving (Week 1-2)
**Priority: HIGH**

**File:** `crates/grobner/src/lib.rs`

**Tasks:**
1. **Implement `buchberger()` algorithm**
   ```rust
   pub fn buchberger(
       store: &mut Store,
       generators: Vec<ExprId>,
       vars: &[String],
       order: MonomialOrder,
   ) -> Vec<ExprId>
   ```
   - Compute S-polynomials for all pairs
   - Reduce and add non-zero remainders
   - Implement Buchberger's criteria (product criterion, chain criterion)
   - Add sugar cube optimization for efficiency

2. **Implement `solve_system()` with back-substitution**
   ```rust
   pub fn solve_system(
       store: &mut Store,
       equations: Vec<ExprId>,
       vars: Vec<String>,
   ) -> Option<Vec<HashMap<String, ExprId>>>
   ```
   - Compute Gröbner basis with lex ordering
   - Check for triangular form
   - Back-substitute from last variable to first
   - Handle multiple solutions (parametric families)
   - Return `None` for inconsistent systems

3. **Add elimination ideal computation**
   ```rust
   pub fn eliminate(
       store: &mut Store,
       basis: Vec<ExprId>,
       vars_to_eliminate: &[String],
       all_vars: &[String],
   ) -> Vec<ExprId>
   ```

**Tests:**
- `tests/grobner_buchberger.rs` - test Buchberger on standard examples
- `tests/grobner_solving.rs` - 2-3 variable systems with known solutions
- Verify against SymPy/Mathematica results

**Acceptance Criteria:**
- Solve 2-3 variable polynomial systems in <1s
- Handle inconsistent systems gracefully
- 20+ test cases passing

---

#### 4.2: Expand ODE Solving (Week 2-3)
**Priority: HIGH**

**File:** `crates/calculus/src/ode.rs`

**Tasks:**

1. **Add Bernoulli equations**
   ```rust
   fn try_bernoulli(
       store: &mut Store,
       rhs: ExprId,
       y_var: &str,
       x_var: &str,
   ) -> Option<ExprId>
   ```
   - Recognize form: dy/dx + p(x)y = q(x)y^n
   - Transform via v = y^(1-n)
   - Reduce to linear ODE

2. **Add exact equations**
   ```rust
   fn try_exact(
       store: &mut Store,
       m: ExprId,  // M(x,y)
       n: ExprId,  // N(x,y)
       y_var: &str,
       x_var: &str,
   ) -> Option<ExprId>
   ```
   - Check exactness: ∂M/∂y = ∂N/∂x
   - Find potential function F(x,y)
   - Return implicit solution F(x,y) = C

3. **Add homogeneous equations**
   ```rust
   fn try_homogeneous(
       store: &mut Store,
       rhs: ExprId,
       y_var: &str,
       x_var: &str,
   ) -> Option<ExprId>
   ```
   - Recognize form: dy/dx = f(y/x)
   - Substitute v = y/x
   - Reduce to separable

4. **Add second-order constant coefficients**
   ```rust
   pub fn solve_ode_second_order_constant_coeff(
       store: &mut Store,
       a: ExprId,  // coefficient of y''
       b: ExprId,  // coefficient of y'
       c: ExprId,  // coefficient of y
       rhs: ExprId, // right-hand side
       y_var: &str,
       x_var: &str,
   ) -> Option<ExprId>
   ```
   - Solve characteristic equation: ar² + br + c = 0
   - Handle distinct real roots, repeated roots, complex roots
   - Add particular solution for non-homogeneous
   - Use variation of parameters or undetermined coefficients

**Tests:**
- `tests/ode_bernoulli.rs` - 10+ Bernoulli equations
- `tests/ode_exact.rs` - 10+ exact equations
- `tests/ode_second_order.rs` - 15+ second-order ODEs
- Verify solutions by substitution

**Acceptance Criteria:**
- Solve 80% of standard textbook first-order ODEs
- Solve constant coefficient second-order ODEs
- 50+ ODE test cases passing
- Solution verification by differentiation

---

#### 4.3: Transcendental Equation Patterns (Week 3)
**Priority: MEDIUM**

**File:** `crates/solver/src/transcendental.rs` (new)

**Tasks:**

1. **Pattern matcher for Lambert W forms**
   ```rust
   pub fn solve_transcendental(
       store: &mut Store,
       equation: ExprId,
       var: &str,
   ) -> Option<Vec<ExprId>>
   ```
   - Detect a·e^(bx) = c·x + d
   - Transform to Lambert W form
   - Return solutions using LambertW function

2. **Add inverse trig patterns**
   - arcsin(x) = a → x = sin(a)
   - arctan(f(x)) = arctan(g(x)) → f(x) = g(x)

3. **Add logarithmic equation patterns**
   - log(f(x)) = a → f(x) = e^a
   - log(f(x)) = log(g(x)) → f(x) = g(x)

**Tests:**
- `tests/transcendental_solve.rs` - 20+ transcendental equations

**Acceptance Criteria:**
- Solve common transcendental forms
- Return symbolic solutions when possible
- 20+ test cases passing

---

## Phase 7: Number Theory (30% → 80%)

### Current State ✅
- Miller-Rabin primality test
- Modular inverse
- Chinese Remainder Theorem
- Experimental factorization (Pollard's rho)

### Remaining Work 🎯

#### 7.1: Enhanced Factorization (Week 4)
**Priority: HIGH**

**File:** `crates/number_theory/src/factorization.rs` (new)

**Tasks:**

1. **Trial division with wheel factorization**
   ```rust
   pub fn trial_division(n: u64, limit: u64) -> Vec<(u64, u32)>
   ```
   - Use 2, 3, 5 wheel to skip multiples
   - Factor up to √n or specified limit
   - Return prime factorization

2. **Quadratic sieve (basic implementation)**
   ```rust
   pub fn quadratic_sieve(n: u64) -> Vec<(u64, u32)>
   ```
   - For numbers > 10^12
   - Sieve smooth numbers
   - Find relations and solve linear algebra

3. **Unified factorization interface**
   ```rust
   pub fn factor(n: u64) -> Vec<(u64, u32)>
   ```
   - Auto-select algorithm based on size
   - Trial division for small n
   - Pollard's rho for medium n
   - Quadratic sieve for large n

**Tests:**
- Factor known semiprimes
- Verify against OEIS factorizations
- Performance benchmarks

**Acceptance Criteria:**
- Factor 20-digit numbers in <10s
- Correct factorization for 100+ test cases
- Handle edge cases (primes, powers of 2, etc.)

---

#### 7.2: Diophantine Equations (Week 4-5)
**Priority: MEDIUM**

**File:** `crates/number_theory/src/diophantine.rs` (new)

**Tasks:**

1. **Linear Diophantine equations**
   ```rust
   pub fn solve_linear_diophantine(
       a: i64,
       b: i64,
       c: i64,
   ) -> Option<(i64, i64)> // (x0, y0) particular solution
   ```
   - ax + by = c
   - Use extended Euclidean algorithm
   - Return general solution (x0 + bt, y0 - at)

2. **Pell's equation**
   ```rust
   pub fn solve_pell(d: u64) -> Option<(u64, u64)> // (x, y)
   ```
   - x² - dy² = 1
   - Use continued fractions
   - Find fundamental solution

3. **Pythagorean triples**
   ```rust
   pub fn pythagorean_triples(limit: u64) -> Vec<(u64, u64, u64)>
   ```
   - Generate primitive triples
   - Use parametric form: (m²-n², 2mn, m²+n²)

**Tests:**
- Known Diophantine solutions
- Pell equation for small d
- Pythagorean triples verification

**Acceptance Criteria:**
- Solve linear Diophantine in O(log n)
- Find Pell solutions for d < 1000
- 30+ test cases passing

---

#### 7.3: Modular Arithmetic Extensions (Week 5)
**Priority: MEDIUM**

**File:** `crates/number_theory/src/modular.rs` (new)

**Tasks:**

1. **Modular exponentiation**
   ```rust
   pub fn mod_pow(base: u64, exp: u64, modulus: u64) -> u64
   ```
   - Binary exponentiation
   - Handle large exponents

2. **Quadratic residues**
   ```rust
   pub fn is_quadratic_residue(a: i64, p: u64) -> bool
   pub fn tonelli_shanks(n: u64, p: u64) -> Option<u64>
   ```
   - Legendre symbol computation
   - Tonelli-Shanks algorithm for square roots mod p

3. **Discrete logarithm (baby-step giant-step)**
   ```rust
   pub fn discrete_log(base: u64, target: u64, modulus: u64) -> Option<u64>
   ```
   - For small moduli
   - Baby-step giant-step algorithm

**Tests:**
- Modular exponentiation correctness
- Quadratic residue detection
- Discrete log for small primes

**Acceptance Criteria:**
- Fast modular exponentiation (O(log n))
- Correct quadratic residue tests
- 25+ test cases passing

---

## Phase 8: Tensor Algebra (20% → 80%)

### Current State ✅
- `Tensor<T>` type with basic operations
- reshape, permute_axes, outer, contract, dot, matmul, trace

### Remaining Work 🎯

#### 8.1: Einstein Summation (Week 5-6)
**Priority: HIGH**

**File:** `crates/tensor/src/einstein.rs` (new)

**Tasks:**

1. **Einstein notation parser**
   ```rust
   pub fn einsum(
       notation: &str,
       tensors: Vec<&Tensor<i64>>,
   ) -> Result<Tensor<i64>, String>
   ```
   - Parse notation like "ij,jk->ik" (matrix multiply)
   - Parse "ii->" (trace)
   - Parse "ij,ij->" (inner product)

2. **Automatic index contraction**
   - Identify repeated indices
   - Determine output shape
   - Perform efficient contraction

3. **Common operations via einsum**
   - Matrix multiply: "ij,jk->ik"
   - Batch matrix multiply: "bij,bjk->bik"
   - Outer product: "i,j->ij"
   - Trace: "ii->"
   - Transpose: "ij->ji"

**Tests:**
- Verify against NumPy einsum
- Performance benchmarks
- Complex multi-tensor contractions

**Acceptance Criteria:**
- Support standard Einstein notation
- Correct results for 50+ einsum patterns
- Performance within 2x of optimized loops

---

#### 8.2: Symbolic Tensor Operations (Week 6)
**Priority: MEDIUM**

**File:** `crates/tensor/src/symbolic.rs` (new)

**Tasks:**

1. **Tensor with symbolic entries**
   ```rust
   pub struct SymbolicTensor {
       shape: Vec<usize>,
       data: Vec<ExprId>,
       store: Store,
   }
   ```
   - Store ExprId instead of concrete values
   - Symbolic tensor operations

2. **Covariant/contravariant indices**
   ```rust
   pub enum IndexType {
       Covariant,    // Lower index
       Contravariant, // Upper index
   }
   
   pub struct IndexedTensor {
       tensor: SymbolicTensor,
       indices: Vec<(String, IndexType)>,
   }
   ```

3. **Metric tensor operations**
   ```rust
   pub fn raise_index(tensor: &IndexedTensor, metric: &SymbolicTensor, idx: usize) -> IndexedTensor
   pub fn lower_index(tensor: &IndexedTensor, metric: &SymbolicTensor, idx: usize) -> IndexedTensor
   ```

**Tests:**
- Symbolic tensor algebra
- Index raising/lowering
- Metric tensor transformations

**Acceptance Criteria:**
- Symbolic tensor operations work correctly
- Index type tracking
- 30+ test cases passing

---

#### 8.3: Differential Geometry Basics (Week 6)
**Priority: LOW**

**File:** `crates/tensor/src/geometry.rs` (new)

**Tasks:**

1. **Christoffel symbols**
   ```rust
   pub fn christoffel_symbols(
       metric: &SymbolicTensor,
       coords: &[String],
   ) -> SymbolicTensor
   ```
   - Compute Γ^k_ij from metric tensor
   - Symbolic differentiation of metric

2. **Riemann curvature tensor**
   ```rust
   pub fn riemann_tensor(
       christoffel: &SymbolicTensor,
       coords: &[String],
   ) -> SymbolicTensor
   ```
   - Compute R^ρ_σμν from Christoffel symbols

**Tests:**
- Flat space (Γ = 0)
- Sphere metric
- Known curvature tensors

**Acceptance Criteria:**
- Correct Christoffel symbols for standard metrics
- Riemann tensor for simple cases
- 10+ test cases passing

---

## Phase 9: Algebraic Extensions (15% → 80%)

### Current State ✅
- `Quad` type for Q(√d)
- Basic arithmetic operations

### Remaining Work 🎯

#### 9.1: General Algebraic Numbers (Week 7)
**Priority: HIGH**

**File:** `crates/algebraic/src/algebraic_number.rs` (new)

**Tasks:**

1. **Algebraic number representation**
   ```rust
   pub struct AlgebraicNumber {
       minimal_poly: Vec<Rational>,  // Coefficients of minimal polynomial
       approx: f64,                   // Numerical approximation for ordering
       store: Store,
       symbolic: ExprId,              // Symbolic representation
   }
   ```
   - Represent as root of minimal polynomial
   - Store numerical approximation for comparisons

2. **Arithmetic operations**
   ```rust
   impl Add for AlgebraicNumber
   impl Mul for AlgebraicNumber
   impl Neg for AlgebraicNumber
   ```
   - Resultant-based multiplication
   - Polynomial arithmetic in quotient ring

3. **Minimal polynomial computation**
   ```rust
   pub fn minimal_polynomial(
       expr: ExprId,
       store: &Store,
   ) -> Option<Vec<Rational>>
   ```
   - Recognize algebraic expressions
   - Compute minimal polynomial
   - Use Groebner basis for elimination

**Tests:**
- √2 + √3 operations
- Nested radicals
- Verify minimal polynomials

**Acceptance Criteria:**
- Correct arithmetic for common algebraic numbers
- Minimal polynomial computation
- 40+ test cases passing

---

#### 9.2: Algebraic Number Recognition (Week 7)
**Priority: MEDIUM**

**File:** `crates/algebraic/src/recognition.rs` (new)

**Tasks:**

1. **Recognize algebraic numbers from expressions**
   ```rust
   pub fn recognize_algebraic(
       expr: ExprId,
       store: &Store,
   ) -> Option<AlgebraicNumber>
   ```
   - Detect nested radicals
   - Detect roots of polynomials
   - Simplify to canonical form

2. **Denesting radicals**
   ```rust
   pub fn denest_radical(
       expr: ExprId,
       store: &mut Store,
   ) -> Option<ExprId>
   ```
   - √(a + b√c) → √d + √e when possible
   - Use Ramanujan's denesting algorithm

3. **Algebraic number simplification**
   ```rust
   pub fn simplify_algebraic(
       num: &AlgebraicNumber,
   ) -> AlgebraicNumber
   ```
   - Reduce to minimal degree
   - Rationalize denominators

**Tests:**
- Denesting examples
- Recognition of standard algebraic numbers
- Simplification correctness

**Acceptance Criteria:**
- Denest common radical expressions
- Recognize algebraic numbers from expressions
- 30+ test cases passing

---

#### 9.3: Field Extensions (Week 8)
**Priority: MEDIUM**

**File:** `crates/algebraic/src/field_extension.rs` (new)

**Tasks:**

1. **Field extension representation**
   ```rust
   pub struct FieldExtension {
       base_field: Box<dyn Field>,
       generator: AlgebraicNumber,
       degree: usize,
   }
   ```
   - Represent K(α) where α is algebraic over K
   - Tower of extensions

2. **Degree and basis computation**
   ```rust
   pub fn extension_degree(ext: &FieldExtension) -> usize
   pub fn power_basis(ext: &FieldExtension) -> Vec<AlgebraicNumber>
   ```
   - Compute [K(α):K]
   - Generate power basis {1, α, α², ..., α^(n-1)}

3. **Galois group (simple cases)**
   ```rust
   pub fn galois_group(
       poly: &[Rational],
   ) -> Option<Vec<Permutation>>
   ```
   - For quadratic and cubic polynomials
   - Compute automorphisms

**Tests:**
- Q(√2), Q(∛2), Q(i) extensions
- Degree computations
- Galois groups for small polynomials

**Acceptance Criteria:**
- Correct field extension operations
- Degree and basis computation
- 25+ test cases passing

---

## Testing Strategy

### Unit Tests
- Each new function gets 3-5 unit tests
- Test edge cases (zero, negative, large values)
- Test error handling

### Integration Tests
- E2E tests in `crates/tests_e2e/tests/`
- `phase4_complete.rs` - Gröbner and ODE solving
- `phase7_complete.rs` - Number theory operations
- `phase8_complete.rs` - Tensor operations
- `phase9_complete.rs` - Algebraic number operations

### Property-Based Tests
- Use `proptest` for algebraic properties
- Verify identities hold symbolically
- Test commutativity, associativity, distributivity

### Performance Benchmarks
- Add `benches/` directory with criterion benchmarks
- Track performance regressions
- Target: <1s for typical operations

---

## Documentation

### API Documentation
- Rustdoc for all public functions
- Examples in doc comments
- Link to mathematical references (DLMF, Wikipedia)

### User Guide
- Add `docs/user_guide.md` with examples
- Tutorial for each phase
- Common use cases and patterns

### Developer Guide
- Add `docs/developer_guide.md`
- Architecture overview
- Contributing guidelines
- Testing best practices

---

## Success Metrics

### Phase 4 Complete (100%)
- ✅ Gröbner basis solving works for 3-variable systems
- ✅ 50+ ODE test cases passing
- ✅ Transcendental equation patterns implemented
- ✅ Solution verification by substitution

### Phase 7 at 80%
- ✅ Factorization for 20-digit numbers
- ✅ Diophantine equation solving
- ✅ Modular arithmetic extensions
- ✅ 80+ number theory test cases passing

### Phase 8 at 80%
- ✅ Einstein summation notation
- ✅ Symbolic tensor operations
- ✅ Differential geometry basics
- ✅ 70+ tensor test cases passing

### Phase 9 at 80%
- ✅ General algebraic number arithmetic
- ✅ Minimal polynomial computation
- ✅ Field extension operations
- ✅ 95+ algebraic test cases passing

### Overall
- ✅ All CI checks passing (fmt, clippy, tests, docs, audit, coverage)
- ✅ Coverage ≥80% for new code
- ✅ Documentation complete for all public APIs
- ✅ Performance benchmarks established

---

## Timeline Summary

| Week | Focus | Deliverables |
|------|-------|--------------|
| 1-2 | Phase 4: Gröbner | Buchberger algorithm, solve_system |
| 2-3 | Phase 4: ODE | Bernoulli, exact, homogeneous, 2nd order |
| 3 | Phase 4: Transcendental | Lambert W patterns, inverse trig |
| 4 | Phase 7: Factorization | Trial division, quadratic sieve |
| 4-5 | Phase 7: Diophantine | Linear, Pell, Pythagorean triples |
| 5 | Phase 7: Modular | mod_pow, quadratic residues, discrete log |
| 5-6 | Phase 8: Einstein | einsum notation, contractions |
| 6 | Phase 8: Symbolic | Symbolic tensors, covariant/contravariant |
| 6 | Phase 8: Geometry | Christoffel symbols, Riemann tensor |
| 7 | Phase 9: Algebraic | General algebraic numbers, operations |
| 7 | Phase 9: Recognition | Denesting, simplification |
| 8 | Phase 9: Extensions | Field extensions, Galois groups |

**Total Estimated Time:** 6-8 weeks of focused development

---

## Getting Started

### Immediate Next Steps (This Week)

1. **Start with Gröbner basis completion** (highest impact)
   ```bash
   # Create new test file
   touch crates/grobner/tests/buchberger.rs
   
   # Implement buchberger() in crates/grobner/src/lib.rs
   # Start with simple 2-variable examples
   ```

2. **Add Bernoulli ODE pattern** (quick win)
   ```bash
   # Extend crates/calculus/src/ode.rs
   # Add try_bernoulli() function
   # Add 5-10 test cases
   ```

3. **Set up benchmarking infrastructure**
   ```bash
   mkdir benches
   touch benches/grobner_bench.rs
   # Add criterion to Cargo.toml
   ```

### Development Workflow

1. **Pick a task from the plan**
2. **Write tests first** (TDD approach)
3. **Implement the feature**
4. **Run local CI checks:**
   ```bash
   cargo fmt --all
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace --all-features
   cargo doc --workspace --no-deps
   ```
5. **Commit and push**
6. **Verify CI passes on GitHub**

---

## Questions to Consider

1. **Performance vs. Correctness:** Should we optimize early or focus on correctness first?
   - **Recommendation:** Correctness first, then profile and optimize

2. **Symbolic vs. Numeric:** How much numeric computation should we support?
   - **Recommendation:** Primarily symbolic, numeric only for validation

3. **Dependencies:** Should we use external crates for complex algorithms?
   - **Recommendation:** Minimize dependencies, implement core algorithms ourselves

4. **API Stability:** When should we commit to stable APIs?
   - **Recommendation:** Mark as experimental until Phase 10 (v2.0)

---

## Resources

### Mathematical References
- **Gröbner Bases:** Cox, Little, O'Shea - "Ideals, Varieties, and Algorithms"
- **ODEs:** Boyce & DiPrima - "Elementary Differential Equations"
- **Number Theory:** Hardy & Wright - "An Introduction to the Theory of Numbers"
- **Tensors:** Wald - "General Relativity" (Appendix on tensors)
- **Algebraic Numbers:** Stewart & Tall - "Algebraic Number Theory"

### Software References
- **SymPy:** Python symbolic math (reference implementation)
- **SageMath:** Comprehensive CAS (algorithm ideas)
- **Mathematica:** Commercial CAS (feature comparison)
- **Maxima:** Open-source CAS (algorithm reference)

### Testing Resources
- **OEIS:** Online Encyclopedia of Integer Sequences (test data)
- **DLMF:** Digital Library of Mathematical Functions (special functions)
- **Wolfram MathWorld:** Mathematical definitions and examples

---

**Last Updated:** 2025-10-07  
**Status:** Ready for implementation  
**Next Review:** After Week 2 (Gröbner completion)
