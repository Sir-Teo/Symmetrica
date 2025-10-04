//! Assumptions module v1.5: tri-valued logic and basic property lattice per symbol.
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
    Integer,
    Nonzero,
}

#[derive(Default, Clone, Debug)]
pub struct Context {
    map: HashMap<String, HashSet<Prop>>,
}

impl Context {
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }

    /// Assume a property for a symbol name.
    pub fn assume<S: Into<String>>(&mut self, sym: S, prop: Prop) {
        self.map.entry(sym.into()).or_default().insert(prop);
    }

    /// Query if a symbol is known to have a property.
    pub fn has(&self, sym: &str, prop: Prop) -> Truth {
        match self.map.get(sym) {
            Some(set) => {
                if set.contains(&prop) {
                    Truth::True
                } else {
                    Truth::Unknown
                }
            }
            None => Truth::Unknown,
        }
    }
}

// Default is derived; `new()` is provided for explicit construction convenience.

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
}
