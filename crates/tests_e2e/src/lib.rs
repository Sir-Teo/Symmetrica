#![deny(warnings)]
//! End-to-end integration tests across crates.

#[cfg(test)]
mod tests {
    use calculus::diff;
    use expr_core::Store;
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
}
