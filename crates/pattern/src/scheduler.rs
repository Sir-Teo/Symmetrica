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

    #[test]
    fn fixpoint_with_no_change() {
        let mut st = Store::new();
        let x = st.sym("x");
        let (out, stats) = rewrite_fixpoint(&mut st, x, 10);
        assert_eq!(out, x);
        assert!(!stats.changed);
        assert_eq!(stats.steps, 1); // Takes 1 step to realize no change
    }

    #[test]
    fn nested_rewrites_converge() {
        let mut st = Store::new();
        // exp(0) + sin(0) + cos(0) -> 1 + 0 + 1 -> 2
        let zero = st.int(0);
        let exp0 = st.func("exp", vec![zero]);
        let zero2 = st.int(0);
        let sin0 = st.func("sin", vec![zero2]);
        let zero3 = st.int(0);
        let cos0 = st.func("cos", vec![zero3]);
        let expr = st.add(vec![exp0, sin0, cos0]);

        let (out, stats) = rewrite_fixpoint(&mut st, expr, 10);
        assert!(stats.changed);
        // Result should simplify to 2
        let expected = st.int(2);
        assert_eq!(out, expected);
    }

    #[test]
    fn max_steps_cap_enforced() {
        let mut st = Store::new();
        // Create nested expression that would require many steps
        let zero = st.int(0);
        let mut expr = st.func("sin", vec![zero]);
        for _ in 0..5 {
            let z = st.int(0);
            let s = st.func("sin", vec![z]);
            expr = st.add(vec![expr, s]);
        }

        let (_, stats) = rewrite_fixpoint(&mut st, expr, 2);
        assert_eq!(stats.steps, 2); // Should stop at max_steps
    }

    #[test]
    fn nodes_count_decreases_on_simplification() {
        let mut st = Store::new();
        let one = st.int(1);
        let ln1 = st.func("ln", vec![one]);

        let (out, stats) = rewrite_fixpoint(&mut st, ln1, 5);
        assert_eq!(out, st.int(0));
        assert!(stats.nodes_after <= stats.nodes_before);
        assert!(stats.changed);
    }
}
