# Quick Start: Next Implementation Tasks

**Goal:** Get started immediately on completing Phase 4 and expanding Phases 7-9  
**Time to First Contribution:** ~30 minutes

---

## 🚀 Immediate Tasks (Pick One to Start)

### Option A: Complete Gröbner Basis Solving (HIGH IMPACT)
**Estimated Time:** 4-6 hours  
**Difficulty:** Medium-Hard  
**Impact:** Enables multivariate polynomial system solving

```bash
# 1. Create test file first (TDD approach)
cat > crates/grobner/tests/buchberger.rs << 'EOF'
use expr_core::Store;
use grobner::{buchberger, MonomialOrder};

#[test]
fn test_buchberger_simple_2var() {
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");
    
    // System: x^2 + y - 1 = 0, x + y^2 - 1 = 0
    let two = st.int(2);
    let one = st.int(1);
    let neg_one = st.int(-1);
    
    let x2 = st.pow(x, two);
    let y2 = st.pow(y, two);
    
    let eq1 = st.add(vec![x2, y, neg_one]);  // x^2 + y - 1
    let eq2 = st.add(vec![x, y2, neg_one]);  // x + y^2 - 1
    
    let basis = buchberger(&mut st, vec![eq1, eq2], &["x".to_string(), "y".to_string()], MonomialOrder::Lex);
    
    // Basis should be in triangular form for lex ordering
    assert!(!basis.is_empty());
    println!("Gröbner basis: {:?}", basis.iter().map(|&e| st.to_string(e)).collect::<Vec<_>>());
}
EOF

# 2. Run test (it will fail)
cargo test --package grobner buchberger

# 3. Implement buchberger() in crates/grobner/src/lib.rs
# See IMPLEMENTATION_PLAN.md section 4.1 for algorithm details

# 4. Iterate until test passes
```

**Key Algorithm Steps:**
1. Initialize basis G = generators
2. Compute all S-polynomials S(fi, fj)
3. Reduce each S-polynomial by current basis
4. If remainder ≠ 0, add to basis
5. Repeat until no new polynomials added

**Resources:**
- Cox, Little, O'Shea - "Ideals, Varieties, and Algorithms" Chapter 2
- Your existing `s_polynomial()` and `reduce()` functions

---

### Option B: Add Bernoulli ODE Pattern (QUICK WIN)
**Estimated Time:** 2-3 hours  
**Difficulty:** Easy-Medium  
**Impact:** Expands ODE solving capabilities

```bash
# 1. Add test cases first
cat >> crates/calculus/src/ode.rs << 'EOF'

#[test]
fn test_bernoulli_simple() {
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");
    
    // dy/dx + y = y^2 (Bernoulli with n=2)
    // Solution: y = 1/(1 - Ce^x)
    let two = st.int(2);
    let y2 = st.pow(y, two);
    let rhs = st.add(vec![y2, store.mul(vec![st.int(-1), y])]);
    
    let solution = solve_ode_first_order(&mut st, rhs, "y", "x");
    assert!(solution.is_some());
}
EOF

# 2. Implement try_bernoulli() function
# Add before try_separable() in crates/calculus/src/ode.rs

# 3. Wire into solve_ode_first_order() dispatcher
```

**Key Algorithm Steps:**
1. Recognize form: dy/dx + p(x)y = q(x)y^n
2. Substitute v = y^(1-n)
3. Transform to linear ODE: dv/dx + (1-n)p(x)v = (1-n)q(x)
4. Solve using existing try_linear()
5. Back-substitute: y = v^(1/(1-n))

---

### Option C: Enhance Number Theory Factorization (MEDIUM IMPACT)
**Estimated Time:** 3-4 hours  
**Difficulty:** Medium  
**Impact:** Faster factorization for large numbers

```bash
# 1. Create new module
cat > crates/number_theory/src/factorization.rs << 'EOF'
//! Integer Factorization Algorithms

/// Trial division with 2,3,5 wheel
pub fn trial_division(n: u64, limit: u64) -> Vec<(u64, u32)> {
    let mut factors = Vec::new();
    let mut n = n;
    
    // Handle 2
    let mut count = 0;
    while n % 2 == 0 {
        count += 1;
        n /= 2;
    }
    if count > 0 {
        factors.push((2, count));
    }
    
    // Handle 3
    count = 0;
    while n % 3 == 0 {
        count += 1;
        n /= 3;
    }
    if count > 0 {
        factors.push((3, count));
    }
    
    // Wheel: 6k±1 pattern
    let mut d = 5;
    let limit = limit.min((n as f64).sqrt() as u64 + 1);
    while d <= limit && n > 1 {
        count = 0;
        while n % d == 0 {
            count += 1;
            n /= d;
        }
        if count > 0 {
            factors.push((d, count));
        }
        
        // Next candidate: 6k+1 -> 6k+5 -> 6(k+1)+1
        d = if d % 6 == 1 { d + 4 } else { d + 2 };
    }
    
    if n > 1 {
        factors.push((n, 1));
    }
    
    factors
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_trial_division_small() {
        assert_eq!(trial_division(12, 100), vec![(2, 2), (3, 1)]);
        assert_eq!(trial_division(17, 100), vec![(17, 1)]);
        assert_eq!(trial_division(100, 100), vec![(2, 2), (5, 2)]);
    }
}
EOF

# 2. Add to lib.rs
echo "pub mod factorization;" >> crates/number_theory/src/lib.rs

# 3. Test
cargo test --package number_theory factorization
```

