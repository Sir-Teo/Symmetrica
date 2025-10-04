# expr_core - Expression Core Module

## Overview

The `expr_core` crate is the foundational kernel of Symmetrica, providing an immutable DAG (Directed Acyclic Graph) representation of symbolic expressions with hash-consing for structural sharing and canonical constructors.

## Core Principles

- **Immutable DAG**: All expressions are immutable nodes stored in an arena allocator
- **Hash-consing**: Identical expressions share the same node (structural sharing)
- **Canonical Forms**: Operations like `Add`, `Mul`, and `Pow` automatically normalize and order their children
- **Deterministic Digests**: Each node has a 64-bit FNV-1a digest for stable ordering and comparison

## Key Types

### ExprId
```rust
pub struct ExprId(pub usize);
```
An opaque handle to an expression node in the store. Copy-able and lightweight.

### Op
```rust
pub enum Op {
    Add,      // n-ary addition
    Mul,      // n-ary multiplication
    Pow,      // binary power (base^exponent)
    Symbol,   // named variable
    Integer,  // i64 integer constant
    Rational, // reduced rational (numerator, denominator)
    Function, // named function application
}
```

### Payload
```rust
pub enum Payload {
    None,
    Sym(String),        // Symbol name
    Int(i64),           // Integer value
    Rat(i64, i64),      // Normalized rational (num, den>0, gcd=1)
    Func(String),       // Function name
}
```

### Node
```rust
pub struct Node {
    pub op: Op,
    pub payload: Payload,
    pub children: Vec<ExprId>,
    pub digest: u64,  // Structural fingerprint
}
```

### Store
```rust
pub struct Store {
    nodes: Vec<Node>,                      // Arena of all nodes
    interner: HashMap<NodeKey, ExprId>,    // Hash-consing table
}
```

The central expression store managing all nodes with automatic deduplication.

## API

### Construction

#### Atomic Values
```rust
pub fn sym<S: Into<String>>(&mut self, name: S) -> ExprId
pub fn int(&mut self, n: i64) -> ExprId
pub fn rat(&mut self, num: i64, den: i64) -> ExprId
pub fn func<S: Into<String>>(&mut self, name: S, args: Vec<ExprId>) -> ExprId
```

**Examples:**
```rust
let mut st = Store::new();
let x = st.sym("x");           // Symbol x
let five = st.int(5);          // Integer 5
let half = st.rat(1, 2);       // Rational 1/2
let sin_x = st.func("sin", vec![x]);  // sin(x)
```

#### Composite Operations
```rust
pub fn add<I: IntoIterator<Item = ExprId>>(&mut self, it: I) -> ExprId
pub fn mul<I: IntoIterator<Item = ExprId>>(&mut self, it: I) -> ExprId
pub fn pow(&mut self, base: ExprId, exp: ExprId) -> ExprId
```

**Canonical Properties:**
- **Add**: Flattens nested additions, folds numeric constants, removes zeros, sorts by digest
- **Mul**: Flattens nested products, folds numeric constants, removes ones, short-circuits on zero, sorts by digest
- **Pow**: Simplifies `x^1 → x` and `x^0 → 1` (except `0^0`)

**Examples:**
```rust
let mut st = Store::new();
let x = st.sym("x");
let y = st.sym("y");

// Addition: x + y (canonically ordered)
let sum = st.add(vec![x, y]);

// Multiplication: 2*x (coefficient extracted and combined)
let two = st.int(2);
let prod = st.mul(vec![two, x]);

// Power: x^3
let three = st.int(3);
let pow = st.pow(x, three);

// Complex: (x + 2y)^2
let two_y = st.mul(vec![two, y]);
let expr = st.add(vec![x, two_y]);
let result = st.pow(expr, two);
```

### Node Access
```rust
pub fn get(&self, id: ExprId) -> &Node
```

Returns a reference to the node for the given ID.

### Pretty Printing
```rust
pub fn to_string(&self, id: ExprId) -> String
```

Produces a precedence-aware text representation:
- Parenthesizes lower-precedence operations when nested
- Rationals as `num/den`
- Functions as `name(args)`

