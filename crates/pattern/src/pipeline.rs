//! Rewrite pipeline: combine core passes under a step cap.
//! Order per iteration: rewrite_basic -> domain -> registry (best by node count) -> simplify_with.

use crate::{
    domain::rewrite_domain,
    registry::{apply_best_rule_by_node_count, Rule},
    rewrite::rewrite_basic,
};
use assumptions::Context as AssumptionsContext;
use expr_core::{ExprId, Store};

/// Run the composite rewrite pipeline with a maximum number of iterations.
/// Returns the final expression (canonicalized via simplify_with in each iteration).
pub fn rewrite_pipeline(
    store: &mut Store,
    id: ExprId,
    ctx: &AssumptionsContext,
    rules: &[Rule],
    max_steps: usize,
) -> ExprId {
    if max_steps == 0 {
        return id;
    }
    let mut cur = id;
    for _ in 0..max_steps {
        let a = rewrite_basic(store, cur);
        let b = rewrite_domain(store, a, ctx);
        let c = match apply_best_rule_by_node_count(store, b, rules) {
            Some(n) => n,
            None => b,
        };
        let d = simplify::simplify_with(store, c, ctx);
        if d == cur {
            return d;
        }
        cur = d;
    }
    cur
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::Rule;

    #[test]
    fn sin_zero_plus_sin_zero_collapses() {
        let mut st = Store::new();
        let zero = st.int(0);
        let sin0a = st.func("sin", vec![zero]);
        // make another 0 and another sin(0) to avoid reuse
        let zero2 = st.int(0);
        let sin0b = st.func("sin", vec![zero2]);
        let expr = st.add(vec![sin0a, sin0b]);

        let rules = vec![Rule {
            name: "sin(0)->0",
            pattern: crate::ac::Pat::Function("sin".into(), vec![crate::ac::Pat::Integer(0)]),
            guard: None,
            build: |st, _| st.int(0),
        }];

        let ctx = AssumptionsContext::new();
        let out = rewrite_pipeline(&mut st, expr, &ctx, &rules, 4);
        assert_eq!(out, st.int(0));
    }

    #[test]
    fn nested_pow_and_domain_then_simplify() {
        // exp(ln(x)) + x^1 -> x + x -> 2*x, with x>0
        let mut st = Store::new();
        let x = st.sym("x");
        let lnx = st.func("ln", vec![x]);
        let ex = st.func("exp", vec![lnx]);
        let one = st.int(1);
        let x1 = st.pow(x, one);
        let expr = st.add(vec![ex, x1]);

        let mut ctx = AssumptionsContext::new();
        ctx.assume("x", assumptions::Prop::Positive);

        let rules: Vec<Rule> = vec![]; // not needed for this case
        let out = rewrite_pipeline(&mut st, expr, &ctx, &rules, 6);
        let two = st.int(2);
        assert_eq!(out, st.mul(vec![two, x]));
    }
}
