#![deny(warnings)]
//! expr_core: minimal immutable DAG expression kernel with hash-consing.
//! - Op/Node/ExprId
//! - Store with interning + canonical Add/Mul/Pow
//! - Basic numeric payload (i64 integers; small rationals)
//! - Deterministic digest (FNV-1a 64) for canonical ordering

use arith::{normalize_rat, rat_add, rat_mul};
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

#[derive(Default)]
pub struct Store {
    nodes: Vec<Node>,
    interner: HashMap<NodeKey, ExprId>,
}

impl Store {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, id: ExprId) -> &Node {
        &self.nodes[id.0]
    }

    // ---- Constructors (atoms) ----
    pub fn sym<S: Into<String>>(&mut self, name: S) -> ExprId {
        self.intern(Op::Symbol, Payload::Sym(name.into()), vec![])
    }
    pub fn int(&mut self, n: i64) -> ExprId {
        self.intern(Op::Integer, Payload::Int(n), vec![])
    }
    pub fn rat(&mut self, num: i64, den: i64) -> ExprId {
        let (n, d) = normalize_rat(num, den);
        if d == 1 {
            return self.int(n);
        }
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
                    for c in &self.get(t).children {
                        terms.push(*c);
                    }
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
        terms.retain(|&id| {
            let n = self.get(id);
            !matches!((&n.op, &n.payload), (&Op::Integer, &Payload::Int(0)))
        });

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
                    for c in &self.get(f).children {
                        factors.push(*c);
                    }
                }
                Op::Integer => {
                    if let Payload::Int(k) = &self.get(f).payload {
                        if *k == 0 {
                            return self.int(0);
                        }
                        num = rat_mul(num, (*k, 1));
                    }
                }
                Op::Rational => {
                    if let Payload::Rat(n, d) = &self.get(f).payload {
                        if *n == 0 {
                            return self.int(0);
                        }
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
        factors.retain(|&id| {
            let n = self.get(id);
            !matches!((&n.op, &n.payload), (&Op::Integer, &Payload::Int(1)))
        });

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
            if matches!(
                (&self.get(base).op, &self.get(base).payload),
                (Op::Integer, Payload::Int(0))
            ) {
                return self.intern(Op::Pow, Payload::None, vec![base, exp]);
            }
            return self.int(1);
        }
        self.intern(Op::Pow, Payload::None, vec![base, exp])
    }

    // ---- Printing (very small, precedence-aware) ----
    pub fn to_string(&self, id: ExprId) -> String {
        fn prec(op: &Op) -> u8 {
            match op {
                Op::Add => 1,
                Op::Mul => 2,
                Op::Pow => 3,
                _ => 4,
            }
        }
        fn go(st: &Store, id: ExprId, parent_prec: u8) -> String {
            let n = st.get(id);
            let s = match (&n.op, &n.payload) {
                (Op::Integer, Payload::Int(k)) => k.to_string(),
                (Op::Rational, Payload::Rat(a, b)) => format!("{}/{}", a, b),
                (Op::Symbol, Payload::Sym(name)) => name.clone(),
                (Op::Function, Payload::Func(name)) => {
                    let args =
                        n.children.iter().map(|c| go(st, *c, 0)).collect::<Vec<_>>().join(", ");
                    format!("{name}({args})")
                }
                (Op::Add, _) => n
                    .children
                    .iter()
                    .map(|c| go(st, *c, prec(&Op::Add)))
                    .collect::<Vec<_>>()
                    .join(" + "),
                (Op::Mul, _) => n
                    .children
                    .iter()
                    .map(|c| go(st, *c, prec(&Op::Mul)))
                    .collect::<Vec<_>>()
                    .join(" * "),
                (Op::Pow, _) => {
                    let b = go(st, n.children[0], prec(&Op::Pow));
                    let e = go(st, n.children[1], prec(&Op::Pow));
                    format!("{b}^{e}")
                }
                _ => "<unknown>".into(),
            };
            if prec(&n.op) < parent_prec {
                format!("({s})")
            } else {
                s
            }
        }
        go(self, id, 0)
    }

    // ---- Interning ----
    fn intern(&mut self, op: Op, payload: Payload, children: Vec<ExprId>) -> ExprId {
        // Compute child digests; some ops (Add/Mul) expect children sorted already
        let child_digests: Vec<u64> = children.iter().map(|id| self.get(*id).digest).collect();
        let key = NodeKey { op: op.clone(), payload: payload.clone(), child_digests };

        if let Some(&id) = self.interner.get(&key) {
            return id;
        }

        // Compute digest for this node deterministically
        let digest = digest_node(
            &op,
            &payload,
            &children.iter().map(|id| self.get(*id).digest).collect::<Vec<_>>(),
        );

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
        Payload::Int(k) => {
            h.write_u8(1);
            h.write_i64(*k);
        }
        Payload::Rat(n, d) => {
            h.write_u8(2);
            h.write_i64(*n);
            h.write_i64(*d);
        }
        Payload::Sym(s) => {
            h.write_u8(3);
            h.write_bytes(s.as_bytes());
        }
        Payload::Func(s) => {
            h.write_u8(4);
            h.write_bytes(s.as_bytes());
        }
    }
    for &cd in child_digests {
        h.write_u64(cd);
    }
    h.finish()
}

