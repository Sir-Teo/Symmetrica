//! Rewrite scheduler v0
//! - Applies rewrite_basic repeatedly up to a step cap (termination guard)
//! - Returns final ExprId and basic stats

use crate::rewrite::rewrite_basic;
use expr_core::{ExprId, Store};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RewriteStats {
    pub steps: usize,
    pub changed: bool,
    pub nodes_before: usize,
    pub nodes_after: usize,
}

/// Apply `rewrite_basic` repeatedly until a fixpoint is reached or `max_steps` is hit.
/// Returns (final_expr, stats).
pub fn rewrite_fixpoint(store: &mut Store, id: ExprId, max_steps: usize) -> (ExprId, RewriteStats) {
    let before = count_nodes(store, id);
    if max_steps == 0 {
        return (
            id,
            RewriteStats { steps: 0, changed: false, nodes_before: before, nodes_after: before },
        );
    }

    let mut cur = id;
    let mut steps = 0;
    loop {
        if steps >= max_steps {
            let after = count_nodes(store, cur);
            return (
                cur,
                RewriteStats {
                    steps,
                    changed: cur != id,
                    nodes_before: before,
                    nodes_after: after,
                },
            );
        }
        let next = rewrite_basic(store, cur);
        steps += 1;
        if next == cur {
            let after = count_nodes(store, cur);
            return (
                cur,
                RewriteStats {
                    steps,
                    changed: cur != id,
                    nodes_before: before,
                    nodes_after: after,
                },
            );
        }
        cur = next;
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_steps_cap_returns_original() {
        let mut st = Store::new();
        let x = st.sym("x");
        let (out, stats) = rewrite_fixpoint(&mut st, x, 0);
        assert_eq!(out, x);
        assert_eq!(stats.steps, 0);
        assert!(!stats.changed);
        assert_eq!(stats.nodes_before, stats.nodes_after);
    }

    #[test]
    fn single_step_rewrite() {
        let mut st = Store::new();
        let zero = st.int(0);
        let sin0 = st.func("sin", vec![zero]);
        let (out, stats) = rewrite_fixpoint(&mut st, sin0, 4);
        assert_eq!(out, st.int(0));
        assert!(stats.changed);
        assert!(stats.steps >= 1);
        assert!(stats.nodes_after <= stats.nodes_before);
    }

    #[test]
    fn cap_one_allows_single_pass() {
        let mut st = Store::new();
        let one = st.int(1);
        let ln1 = st.func("ln", vec![one]);
        let x = st.sym("x");
        let one2 = st.int(1);
        let x1 = st.pow(x, one2);
        let expr = st.add(vec![ln1, x1]);
        let (out, stats) = rewrite_fixpoint(&mut st, expr, 1);
        // After one pass, both ln(1)->0 and x^1->x should have been applied due to bottom-up rewrite
        // so result is x
        let x = st.sym("x");
        assert_eq!(out, x);
        assert!(stats.changed);
        assert_eq!(stats.steps, 1);
        assert!(stats.nodes_after < stats.nodes_before);
    }
}
