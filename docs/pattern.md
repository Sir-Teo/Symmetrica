# pattern - Pattern Matching and Rewriting (Phase H)

## Overview

The `pattern` crate provides **Phase H: Pattern Matching v2** capabilities including:
- **AC-aware pattern matching** for Add/Mul (order-insensitive)
- **Rule registry** with guards and cost-based selection
- **Rewrite scheduler** with termination caps and metrics
- **Basic substitution** utilities for simple cases

This implements the roadmap's "AC-aware pattern matching for Add/Mul; constraint predicates; rule registry DSL; rewrite scheduler: cost model; termination caps; metrics."

---

## Part 1: AC-Aware Pattern Matching

### Pattern Types

The `ac` module provides structural pattern matching with wildcards:

```rust
pub enum Pat {
    Any(String),                      // Wildcard: matches any subexpr
    Symbol(String),                   // Literal symbol
    Integer(i64),                     // Literal integer
    Rational(i64, i64),              // Literal rational
    Function(String, Vec<Pat>),      // Function with args
    Add(Vec<Pat>),                   // Addition (AC-aware)
    Mul(Vec<Pat>),                   // Multiplication (AC-aware)
    Pow(Box<Pat>, Box<Pat>),        // Power: base^exp
}
```

### AC (Associative-Commutative) Matching

For `Add` and `Mul` nodes, matching is **order-insensitive**:

```rust
use pattern::ac::{match_expr, Pat};

let mut st = Store::new();
let x = st.sym("x");
let y = st.sym("y");

// Expression: x + y
let expr = st.add(vec![x, y]);

// Pattern: y + x (different order)
let pattern = Pat::Add(vec![
    Pat::Symbol("y".into()),
    Pat::Symbol("x".into()),
]);

// Match succeeds despite different order!
let bindings = match_expr(&st, &pattern, expr);
assert!(bindings.is_some());
```

### Wildcard Matching

Use `Pat::Any(name)` to capture subexpressions:

```rust
// Pattern: x + ?a  (where ?a is a wildcard)
let pattern = Pat::Add(vec![
    Pat::Symbol("x".into()),
    Pat::Any("a".into()),
]);

// Expression: x + 5
let five = st.int(5);
let expr = st.add(vec![x, five]);

let bindings = match_expr(&st, &pattern, expr).unwrap();
assert_eq!(bindings["a"], five);
```

### Consistency Check

If a wildcard appears multiple times, all occurrences must bind to the **same** expression:

```rust
// Pattern: ?a + ?a  (same variable twice)
let pattern = Pat::Add(vec![
    Pat::Any("a".into()),
    Pat::Any("a".into()),
]);

// x + x matches
let x_plus_x = st.add(vec![x, x]);
assert!(match_expr(&st, &pattern, x_plus_x).is_some());

// x + y does NOT match
let x_plus_y = st.add(vec![x, y]);
assert!(match_expr(&st, &pattern, x_plus_y).is_none());
```

---

## Part 2: Rule Registry

### Defining Rules

Rules consist of a pattern, optional guard, and build function:

```rust
use pattern::registry::{Rule, GuardFn, BuildFn};
use pattern::ac::Pat;

let rule = Rule {
    name: "x + 0 = x",
    pattern: Pat::Add(vec![
        Pat::Any("x".into()),
        Pat::Integer(0),
    ]),
    guard: None,  // Optional guard function
    build: |store, bindings| bindings["x"],
};
```

### Guards

Guards are predicates that must pass for a rule to apply:

```rust
fn positive_only(store: &Store, bindings: &Bindings) -> bool {
    let x = bindings["x"];
    // Check if x is a positive integer
    match (&store.get(x).op, &store.get(x).payload) {
        (Op::Integer, Payload::Int(k)) => *k > 0,
        _ => false,
    }
}

let rule_with_guard = Rule {
    name: "positive identity",
    pattern: Pat::Any("x".into()),
    guard: Some(positive_only),
    build: |store, bindings| bindings["x"],
};
```

