Awesome—here’s a repo skeleton + starter code you can paste in and build right away.
It’s deliberately lightweight (no external deps by default), modular, and matches the roadmap: an immutable DAG expression core with hash‑consing, canonical Add/Mul/Pow, a minimal simplifier (collect‑like‑terms, numeric folding), and a tiny CLI demo. Everything else is scaffolded so you can grow cleanly.

⸻

Repository layout

matika/                         # your working name (rename anytime)
├─ Cargo.toml                   # workspace
├─ README.md
└─ crates/
   ├─ expr_core/                # core expression kernel (immutable DAG + interner)
   │  ├─ Cargo.toml
   │  └─ src/lib.rs
   ├─ simplify/                 # explicit simplification passes (collect terms, etc.)
   │  ├─ Cargo.toml
   │  └─ src/lib.rs
   ├─ cli/                      # tiny REPL/demo
   │  ├─ Cargo.toml
   │  └─ src/main.rs
   ├─ polys/                    # (stub) polynomial module
   │  ├─ Cargo.toml
   │  └─ src/lib.rs
   ├─ calculus/                 # (stub) diff/limits/series/integration orchestrators
   │  ├─ Cargo.toml
   │  └─ src/lib.rs
   ├─ solver/                   # (stub) linear/univariate solve orchestrators
   │  ├─ Cargo.toml
   │  └─ src/lib.rs
   ├─ matrix/                   # (stub) symbolic matrices
   │  ├─ Cargo.toml
   │  └─ src/lib.rs
   ├─ assumptions/              # (stub) property lattice & context
   │  ├─ Cargo.toml
   │  └─ src/lib.rs
   ├─ api/                      # (stub) FFI & Python bindings later
   │  ├─ Cargo.toml
   │  └─ src/lib.rs
   └─ io/                       # (stub) parser/printers beyond basics
      ├─ Cargo.toml
      └─ src/lib.rs


⸻

Workspace manifest

Cargo.toml (root)

[workspace]
resolver = "2"
members = [
  "crates/expr_core",
  "crates/simplify",
  "crates/cli",
  "crates/polys",
  "crates/calculus",
  "crates/solver",
  "crates/matrix",
  "crates/assumptions",
  "crates/api",
  "crates/io",
]

[workspace.package]
edition = "2021"


⸻

Core engine (immutable DAG + hash‑consing + canonical Add/Mul/Pow)

crates/expr_core/Cargo.toml

[package]
name = "expr_core"
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"

[dependencies]

crates/expr_core/src/lib.rs

//! expr_core: minimal immutable DAG expression kernel with hash-consing.
//! - Op/Node/ExprId
//! - Store with interning + canonical Add/Mul/Pow
//! - Basic numeric payload (i64 integers; small rationals)
//! - Deterministic digest (FNV-1a 64) for canonical ordering

use std::collections::HashMap;

// ---------- IDs & basic nodes ----------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ExprId(pub usize);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Op {
    Add,
    Mul,
    Pow,
    Symbol,
    Integer,
    Rational,
    Function,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Payload {
    None,
    Sym(String),
    Int(i64),
    // Reduced fraction: den>0 and gcd(|num|, den)=1
    Rat(i64, i64),
    Func(String),
}

#[derive(Clone, Debug)]
pub struct Node {
    pub op: Op,
    pub payload: Payload,
    pub children: Vec<ExprId>,
    pub digest: u64, // structural fingerprint for ordering
}

// Key used for interning (avoid storing unstable child ids in key; use child digests)
#[derive(Hash, PartialEq, Eq)]
struct NodeKey {
    op: Op,
    payload: Payload,
    child_digests: Vec<u64>,
}

// ---------- Store (arena + interner) ----------

pub struct Store {
    nodes: Vec<Node>,
    interner: HashMap<NodeKey, ExprId>,
}

impl Default for Store {
    fn default() -> Self {
        Self { nodes: Vec::new(), interner: HashMap::new() }
    }
}

impl Store {
    pub fn new() -> Self { Self::default() }

    pub fn get(&self, id: ExprId) -> &Node { &self.nodes[id.0] }

