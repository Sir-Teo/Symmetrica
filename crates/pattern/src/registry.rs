//! Minimal rewrite rule registry (Phase H: Rule registry DSL, partial)
//!
//! Provides a lightweight way to register pattern-based rules with optional guards,
//! and apply the first matching rule at the expression root.

use crate::ac::{match_expr, Bindings, Pat};
use expr_core::{ExprId, Store};
use std::collections::HashSet;

pub type GuardFn = fn(store: &Store, bindings: &Bindings) -> bool;
pub type BuildFn = fn(store: &mut Store, bindings: &Bindings) -> ExprId;

#[derive(Clone)]
pub struct Rule {
    pub name: &'static str,
    pub pattern: Pat,
    pub guard: Option<GuardFn>,
    pub build: BuildFn,
}

/// Choose the matching rule that minimizes node count of the result.
/// Returns None if no rules match.
pub fn apply_best_rule_by_node_count(
    store: &mut Store,
    expr: ExprId,
    rules: &[Rule],
) -> Option<ExprId> {
    let mut best: Option<(usize, ExprId)> = None;
    for r in rules {
        if let Some(binds) = match_expr(store, &r.pattern, expr) {
            if r.guard.map(|g| g(store, &binds)).unwrap_or(true) {
                let out = (r.build)(store, &binds);
                if out == expr {
                    continue;
                }
                let cost = count_nodes(store, out);
                match best {
                    None => best = Some((cost, out)),
                    Some((bc, _)) if cost < bc => best = Some((cost, out)),
                    _ => {}
                }
            }
        }
    }
    best.map(|(_, id)| id)
}

fn count_nodes(store: &Store, id: ExprId) -> usize {
    let mut seen: HashSet<ExprId> = HashSet::new();
    let mut stack = vec![id];
    while let Some(nid) = stack.pop() {
        if !seen.insert(nid) {
            continue;
        }
        let node = store.get(nid);
        for &c in &node.children {
            stack.push(c);
        }
    }
    seen.len()
}

/// Try rules in order; return the first rewrite result if any matches at root.
pub fn apply_first_rule(store: &mut Store, expr: ExprId, rules: &[Rule]) -> Option<ExprId> {
    for r in rules {
        if let Some(binds) = match_expr(store, &r.pattern, expr) {
            if r.guard.map(|g| g(store, &binds)).unwrap_or(true) {
                let out = (r.build)(store, &binds);
                // Avoid trivial self-rewrite
                if out != expr {
                    return Some(out);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_pow_u_two_to_u_mul_u() {
        let rules = vec![Rule {
            name: "pow(u,2)->u*u",
            pattern: Pat::Pow(Box::new(Pat::Any("u".into())), Box::new(Pat::Integer(2))),
            guard: None,
            build: |st, b| {
                let u = *b.get("u").unwrap();
                st.mul(vec![u, u])
            },
        }];

        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let expr = st.pow(x, two);
        let out = apply_first_rule(&mut st, expr, &rules).unwrap();
        assert_eq!(out, st.mul(vec![x, x]));
    }

    #[test]
    fn registry_sin_zero_to_zero() {
        let rules = vec![Rule {
            name: "sin(0)->0",
            pattern: Pat::Function("sin".into(), vec![Pat::Integer(0)]),
            guard: None,
            build: |st, _| st.int(0),
        }];

        let mut st = Store::new();
        let zero = st.int(0);
        let expr = st.func("sin", vec![zero]);
        let out = apply_first_rule(&mut st, expr, &rules);
        assert_eq!(out, Some(st.int(0)));
    }

    #[test]
    fn best_rule_minimizes_node_count() {
        // Two rules: sin(0) -> 0, and sin(0) -> 0+0. Best should pick 0.
        let rules = vec![
            Rule {
                name: "sin(0)->0",
                pattern: Pat::Function("sin".into(), vec![Pat::Integer(0)]),
                guard: None,
                build: |st, _| st.int(0),
            },
            Rule {
                name: "sin(0)->0+0",
                pattern: Pat::Function("sin".into(), vec![Pat::Integer(0)]),
                guard: None,
                build: |st, _| {
                    let z1 = st.int(0);
                    let z2 = st.int(0);
                    st.add(vec![z1, z2])
                },
            },
        ];

        let mut st = Store::new();
        let zero = st.int(0);
        let expr = st.func("sin", vec![zero]);
        let out = apply_best_rule_by_node_count(&mut st, expr, &rules).unwrap();
        assert_eq!(out, st.int(0));
    }
}
