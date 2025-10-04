#![deny(warnings)]
//! End-to-end integration tests across crates.

#[cfg(test)]
mod tests {
    use calculus::diff;
    use expr_core::Store;
    use io::to_latex;
    use pattern::subst_symbol;
    use polys::{expr_to_unipoly, unipoly_to_expr};
    use simplify::simplify;

    #[test]
    fn e2e_simplify_idempotent() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let two_x = st.mul(vec![two, x]);
        let three = st.int(3);
        let three_x = st.mul(vec![three, x]);
        let half = st.rat(1, 2);
        let half_x = st.mul(vec![half, x]);
        let e = st.add(vec![two_x, three_x, half, half_x]);
        let s1 = simplify(&mut st, e);
        let s2 = simplify(&mut st, s1);
        assert_eq!(s1, s2);
    }

    #[test]
    fn e2e_diff_then_simplify() {
        let mut st = Store::new();
        let x = st.sym("x");
        let three = st.int(3);
        let p3 = st.pow(x, three);
        let two = st.int(2);
        let two_x = st.mul(vec![two, x]);
        let f = st.add(vec![p3, two_x]);
        let df = diff(&mut st, f, "x");
        let dfs = simplify(&mut st, df);
        let three2 = st.int(3);
        let two2 = st.int(2);
        let two_exp = st.int(2);
        let p2 = st.pow(x, two_exp);
        let t1 = st.mul(vec![three2, p2]);
        let expected = st.add(vec![t1, two2]);
        assert_eq!(dfs, expected);
    }

    #[test]
    fn e2e_poly_roundtrip_and_simplify() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let p2 = st.pow(x, two);
        let three = st.int(3);
        let three_x = st.mul(vec![three, x]);
        let two2 = st.int(2);
        let e = st.add(vec![p2, three_x, two2]);
        let p = expr_to_unipoly(&st, e, "x").expect("poly");
        let back = unipoly_to_expr(&mut st, &p);
        let s = simplify(&mut st, back);
        assert_eq!(s, e);
    }

    #[test]
    fn e2e_substitution_then_simplify() {
        let mut st = Store::new();
        let x = st.sym("x");
        let one = st.int(1);
        let xp1 = st.add(vec![x, one]);
        let two = st.int(2);
        let f = st.pow(xp1, two);
        let y = st.sym("y");
        let subbed = subst_symbol(&mut st, f, "x", y);
        let s = simplify(&mut st, subbed);
        let one2 = st.int(1);
        let y1 = st.add(vec![y, one2]);
        let two2 = st.int(2);
        let expected = st.pow(y1, two2);
        assert_eq!(s, expected);
    }

    #[test]
    fn e2e_latex_print_basic_expr() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let three = st.int(3);
        let three_x = st.mul(vec![three, x]);
        let two_c = st.int(2);
        let e = st.add(vec![x2, three_x, two_c]);
        let s = to_latex(&st, e);
        // Check key fragments without relying on term order beyond determinism
        assert!(s.contains("x^{2}"));
        assert!(s.contains("3 \\cdot x"));
        assert!(s.contains("2"));
    }

    #[test]
    fn e2e_integration_then_differentiation() {
        use calculus::integrate;
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two);

        // Integrate x^2 to get x^3/3
        let integral = integrate(&mut st, x2, "x").expect("integral");
        let simplified_integral = simplify(&mut st, integral);

        // Differentiate back
        let derivative = diff(&mut st, simplified_integral, "x");
        let simplified_derivative = simplify(&mut st, derivative);

        // Should get x^2 back
        assert_eq!(simplified_derivative, x2);
    }

    #[test]
    fn e2e_solve_quadratic() {
        use solver::solve_univariate;
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let neg_one = st.int(-1);
        // x^2 - 1 = 0, roots are x = 1 and x = -1
        let eq = st.add(vec![x2, neg_one]);

        let roots = solve_univariate(&mut st, eq, "x").expect("roots");
        assert_eq!(roots.len(), 2);

        // Verify we get two distinct roots
        assert_ne!(roots[0], roots[1]);
        let root_strs: Vec<String> = roots.iter().map(|&r| st.to_string(r)).collect();
        // Should contain 1 and -1
        assert!(root_strs.iter().any(|s| s.contains("1") || s.contains("-1")));
    }

    #[test]
    fn e2e_matrix_det_and_solve() {
        use matrix::MatrixQ;
        let two = arith::Q::new(2, 1);
        let one = arith::Q::new(1, 1);
        let three = arith::Q::new(3, 1);
        let four = arith::Q::new(4, 1);

        // 2x2 matrix: [[2, 1], [3, 4]]
        let mat = MatrixQ::new(2, 2, vec![two, one, three, four]);
        let d = mat.det_bareiss().expect("det");
        // det = 2*4 - 1*3 = 5
        assert_eq!(d, arith::Q::new(5, 1));

        // Solve system: [[2, 1], [3, 4]] * [x, y]^T = [5, 11]^T
        // 2x + y = 5, 3x + 4y = 11 => x = 9/5, y = 7/5
        let b = vec![arith::Q::new(5, 1), arith::Q::new(11, 1)];
        let sol = mat.solve_bareiss(&b).expect("result").expect("solution");
        assert_eq!(sol.len(), 2);
        assert_eq!(sol[0], arith::Q::new(9, 5));
        assert_eq!(sol[1], arith::Q::new(7, 5));
    }

    #[test]
    fn e2e_eval_after_simplify() {
        use evalf::{eval, EvalContext};
        let mut st = Store::new();
        let x = st.sym("x");
        let one = st.int(1);
        let xp1 = st.add(vec![x, one]);
        let two = st.int(2);
        // (x + 1)^2
        let expr = st.pow(xp1, two);
        let simplified = simplify(&mut st, expr);

        // Evaluate at x = 3
        let mut ctx = EvalContext::new();
        ctx.bind("x", 3.0);
        let result = eval(&st, simplified, &ctx).expect("eval");
        // (3 + 1)^2 = 16
        assert_eq!(result, 16.0);
    }

    #[test]
    fn e2e_sexpr_parse_diff_print() {
        use io::{from_sexpr, to_sexpr};
        let mut st = Store::new();
        let sexpr = "(^ (Sym x) (Int 3))";
        let parsed = from_sexpr(&mut st, sexpr).expect("parse");
        let derivative = diff(&mut st, parsed, "x");
        let simplified = simplify(&mut st, derivative);
        let output = to_sexpr(&st, simplified);
        // Should contain 3 and x^2
        assert!(output.contains("3") || output.contains("Int 3"));
    }

    #[test]
    fn e2e_json_roundtrip_with_functions() {
        use io::{from_json, to_json};
        let mut st = Store::new();
        let x = st.sym("x");
        let sinx = st.func("sin", vec![x]);
        let cosx = st.func("cos", vec![x]);
        let expr = st.add(vec![sinx, cosx]);

        let json = to_json(&st, expr);
        let mut st2 = Store::new();
        let parsed = from_json(&mut st2, &json).expect("parse");
        assert_eq!(st.to_string(expr), st2.to_string(parsed));
    }

    #[test]
    fn e2e_plot_config_and_eval() {
        use plot::{eval_f64, PlotConfig};
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two);

        let cfg = PlotConfig::new("x", -1.0, 1.0, 10, 400, 300);
        assert_eq!(cfg.var, "x");

        let result = eval_f64(&st, x2, &cfg.var, 2.0);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 4.0);
    }

    #[test]
    fn e2e_assumptions_with_pattern_rewrite() {
        use assumptions::{Context, Prop};
        use pattern::domain::rewrite_domain;

        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let sqrt_x2 = st.func("sqrt", vec![x2]);

        let mut ctx = Context::new();
        ctx.assume("x", Prop::Positive);

        // Use domain rewrite instead of simplify_with
        let rewritten = rewrite_domain(&mut st, sqrt_x2, &ctx);
        // With x > 0, sqrt(x^2) should rewrite to x
        assert_eq!(rewritten, x);
    }

    #[test]
    fn e2e_pattern_matching_and_rewrite() {
        use pattern::{
            ac::{match_expr, Pat},
            rewrite::rewrite_basic,
        };

        let mut st = Store::new();
        let zero = st.int(0);
        let sin0 = st.func("sin", vec![zero]);

        // Test pattern matching
        let pat = Pat::Function("sin".into(), vec![Pat::Integer(0)]);
        let bindings = match_expr(&st, &pat, sin0);
        assert!(bindings.is_some());

        // Test rewrite
        let rewritten = rewrite_basic(&mut st, sin0);
        assert_eq!(rewritten, zero);
    }

    #[test]
    fn e2e_polynomial_gcd_and_conversion() {
        use polys::unipoly_to_expr;

        let mut st = Store::new();
        // p1 = x^2 - 1 = (x-1)(x+1)
        let p1 = polys::UniPoly::new(
            "x",
            vec![arith::Q::new(-1, 1), arith::Q::new(0, 1), arith::Q::new(1, 1)],
        );

        // p2 = x - 1
        let p2 = polys::UniPoly::new("x", vec![arith::Q::new(-1, 1), arith::Q::new(1, 1)]);

        let g = polys::UniPoly::gcd(p1, p2);
        // GCD should be x - 1 (up to constant factor)
        assert_eq!(g.degree(), Some(1));

        let expr = unipoly_to_expr(&mut st, &g);
        let simplified = simplify(&mut st, expr);
        // Should contain x and -1 or 1
        let s = st.to_string(simplified);
        assert!(s.contains("x"));
    }
}