    // ---- Constructors (atoms) ----
    pub fn sym<S: Into<String>>(&mut self, name: S) -> ExprId {
        self.intern(Op::Symbol, Payload::Sym(name.into()), vec![])
    }
    pub fn int(&mut self, n: i64) -> ExprId {
        self.intern(Op::Integer, Payload::Int(n), vec![])
    }
    pub fn rat(&mut self, num: i64, den: i64) -> ExprId {
        let (n, d) = normalize_rat(num, den);
        if d == 1 { return self.int(n); }
        self.intern(Op::Rational, Payload::Rat(n, d), vec![])
    }
    pub fn func<S: Into<String>>(&mut self, name: S, args: Vec<ExprId>) -> ExprId {
        // Functions are not canonicalized across args (order matters).
        self.intern(Op::Function, Payload::Func(name.into()), args)
    }

    // ---- Canonical combinators ----
    pub fn add<I: IntoIterator<Item = ExprId>>(&mut self, it: I) -> ExprId {
        let mut terms: Vec<ExprId> = Vec::new();
        let mut num = (0i64, 1i64); // rational accumulator (num, den)

        // Flatten and fold numeric terms
        for t in it {
            match self.get(t).op {
                Op::Add => {
                    for c in &self.get(t).children { terms.push(*c); }
                }
                Op::Integer => {
                    if let Payload::Int(k) = &self.get(t).payload {
                        num = rat_add(num, (*k, 1));
                    }
                }
                Op::Rational => {
                    if let Payload::Rat(n, d) = &self.get(t).payload {
                        num = rat_add(num, (*n, *d));
                    }
                }
                _ => terms.push(t),
            }
        }

        // Push folded numeric if nonzero
        if num.0 != 0 {
            terms.push(self.rat(num.0, num.1));
        }

        // Remove trivial zeros
        terms.retain(|&id| !matches!((self.get(id).op, &self.get(id).payload), (Op::Integer, Payload::Int(0))));

        if terms.is_empty() {
            return self.int(0);
        }
        if terms.len() == 1 {
            return terms[0];
        }

        // Deterministic order by digest to achieve canonical form
        terms.sort_by_key(|id| self.get(*id).digest);

        self.intern(Op::Add, Payload::None, terms)
    }

    pub fn mul<I: IntoIterator<Item = ExprId>>(&mut self, it: I) -> ExprId {
        let mut factors: Vec<ExprId> = Vec::new();
        let mut num = (1i64, 1i64); // rational product

        for f in it {
            match self.get(f).op {
                Op::Mul => {
                    for c in &self.get(f).children { factors.push(*c); }
                }
                Op::Integer => {
                    if let Payload::Int(k) = &self.get(f).payload {
                        if *k == 0 { return self.int(0); }
                        num = rat_mul(num, (*k, 1));
                    }
                }
                Op::Rational => {
                    if let Payload::Rat(n, d) = &self.get(f).payload {
                        if *n == 0 { return self.int(0); }
                        num = rat_mul(num, (*n, *d));
                    }
                }
                _ => factors.push(f),
            }
        }

        // If numeric product != 1, include it
        if !(num.0 == 1 && num.1 == 1) {
            factors.push(self.rat(num.0, num.1));
        }

        // Remove multiplicative identity 1
        factors.retain(|&id| !matches!((self.get(id).op, &self.get(id).payload), (Op::Integer, Payload::Int(1))));

        if factors.is_empty() {
            return self.int(1);
        }
        if factors.len() == 1 {
            return factors[0];
        }

        // Deterministic order (by digest)
        factors.sort_by_key(|id| self.get(*id).digest);

        self.intern(Op::Mul, Payload::None, factors)
    }

    pub fn pow(&mut self, base: ExprId, exp: ExprId) -> ExprId {
        // Basic safe simplifications
        if let (Op::Integer, Payload::Int(1)) = (&self.get(exp).op, &self.get(exp).payload) {
            return base;
        }
        if let (Op::Integer, Payload::Int(0)) = (&self.get(exp).op, &self.get(exp).payload) {
            // 0^0 left as-is (non-simplifying) to avoid domain issues
            if matches!((&self.get(base).op, &self.get(base).payload), (Op::Integer, Payload::Int(0))) {
                return self.intern(Op::Pow, Payload::None, vec![base, exp]);
            }
            return self.int(1);
        }
        self.intern(Op::Pow, Payload::None, vec![base, exp])
    }