fn op_tag(op: &Op) -> u8 {
    match op {
        Op::Add => 1,
        Op::Mul => 2,
        Op::Pow => 3,
        Op::Symbol => 4,
        Op::Integer => 5,
        Op::Rational => 6,
        Op::Function => 7,
    }
}

// Minimal FNV-1a 64 hasher (deterministic)
struct Fnv64(u64);
impl Fnv64 {
    fn new() -> Self {
        Self(0xcbf29ce484222325)
    }
    fn write_u8(&mut self, x: u8) {
        self.0 ^= x as u64;
        self.0 = self.0.wrapping_mul(0x100000001b3);
    }
    fn write_i64(&mut self, x: i64) {
        self.write_u64(x as u64);
    }
    fn write_u64(&mut self, x: u64) {
        for b in x.to_le_bytes() {
            self.write_u8(b);
        }
    }
    fn write_bytes(&mut self, bs: &[u8]) {
        for &b in bs {
            self.write_u8(b)
        }
    }
    fn finish(&self) -> u64 {
        self.0
    }
}

// rational helpers now sourced from `arith` crate

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_consing() {
        let mut st = Store::new();
        let x1 = st.sym("x");
        let x2 = st.sym("x");
        assert_eq!(x1, x2);
        assert_eq!(st.get(x1).digest, st.get(x2).digest);
    }

    #[test]
    fn test_add_canonical_and_deterministic() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let a = st.add(vec![x, y]);
        let b = st.add(vec![y, x]);
        assert_eq!(a, b);
        // Flatten
        let one = st.int(1);
        let two = st.int(2);
        let c = st.add(vec![a, one, two]);
        let three = st.int(3);
        let d = st.add(vec![x, y, three]);
        assert_eq!(c, d);
    }

    #[test]
    fn test_mul_canonical_and_zero_one_rules() {
        let mut st = Store::new();
        let x = st.sym("x");
        // zero annihilates
        let zero = st.int(0);
        let five = st.int(5);
        let z = st.mul(vec![x, zero, five]);
        assert_eq!(z, st.int(0));
        // one removed, rationals folded
        let two = st.int(2);
        let rat = st.rat(1, 3);
        let one = st.int(1);
        let m = st.mul(vec![two, x, rat, one]);
        let rat23 = st.rat(2, 3);
        let expected = st.mul(vec![x, rat23]);
        assert_eq!(m, expected);
    }

    #[test]
    fn test_rat_normalization() {
        let mut st = Store::new();
        // 2/4 -> 1/2
        let a = st.rat(2, 4);
        let b = st.rat(1, 2);
        assert_eq!(a, b);
        // 2/(-4) -> -1/2
        let c = st.rat(2, -4);
        let d = st.rat(-1, 2);
        assert_eq!(c, d);
        // 0/n -> 0 as integer
        let e = st.rat(0, 5);
        assert_eq!(e, st.int(0));
    }

    #[test]
    fn test_flatten_add_and_identities() {
        let mut st = Store::new();
        let x = st.sym("x");
        let zero = st.int(0);
        let one = st.int(1);
        let a = st.add(vec![x, zero]);
        let b = st.add(vec![one, x]);
        let nested = st.add(vec![a, b]);
        // Expect flattened: x + x + 1 (numeric folded)
        let expect = st.add(vec![x, x, one]);
        assert_eq!(nested, expect);
    }

    #[test]
    fn test_pow_rules_zero_one() {
        let mut st = Store::new();
        let x = st.sym("x");
        // x^1 -> x
        let one = st.int(1);
        let p1 = st.pow(x, one);
        assert_eq!(p1, x);
        // x^0 -> 1 for nonzero base
        let zero = st.int(0);
        let p0 = st.pow(x, zero);
        assert_eq!(p0, st.int(1));
        // 0^0 stays as Pow node
        let zero2 = st.int(0);
        let p_undefined = st.pow(zero, zero2);
        assert!(matches!(st.get(p_undefined).op, Op::Pow));
    }

    #[test]
    fn test_printer_precedence() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let two = st.int(2);
        let sum = st.add(vec![y, two]);
        let prod = st.mul(vec![x, sum]);
        assert_eq!(st.to_string(prod), "x * (2 + y)");
        let three = st.int(3);
        let pow = st.pow(sum, three);
        assert_eq!(st.to_string(pow), "(2 + y)^3");
    }

    #[test]
    fn test_function_construction_and_printing() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let two = st.int(2);
        let sum = st.add(vec![y, two]); // canonical prints as 2 + y
        let f = st.func("f", vec![x, sum]);
        assert_eq!(st.to_string(f), "f(x, 2 + y)");
    }

    #[test]
    fn test_function_argument_order_matters() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let f_xy = st.func("f", vec![x, y]);
        let f_yx = st.func("f", vec![y, x]);
        assert_ne!(f_xy, f_yx, "function args are ordered and not canonicalized");
    }

    #[test]
    fn test_mul_flatten_and_sorting() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let two = st.int(2);
        let y_two = st.mul(vec![y, two]);
        let nested = st.mul(vec![x, y_two]);
        let flat = st.mul(vec![x, y, two]);
        assert_eq!(nested, flat);
    }

    #[test]
    #[should_panic(expected = "zero denominator")]
    fn test_rat_zero_denominator_panics() {
        let mut st = Store::new();
        // This should panic due to assert! in normalize_rat
        let _ = st.rat(1, 0);
    }

    #[test]
    fn test_arith_q_and_helpers() {
        use arith::*;

        // Q constructors and predicates
        let q1 = Q::new(2, 4);
        assert_eq!(q1, Q(1, 2));
        assert!(!q1.is_zero());
        assert!(Q::zero().is_zero());
        assert_eq!(Q::one(), Q(1, 1));

        // Q arithmetic
        assert_eq!(add_q(Q(1, 2), Q(1, 3)), Q(5, 6));
        assert_eq!(sub_q(Q(1, 2), Q(1, 3)), Q(1, 6));
        assert_eq!(mul_q(Q(2, 3), Q(3, 5)), Q(2, 5));
        assert_eq!(div_q(Q(2, 3), Q(4, 9)), Q(3, 2));

        // Tuple helpers and gcd
        assert_eq!(gcd_i64(54, 24), 6);
        assert_eq!(q_norm(-2, -4), (1, 2));
        assert_eq!(q_add((1, 2), (1, 3)), (5, 6));
        assert_eq!(q_sub((1, 2), (1, 3)), (1, 6));
        assert_eq!(q_mul((1, 2), (2, 3)), (1, 3));
        assert_eq!(q_div((1, 2), (2, 3)), (3, 4));
        assert_eq!(rat_sub((1, 2), (1, 2)), (0, 1));
    }
}