### Cost-Based Selection

`apply_best_rule_by_node_count` chooses the rule that minimizes expression size:

```rust
use pattern::registry::apply_best_rule_by_node_count;

let rules = vec![rule1, rule2, rule3];
let simplified = apply_best_rule_by_node_count(&mut store, expr, &rules);
```

The registry counts nodes (DAG size) and picks the smallest result.

---

## Part 3: Rewrite Scheduler

### Fixpoint Iteration

The scheduler applies rewrite rules until a fixpoint or step limit:

```rust
use pattern::scheduler::rewrite_fixpoint;

let (result, stats) = rewrite_fixpoint(&mut store, expr, max_steps);

println!("Steps: {}", stats.steps);
println!("Changed: {}", stats.changed);
println!("Nodes before: {}", stats.nodes_before);
println!("Nodes after: {}", stats.nodes_after);
```

### Termination Guarantees

- **Step cap**: Prevents infinite loops
- **Fixpoint detection**: Stops when `rewrite(e) == e`
- **Metrics tracking**: `RewriteStats` reports progress

### Example Workflow

```rust
let mut st = Store::new();
let zero = st.int(0);

// exp(0) + sin(0) + cos(0)
let exp0 = st.func("exp", vec![zero]);
let sin0 = st.func("sin", vec![zero]);
let cos0 = st.func("cos", vec![zero]);
let expr = st.add(vec![exp0, sin0, cos0]);

// Rewrite to fixpoint (max 10 steps)
let (result, stats) = rewrite_fixpoint(&mut st, expr, 10);

// Result: 2  (because exp(0)=1, sin(0)=0, cos(0)=1)
assert_eq!(result, st.int(2));
assert!(stats.changed);
```

---

## Part 4: Basic Substitution

### Core Function

```rust
pub fn subst_symbol(store: &mut Store, id: ExprId, sym: &str, with_expr: ExprId) -> ExprId
```

Replaces all occurrences of symbol `sym` with `with_expr` in the expression `id`.

## Behavior

### Symbol Replacement
```rust
let mut st = Store::new();
let x = st.sym("x");
let y = st.sym("y");

// Replace x with y in expression
let expr = st.add(vec![x, st.int(1)]);  // x + 1
let result = subst_symbol(&mut st, expr, "x", y);
// Result: y + 1
```

### Non-recursive Substitution
The replacement expression is **not** recursively searched:
```rust
// If with_expr contains the symbol, it won't be replaced again
let x_plus_2 = st.add(vec![x, st.int(2)]);
let result = subst_symbol(&mut st, x, "x", x_plus_2);
// Result: x + 2 (x inside with_expr is NOT replaced)
```

This prevents infinite recursion and allows safe simultaneous substitutions.

### Unchanged Symbols
Symbols that don't match are left unchanged:
```rust
let expr = st.mul(vec![st.int(2), x]);  // 2*x
let result = subst_symbol(&mut st, expr, "y", st.int(5));
// Result: 2*x (no change, since expr doesn't contain y)
```

## Operation-Specific Behavior

### Constants (Integer, Rational)
```rust
subst_symbol(store, constant, sym, with_expr) == constant
```

Constants are never affected by substitution.

### Symbols
```rust
if symbol_name == sym {
    return with_expr;
} else {
    return symbol;
}
```

Matching symbols are replaced; others are preserved.

### Addition
```rust
subst_symbol(store, Add(a, b, c), sym, with_expr) 
  == Add(subst(a), subst(b), subst(c))
```

Recursively substitutes in all children and rebuilds using canonical `store.add()`.

### Multiplication
Same recursive strategy as addition:
```rust
subst_symbol(store, Mul(a, b), sym, with_expr)
  == Mul(subst(a), subst(b))
```

### Power
Substitutes in both base and exponent:
```rust
subst_symbol(store, Pow(base, exp), sym, with_expr)
  == Pow(subst(base), subst(exp))
```