    // ---- Printing (very small, precedence-aware) ----
    pub fn to_string(&self, id: ExprId) -> String {
        fn prec(op: &Op) -> u8 {
            match op { Op::Add => 1, Op::Mul => 2, Op::Pow => 3, _ => 4 }
        }
        fn go(st: &Store, id: ExprId, parent_prec: u8) -> String {
            let n = st.get(id);
            let s = match (&n.op, &n.payload) {
                (Op::Integer, Payload::Int(k)) => k.to_string(),
                (Op::Rational, Payload::Rat(a,b)) => format!("{}/{}", a, b),
                (Op::Symbol, Payload::Sym(name)) => name.clone(),
                (Op::Function, Payload::Func(name)) => {
                    let args = n.children.iter().map(|c| go(st, *c, 0)).collect::<Vec<_>>().join(", ");
                    format!("{name}({args})")
                }
                (Op::Add, _) => {
                    n.children.iter().map(|c| go(st, *c, prec(&Op::Add))).collect::<Vec<_>>().join(" + ")
                }
                (Op::Mul, _) => {
                    n.children.iter().map(|c| go(st, *c, prec(&Op::Mul))).collect::<Vec<_>>().join(" * ")
                }
                (Op::Pow, _) => {
                    let b = go(st, n.children[0], prec(&Op::Pow));
                    let e = go(st, n.children[1], prec(&Op::Pow));
                    format!("{b}^{e}")
                }
                _ => "<unknown>".into(),
            };
            if prec(&n.op) < parent_prec { format!("({s})") } else { s }
        }
        go(self, id, 0)
    }

    // ---- Interning ----
    fn intern(&mut self, op: Op, payload: Payload, mut children: Vec<ExprId>) -> ExprId {
        // Compute child digests; some ops (Add/Mul) expect children sorted already
        let child_digests: Vec<u64> = children.iter().map(|id| self.get(*id).digest).collect();
        let key = NodeKey { op: op.clone(), payload: payload.clone(), child_digests };

        if let Some(&id) = self.interner.get(&key) {
            return id;
        }

        // Compute digest for this node deterministically
        let digest = digest_node(&op, &payload, &children.iter().map(|id| self.get(*id).digest).collect::<Vec<_>>());

        let id = ExprId(self.nodes.len());
        self.nodes.push(Node { op, payload, children, digest });
        self.interner.insert(key, id);
        id
    }
}

// ---------- Deterministic digest (FNV-1a 64) ----------

fn digest_node(op: &Op, payload: &Payload, child_digests: &[u64]) -> u64 {
    let mut h = Fnv64::new();
    h.write_u8(op_tag(op));
    match payload {
        Payload::None => h.write_u8(0),
        Payload::Int(k) => { h.write_u8(1); h.write_i64(*k); }
        Payload::Rat(n,d) => { h.write_u8(2); h.write_i64(*n); h.write_i64(*d); }
        Payload::Sym(s) => { h.write_u8(3); h.write_bytes(s.as_bytes()); }
        Payload::Func(s) => { h.write_u8(4); h.write_bytes(s.as_bytes()); }
    }
    for &cd in child_digests { h.write_u64(cd); }
    h.finish()
}

fn op_tag(op: &Op) -> u8 {
    match op {
        Op::Add => 1, Op::Mul => 2, Op::Pow => 3, Op::Symbol => 4,
        Op::Integer => 5, Op::Rational => 6, Op::Function => 7,
    }
}

// Minimal FNV-1a 64 hasher (deterministic)
struct Fnv64(u64);
impl Fnv64 {
    fn new() -> Self { Self(0xcbf29ce484222325) }
    fn write_u8(&mut self, x: u8) { self.0 ^= x as u64; self.0 = self.0.wrapping_mul(0x100000001b3); }
    fn write_i64(&mut self, x: i64) { self.write_u64(x as u64); }
    fn write_u64(&mut self, x: u64) {
        for b in x.to_le_bytes() { self.write_u8(b); }
    }
    fn write_bytes(&mut self, bs: &[u8]) { for &b in bs { self.write_u8(b) } }
    fn finish(&self) -> u64 { self.0 }
}

// ---------- Small rational helpers (i64) ----------