**Examples:**
```rust
st.to_string(st.add(vec![x, two]));        // "2 + x" (canonically sorted)
st.to_string(st.mul(vec![x, sum]));        // "x * (2 + y)"
st.to_string(st.pow(sum, three));          // "(2 + y)^3"
```

## Canonical Normalization

### Addition Rules
1. Flatten: `(a + b) + c → a + b + c`
2. Fold constants: `1 + 2 + x → 3 + x`
3. Remove zeros: `x + 0 → x`
4. Sort by digest for determinism
5. Empty sum → `0`
6. Single term → unwrap

### Multiplication Rules
1. Flatten: `(a * b) * c → a * b * c`
2. Fold constants: `2 * 3 * x → 6 * x`
3. Zero propagation: `x * 0 → 0`
4. Remove ones: `x * 1 → x`
5. Sort by digest
6. Empty product → `1`
7. Single factor → unwrap

### Power Rules
1. `x^1 → x`
2. `x^0 → 1` (except `0^0` stays as-is)
3. No further simplification (see `simplify` crate for advanced rules)

## Deterministic Digests

Each node has a 64-bit FNV-1a digest computed from:
- Operation type tag
- Payload data (symbols, integers, rationals)
- Child digests (recursively)

Digests enable:
- Deterministic ordering in canonical Add/Mul
- Fast structural comparison
- Stable output across runs

## Hash-Consing (Structural Sharing)

The interner ensures that constructing identical expressions returns the same `ExprId`:

```rust
let x1 = st.sym("x");
let x2 = st.sym("x");
assert_eq!(x1, x2);  // Same ID

let sum1 = st.add(vec![x, y]);
let sum2 = st.add(vec![y, x]);  // Order doesn't matter
assert_eq!(sum1, sum2);  // Canonicalized to same form
```

Benefits:
- Memory efficiency through sharing
- O(1) structural equality checks
- Stable subexpression identification

## Rational Arithmetic

Small rationals use `i64` numerator and denominator:
- Always normalized: `den > 0`, `gcd(|num|, den) = 1`
- Zero denominator panics
- Rationals with `den=1` are stored as integers

## Design Constraints

- **No simplification**: Core constructors perform minimal normalization. Advanced simplification lives in the `simplify` crate.
- **Deterministic**: All operations produce identical results across platforms and runs.
- **Minimal kernel**: Small dependency footprint, only depends on `arith` for rational helpers.

## Performance Characteristics

- **Construction**: O(log n) amortized per node (hash table lookup)
- **Access**: O(1) via ExprId
- **Memory**: Linear in unique subexpressions (sharing eliminates duplicates)
- **Digest computation**: O(children) per node

## Thread Safety

`Store` is not thread-safe. Use separate stores per thread or external synchronization.

## Testing

The crate has comprehensive unit tests covering:
- Hash-consing invariants
- Canonical ordering
- Rational normalization
- Power simplification
- Pretty printing
- Edge cases (empty operations, zero exponents, etc.)

Run tests:
```bash
cargo test -p expr_core
```

## Integration

Most users will not use `expr_core` directly but through higher-level APIs:
- **simplify**: Advanced simplification passes
- **calculus**: Differentiation and integration
- **solver**: Equation solving
- **pattern**: Substitution and pattern matching

## Example: Building a Polynomial

```rust
use expr_core::Store;

let mut st = Store::new();
let x = st.sym("x");

// Build x^2 + 3x + 2
let two_int = st.int(2);
let x_squared = st.pow(x, two_int);

let three = st.int(3);
let three_x = st.mul(vec![three, x]);

let constant = st.int(2);

let poly = st.add(vec![x_squared, three_x, constant]);

println!("{}", st.to_string(poly));  // "2 + 3 * x + x^2"
```

## References

- **FNV-1a hash**: http://www.isthe.com/chongo/tech/comp/fnv/
- **Hash-consing**: Classic technique in symbolic computation and compilers
- **DAG representation**: Standard in computer algebra systems (Maxima, SymPy, etc.)