### Functions
Substitutes in all arguments:
```rust
subst_symbol(store, Func(name, [a, b]), sym, with_expr)
  == Func(name, [subst(a), subst(b)])
```

Function names are never substituted (they're not symbols).

## Examples

### Simple Substitution
```rust
use expr_core::Store;
use pattern::subst_symbol;

let mut st = Store::new();
let x = st.sym("x");

// Build: x^2 + 3x + 1
let expr = st.add(vec![
    st.pow(x, st.int(2)),
    st.mul(vec![st.int(3), x]),
    st.int(1),
]);

// Substitute x -> 5
let result = subst_symbol(&mut st, expr, "x", st.int(5));
// Result: 5^2 + 3*5 + 1 = 41 (after simplification)
```

### Expression Substitution
```rust
let x = st.sym("x");
let y = st.sym("y");

// Build: x + 1
let expr = st.add(vec![x, st.int(1)]);

// Substitute x -> (y + 2)
let y_plus_2 = st.add(vec![y, st.int(2)]);
let result = subst_symbol(&mut st, expr, "x", y_plus_2);
// Result: (y + 2) + 1 = y + 3 (after simplification)
```

### Nested Substitution
```rust
let x = st.sym("x");

// Build: (x + 1)^2
let xp1 = st.add(vec![x, st.int(1)]);
let expr = st.pow(xp1, st.int(2));

// Substitute x -> (y + 2)
let y = st.sym("y");
let y_plus_2 = st.add(vec![y, st.int(2)]);
let result = subst_symbol(&mut st, expr, "x", y_plus_2);
// Result: ((y+2)+1)^2 = (y+3)^2
```

### Function Argument Substitution
```rust
let x = st.sym("x");
let sin_x = st.func("sin", vec![x]);

// Substitute x -> 2x
let two_x = st.mul(vec![st.int(2), x]);
let result = subst_symbol(&mut st, sin_x, "x", two_x);
// Result: sin(2x)
```

### Power with Substitution in Exponent
```rust
let x = st.sym("x");
let n = st.sym("n");
let x_to_n = st.pow(x, n);

// Substitute n -> 3
let result = subst_symbol(&mut st, x_to_n, "n", st.int(3));
// Result: x^3
```

## Integration with Simplify

Substitution does NOT automatically simplify. For canonical results:
```rust
let result = subst_symbol(&mut st, expr, "x", with_expr);
let simplified = simplify::simplify(&mut st, result);
```

**Example:**
```rust
let expr = st.add(vec![x, x]);  // x + x
let result = subst_symbol(&mut st, expr, "x", st.int(2));
// Result: 2 + 2 (not simplified)

let simplified = simplify::simplify(&mut st, result);
// Result: 4
```

## Use Cases

### Evaluating Expressions
```rust
// Define f(x) = x^2 + 1
let x = st.sym("x");
let f = st.add(vec![st.pow(x, st.int(2)), st.int(1)]);

// Evaluate f(3) = 3^2 + 1
let result = subst_symbol(&mut st, f, "x", st.int(3));
let value = simplify::simplify(&mut st, result);
// value = 10
```

### Composing Functions
```rust
// f(x) = x^2
// g(x) = x + 1
// Compute f(g(x)) = (x+1)^2
let f = st.pow(x, st.int(2));
let g = st.add(vec![x, st.int(1)]);
let composition = subst_symbol(&mut st, f, "x", g);
// Result: (x+1)^2
```

### Change of Variables
```rust
// u = x^2, substitute in expression x^4 + 2x^2 + 1
let x = st.sym("x");
let u = st.sym("u");
let x2 = st.pow(x, st.int(2));

let expr = st.add(vec![
    st.pow(x, st.int(4)),
    st.mul(vec![st.int(2), x2]),
    st.int(1),
]);

// First substitute x^2 -> u (conceptually)
// For actual implementation, would need pattern matching on x^2
// Current API only handles symbol-to-expr substitution
```

### Parameterized Expressions
```rust
// Template: ax^2 + bx + c
let a = st.sym("a");
let b = st.sym("b");
let c = st.sym("c");
let x = st.sym("x");

let template = st.add(vec![
    st.mul(vec![a, st.pow(x, st.int(2))]),
    st.mul(vec![b, x]),
    c,
]);

// Instantiate with specific values
let instance = subst_symbol(&mut st, template, "a", st.int(2));
let instance = subst_symbol(&mut st, instance, "b", st.int(3));
let instance = subst_symbol(&mut st, instance, "c", st.int(1));
// Result: 2x^2 + 3x + 1
```

## Performance

- **Time complexity**: O(n) where n is the size of the expression tree
- **Space complexity**: O(n) for building the new tree
- **Hash-consing**: Result benefits from structural sharing in Store

## Limitations

### No Pattern Matching
Only handles exact symbol replacement, not structural patterns:
```rust
// Cannot match patterns like "x^2" and replace with "u"
// Can only replace symbol "x" with an expression
```

### No Simultaneous Substitution
Multiple substitutions must be sequential:
```rust
// To swap x and y:
let temp = st.sym("temp");
let step1 = subst_symbol(&mut st, expr, "x", temp);
let step2 = subst_symbol(&mut st, step1, "y", x);
let step3 = subst_symbol(&mut st, step2, "temp", y);
```

### No Conditional Substitution
Cannot apply different substitutions based on context:
```rust
// Cannot say "replace x with y only in sin(x) but not in x^2"
```

## Testing

Comprehensive tests cover:
- Basic symbol substitution
- Substitution in Add, Mul, Pow
- Function argument substitution
- No-op when symbol is absent
- Constants remain unchanged
- Nested expression substitution

Run tests:
```bash
cargo test -p pattern
```

## Phase H: Implementation Status

✅ **COMPLETE** - All Phase H deliverables implemented:

| Feature | Status | Module |
|---------|--------|--------|
| AC-aware pattern matching | ✅ Complete | `ac.rs` |
| Constraint predicates (guards) | ✅ Complete | `registry.rs` |
| Rule registry DSL | ✅ Complete | `registry.rs` |
| Cost model (node count) | ✅ Complete | `registry.rs`, `scheduler.rs` |
| Termination caps | ✅ Complete | `scheduler.rs` |
| Metrics (RewriteStats) | ✅ Complete | `scheduler.rs` |
| Rewrite scheduler | ✅ Complete | `scheduler.rs` |
| Bottom-up rewriting | ✅ Complete | `rewrite.rs` |

**Acceptance Criteria Met:**
- ✅ Rules for common identities apply deterministically
- ✅ Measured rewrite step bounds respected
- ✅ AC-aware matching for Add/Mul operations
- ✅ Termination guaranteed by step caps and fixpoint detection

## Future Enhancements (Phase I+)

Phase H is complete. Future work (beyond current scope):

- **Simultaneous substitution**: Replace multiple symbols atomically
- **Conditional substitution**: Context-dependent rule application
- **Pattern compilation**: Pre-compile patterns for performance
- **More sophisticated cost models**: Beyond node count
- **Domain-specific rewrite strategies**: Algebra, trigonometry, etc.

## Example: Polynomial Evaluation

```rust
use expr_core::Store;
use pattern::subst_symbol;
use simplify::simplify;

let mut st = Store::new();
let x = st.sym("x");

// Define polynomial: p(x) = x^3 - 2x^2 + x - 1
let poly = st.add(vec![
    st.pow(x, st.int(3)),
    st.mul(vec![st.int(-2), st.pow(x, st.int(2))]),
    x,
    st.int(-1),
]);

// Evaluate at x = 2
let evaluated = subst_symbol(&mut st, poly, "x", st.int(2));
let result = simplify(&mut st, evaluated);
println!("p(2) = {}", st.to_string(result));
// Output: p(2) = 1
```

## References

- Depends on: `expr_core`
- Used by: Integration tests, user code
- Related: Pattern matching in Mathematica, SymPy's `subs()`