fn gcd_i64(mut a: i64, mut b: i64) -> i64 {
    if a == 0 { return b.abs() }
    if b == 0 { return a.abs() }
    while b != 0 {
        let t = a % b;
        a = b; b = t;
    }
    a.abs()
}
fn normalize_rat(num: i64, den: i64) -> (i64, i64) {
    assert!(den != 0, "zero denominator");
    let mut n = num; let mut d = den;
    if d < 0 { n = -n; d = -d; }
    if n == 0 { return (0, 1); }
    let g = gcd_i64(n.abs(), d);
    (n / g, d / g)
}
fn rat_add(a: (i64,i64), b: (i64,i64)) -> (i64,i64) {
    normalize_rat(a.0 * b.1 + b.0 * a.1, a.1 * b.1)
}
fn rat_mul(a: (i64,i64), b: (i64,i64)) -> (i64,i64) {
    normalize_rat(a.0 * b.0, a.1 * b.1)
}


⸻

Simplifier (collect like terms, numeric folding)

crates/simplify/Cargo.toml

[package]
name = "simplify"
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"

[dependencies]
expr_core = { path = "../expr_core" }

crates/simplify/src/lib.rs

//! simplify: explicit passes on top of expr_core canonical constructors.
//! v0: recursive simplify; collect-like-terms for Add; basic Pow/Mul cleanups.

use expr_core::{ExprId, Op, Payload, Store};

pub fn simplify(store: &mut Store, id: ExprId) -> ExprId {
    match store.get(id).op {
        Op::Add => simplify_add(store, id),
        Op::Mul => simplify_mul(store, id),
        Op::Pow => {
            let b = simplify(store, store.get(id).children[0]);
            let e = simplify(store, store.get(id).children[1]);
            store.pow(b, e)
        }
        Op::Function => {
            let name = match &store.get(id).payload { Payload::Func(s) => s.clone(), _ => "<f>".into() };
            let args = store.get(id).children.iter().map(|c| simplify(store, *c)).collect::<Vec<_>>();
            store.func(name, args)
        }
        _ => id,
    }
}

fn simplify_add(store: &mut Store, id: ExprId) -> ExprId {
    // First simplify children
    let mut terms = Vec::new();
    for &c in &store.get(id).children {
        terms.push(simplify(store, c));
    }
    // Split each term into (coeff, base), then collect coefficients per base
    use std::collections::HashMap;
    let mut map: HashMap<ExprId, (i64, i64)> = HashMap::new(); // base -> rational coeff (num, den)
    for t in terms {
        let (coeff, base) = split_coeff(store, t);
        let entry = map.entry(base).or_insert((0, 1));
        *entry = rat_add(*entry, coeff);
    }

    // Rebuild sum; numeric-only terms are under base==1
    let mut new_terms: Vec<ExprId> = Vec::new();
    for (base, (n, d)) in map {
        if n == 0 { continue; }
        let term = if is_one(store, base) {
            store.rat(n, d)
        } else if n == 1 && d == 1 {
            base
        } else {
            store.mul(vec![store.rat(n, d), base])
        };
        new_terms.push(term);
    }
    if new_terms.is_empty() { return store.int(0); }
    store.add(new_terms)
}

fn simplify_mul(store: &mut Store, id: ExprId) -> ExprId {
    let mut factors = Vec::new();
    for &c in &store.get(id).children {
        factors.push(simplify(store, c));
    }
    // TODO: optional: merge powers with same base (x^a * x^b -> x^(a+b))
    store.mul(factors)
}

/// Split term into (coeff rational, base expr) where term == coeff * base
fn split_coeff(store: &mut Store, id: ExprId) -> ((i64, i64), ExprId) {
    match (&store.get(id).op, &store.get(id).payload) {
        (Op::Integer, Payload::Int(k)) => (((*k), 1), store.int(1)),
        (Op::Rational, Payload::Rat(n,d)) => (((*n), (*d)), store.int(1)),
        (Op::Mul, _) => {
            let mut coeff = (1i64, 1i64);
            let mut rest: Vec<ExprId> = Vec::new();
            for &f in &store.get(id).children {
                match (&store.get(f).op, &store.get(f).payload) {
                    (Op::Integer, Payload::Int(k)) => { coeff = rat_mul(coeff, (*k, 1)); }
                    (Op::Rational, Payload::Rat(n,d)) => { coeff = rat_mul(coeff, (*n, *d)); }
                    _ => rest.push(f),
                }
            }
            let base = if rest.is_empty() { store.int(1) } else { store.mul(rest) };
            (coeff, base)
        }
        _ => ((1, 1), id),
    }
}