---

## 📋 Development Checklist

Before starting any task:
- [ ] Read the relevant section in `IMPLEMENTATION_PLAN.md`
- [ ] Create test file with failing tests
- [ ] Understand the mathematical algorithm
- [ ] Check for existing helper functions you can reuse

While implementing:
- [ ] Write clear comments explaining the algorithm
- [ ] Add doc comments with examples
- [ ] Handle edge cases (zero, negative, overflow)
- [ ] Use meaningful variable names

Before committing:
- [ ] Run `cargo fmt --all`
- [ ] Run `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] Run `cargo test --workspace --all-features`
- [ ] Verify all tests pass
- [ ] Update `CHANGELOG.md` if needed

---

## 🎯 This Week's Goals

### Minimum (1-2 tasks)
- [ ] Complete Gröbner basis Buchberger algorithm
- [ ] Add 2-3 ODE patterns (Bernoulli, exact, or homogeneous)

### Target (3-4 tasks)
- [ ] Complete Gröbner basis + solve_system()
- [ ] Add 4-5 ODE patterns
- [ ] Enhance factorization with trial division
- [ ] Add 10+ test cases for each

### Stretch (5+ tasks)
- [ ] Complete all Phase 4 ODE patterns
- [ ] Add Einstein summation notation
- [ ] Implement general algebraic number arithmetic
- [ ] Add comprehensive benchmarks

---

## 🔧 Useful Commands

### Run specific test
```bash
cargo test --package grobner test_buchberger_simple_2var
```

### Run tests with output
```bash
cargo test --package calculus ode -- --nocapture
```

### Check a specific file
```bash
cargo clippy --package grobner -- -D warnings
```

### Run benchmarks (after setup)
```bash
cargo bench --package grobner
```

### Generate documentation
```bash
cargo doc --package grobner --open
```

---

## 📚 Quick Reference

### Existing Helper Functions You Can Use

**In `crates/grobner/src/lib.rs`:**
- `Monomial::from_expr()` - extract monomial from expression
- `s_polynomial()` - compute S-polynomial of two polynomials
- `reduce()` - reduce polynomial by basis

**In `crates/calculus/src/integrate.rs`:**
- `integrate()` - integrate expression w.r.t. variable
- Useful for ODE solving

**In `crates/calculus/src/diff.rs`:**
- `diff()` - differentiate expression
- Useful for checking exactness in ODEs

**In `crates/expr_core/src/lib.rs`:**
- `Store::add()`, `Store::mul()`, `Store::pow()` - build expressions
- `Store::get()` - inspect expression structure
- `simplify::simplify()` - simplify expressions

---

## 💡 Tips for Success

1. **Start Small:** Pick the easiest task first to build momentum
2. **Test-Driven:** Write tests before implementation
3. **Incremental:** Commit working code frequently
4. **Ask Questions:** If stuck, review similar code in the codebase
5. **Document:** Write clear comments and doc strings
6. **Verify:** Always run full test suite before pushing

---

## 🐛 Common Pitfalls

### Borrow Checker Issues
```rust
// ❌ Bad: nested mutable borrows
let result = store.add(vec![x, store.mul(vec![y, z])]);

// ✅ Good: precompute intermediate values
let yz = store.mul(vec![y, z]);
let result = store.add(vec![x, yz]);
```

### Pattern Matching
```rust
// ❌ Bad: doesn't handle all cases
match store.get(expr).op {
    Op::Add => { /* ... */ }
    _ => panic!("unexpected op")
}

// ✅ Good: return None for unhandled cases
match store.get(expr).op {
    Op::Add => { /* ... */ }
    _ => return None,
}
```

### Integer Overflow
```rust
// ❌ Bad: can overflow
let fact = (1..=n).product::<i64>();

// ✅ Good: use saturating arithmetic or i128
let mut fact = 1i128;
for i in 1..=n {
    fact = fact.saturating_mul(i as i128);
    if fact == 0 { return None; }
}
```

---

## 📞 Need Help?

1. **Check existing code:** Look for similar functions in the codebase
2. **Read tests:** Existing tests show usage patterns
3. **Review IMPLEMENTATION_PLAN.md:** Detailed algorithm descriptions
4. **Check documentation:** `cargo doc --open`

---

**Ready to start? Pick a task above and dive in!** 🚀

Remember: The goal is progress, not perfection. Start with something simple, get it working, then iterate.
