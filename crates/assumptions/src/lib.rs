//! Assumptions module v2: tri-valued logic and enhanced property lattice per symbol.
//! Phase I implementation: domain-aware assumptions with negative properties.
#![deny(warnings)]

use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Truth {
    True,
    False,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Prop {
    Real,
    Positive,
    Negative,
    Integer,
    Nonzero,
    Nonnegative, // Positive or zero
}

#[derive(Default, Clone, Debug)]
pub struct Context {
    // Stack of frames to support scoping. New assumptions go into the top frame.
    stack: Vec<HashMap<String, HashSet<Prop>>>,
}

impl Context {
    pub fn new() -> Self {
        Self { stack: vec![HashMap::new()] }
    }

    /// Enter a new scope frame.
    pub fn push(&mut self) {
        self.stack.push(HashMap::new());
    }

    /// Exit the top scope frame. Returns false if at base scope.
    pub fn pop(&mut self) -> bool {
        if self.stack.len() <= 1 {
            return false;
        }
        self.stack.pop();
        true
    }

    /// Assume a property for a symbol name.
    pub fn assume<S: Into<String>>(&mut self, sym: S, prop: Prop) {
        if let Some(top) = self.stack.last_mut() {
            top.entry(sym.into()).or_default().insert(prop);
        }
    }

    /// Query if a symbol is known to have a property.
    pub fn has(&self, sym: &str, prop: Prop) -> Truth {
        // Union all properties for the symbol from all frames (top overrides by union here).
        let mut props: HashSet<Prop> = HashSet::new();
        for frame in self.stack.iter().rev() {
            if let Some(set) = frame.get(sym) {
                for &p in set {
                    props.insert(p);
                }
            }
        }
        if props.is_empty() {
            return Truth::Unknown;
        }
        let closure = derive_props(&props);
        if closure.contains(&prop) {
            Truth::True
        } else {
            Truth::Unknown
        }
    }
}

// Default is derived; `new()` is provided for explicit construction convenience.

fn derive_props(base: &HashSet<Prop>) -> HashSet<Prop> {
    let mut out = base.clone();
    let mut changed = true;

    // Iterate until fixpoint to handle transitive implications
    while changed {
        let old_size = out.len();

        // Positive implies Real, Nonzero, and Nonnegative
        if out.contains(&Prop::Positive) {
            out.insert(Prop::Real);
            out.insert(Prop::Nonzero);
            out.insert(Prop::Nonnegative);
        }

        // Negative implies Real and Nonzero
        if out.contains(&Prop::Negative) {
            out.insert(Prop::Real);
            out.insert(Prop::Nonzero);
        }

        // Integer implies Real
        if out.contains(&Prop::Integer) {
            out.insert(Prop::Real);
        }

        // Nonnegative + Nonzero implies Positive
        if out.contains(&Prop::Nonnegative) && out.contains(&Prop::Nonzero) {
            out.insert(Prop::Positive);
        }

        changed = out.len() > old_size;
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assume_and_query() {
        let mut ctx = Context::new();
        assert!(matches!(ctx.has("x", Prop::Nonzero), Truth::Unknown));
        ctx.assume("x", Prop::Nonzero);
        assert!(matches!(ctx.has("x", Prop::Nonzero), Truth::True));
        assert!(matches!(ctx.has("x", Prop::Positive), Truth::Unknown));
    }

    #[test]
    fn derived_properties() {
        let mut ctx = Context::new();
        ctx.assume("x", Prop::Integer);
        // Integer implies Real
        assert!(matches!(ctx.has("x", Prop::Real), Truth::True));
        assert!(matches!(ctx.has("x", Prop::Positive), Truth::Unknown));
        ctx.assume("y", Prop::Positive);
        // Positive implies Real and Nonzero
        assert!(matches!(ctx.has("y", Prop::Real), Truth::True));
        assert!(matches!(ctx.has("y", Prop::Nonzero), Truth::True));
    }

    #[test]
    fn scoped_push_pop() {
        let mut ctx = Context::new();
        ctx.assume("x", Prop::Nonzero);
        // base scope: only Nonzero
        assert!(matches!(ctx.has("x", Prop::Nonzero), Truth::True));
        assert!(matches!(ctx.has("x", Prop::Positive), Truth::Unknown));
        ctx.push();
        ctx.assume("x", Prop::Positive);
        // inner scope: Positive implies Real and Nonzero
        assert!(matches!(ctx.has("x", Prop::Positive), Truth::True));
        assert!(matches!(ctx.has("x", Prop::Real), Truth::True));
        assert!(matches!(ctx.has("x", Prop::Nonzero), Truth::True));
        assert!(ctx.pop());
        // back to base: Positive gone
        assert!(matches!(ctx.has("x", Prop::Positive), Truth::Unknown));
        assert!(matches!(ctx.has("x", Prop::Nonzero), Truth::True));
    }

    #[test]
    fn negative_property() {
        let mut ctx = Context::new();
        ctx.assume("x", Prop::Negative);
        // Negative implies Real and Nonzero
        assert!(matches!(ctx.has("x", Prop::Negative), Truth::True));
        assert!(matches!(ctx.has("x", Prop::Real), Truth::True));
        assert!(matches!(ctx.has("x", Prop::Nonzero), Truth::True));
        // But not Positive or Nonnegative
        assert!(matches!(ctx.has("x", Prop::Positive), Truth::Unknown));
        assert!(matches!(ctx.has("x", Prop::Nonnegative), Truth::Unknown));
    }

    #[test]
    fn nonnegative_property() {
        let mut ctx = Context::new();
        ctx.assume("x", Prop::Nonnegative);
        // Nonnegative alone doesn't imply anything else (could be zero)
        assert!(matches!(ctx.has("x", Prop::Nonnegative), Truth::True));
        assert!(matches!(ctx.has("x", Prop::Positive), Truth::Unknown));
        assert!(matches!(ctx.has("x", Prop::Nonzero), Truth::Unknown));

        // But Nonnegative + Nonzero implies Positive (and Positive implies Real)
        ctx.assume("x", Prop::Nonzero);
        assert!(matches!(ctx.has("x", Prop::Positive), Truth::True));
        // Note: Since Positive implies Real, this should now be True
        assert!(matches!(ctx.has("x", Prop::Real), Truth::True));
    }

    #[test]
    fn positive_implies_nonnegative() {
        let mut ctx = Context::new();
        ctx.assume("x", Prop::Positive);
        // Positive implies Nonnegative
        assert!(matches!(ctx.has("x", Prop::Nonnegative), Truth::True));
        assert!(matches!(ctx.has("x", Prop::Nonzero), Truth::True));
        assert!(matches!(ctx.has("x", Prop::Real), Truth::True));
    }
}