fn is_one(store: &Store, id: ExprId) -> bool {
    matches!((&store.get(id).op, &store.get(id).payload), (Op::Integer, Payload::Int(1)))
}

// Local rational ops (mirror expr_core helpers)
fn gcd_i64(mut a: i64, mut b: i64) -> i64 {
    if a == 0 { return b.abs() }
    if b == 0 { return a.abs() }
    while b != 0 { let t = a % b; a = b; b = t; }
    a.abs()
}
fn normalize_rat(num: i64, den: i64) -> (i64, i64) {
    let mut n = num; let mut d = den;
    if d < 0 { n = -n; d = -d; }
    if n == 0 { return (0, 1); }
    let g = gcd_i64(n.abs(), d);
    (n / g, d / g)
}
fn rat_add(a: (i64,i64), b: (i64,i64)) -> (i64,i64) {
    normalize_rat(a.0 * b.1 + b.0 * a.1, a.1 * b.1)
}
fn rat_mul(a: (i64,i64), b: (i64,i64)) -> (i64,i64) {
    normalize_rat(a.0 * b.0, a.1 * b.1)
}


⸻

CLI demo

crates/cli/Cargo.toml

[package]
name = "matika_cli"
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"

[dependencies]
expr_core = { path = "../expr_core" }
simplify  = { path = "../simplify" }

crates/cli/src/main.rs

use expr_core::Store;
use simplify::simplify;

fn main() {
    let mut st = Store::new();
    let x = st.sym("x");
    let one = st.int(1);

    // Build (x + 1)^2 + 3*(x + 1)
    let xp1 = st.add(vec![x, one]);
    let expr = st.add(vec![
        st.pow(xp1, st.int(2)),
        st.mul(vec![st.int(3), xp1])
    ]);

    println!("Raw:        {}", st.to_string(expr));
    let s = simplify(&mut st, expr);
    println!("Simplified: {}", st.to_string(s));

    // Another: (2x + 3x) + (1/2)x + 1/2
    let two_x = st.mul(vec![st.int(2), x]);
    let three_x = st.mul(vec![st.int(3), x]);
    let half = st.rat(1, 2);
    let half_x = st.mul(vec![half, x]);
    let expr2 = st.add(vec![two_x, three_x, half_x, half]);
    println!("Expr2 raw:  {}", st.to_string(expr2));
    let s2 = simplify(&mut st, expr2);
    println!("Expr2 simp: {}", st.to_string(s2));
}

Run it:

cd matika
cargo run -p matika_cli

You should see canonicalized raw forms and simplified output with like-term collection.

⸻

Stubs for the rest (keep the repo compiling cleanly)

crates/polys/Cargo.toml

[package]
name = "polys"
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"
[dependencies]
expr_core = { path = "../expr_core" }

crates/polys/src/lib.rs

//! Polynomial types/algorithms (stub).
#![allow(unused)]
pub struct Poly;

Repeat the same pattern for calculus, solver, matrix, assumptions, api, and io (minimal lib.rs with a module doc so the workspace builds).

⸻

Why this is “lightweight & scalable”
	•	Lightweight: no external crates right now; tiny deterministic hasher; simple i64 rational for bootstrap.
	•	Scalable: immutable DAG + hash‑consing ensures structural sharing; canonical Add/Mul prevent operand-order churn; simplifier works by split‑coeff + map reduce (linear in child count).
	•	Extensible: all heavy features (polys, calculus, solver) are separate crates—add them without touching the kernel.

⸻

Next steps you can do immediately
	1.	Numeric backend feature flags
Swap i64 rational to big‑int/MPQ when you need it:
	•	Add optional dependency rug or num-bigint in expr_core and gate payload behind #[cfg(feature = "bigint")].
	2.	Combine powers in simplify_mul
Implement x^a * x^b → x^(a+b) by grouping factors with the same base signature.
	3.	Partial fractions & rational integration
Fill polys with PRS GCD + partial fractions; then add calculus::integrate_rational.
	4.	Assumptions skeleton
Add a context param to simplify and safe rewrites (e.g., avoid sqrt(x*x)->x unless x≥0).
	5.	FFI/Python
In api, add pyo3 bindings to expose parse/build/simplify/print to Python; keep it an optional feature.

⸻

If you want, I can extend this with a pattern rule registry (AC-aware) and a small unit test suite next, but the above is enough to start coding and iterating.