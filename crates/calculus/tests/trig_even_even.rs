//! Tests for even-even mixed trig products: sin^(2k)(x) * cos^(2l)(x)

use calculus::integrate;
use expr_core::{ExprId, Store};
use simplify::simplify;

#[test]
fn integrate_sin2_cos2_structure_check() {
    // ∫ sin^2(x) * cos^2(x) dx
    let mut st = Store::new();
    let x = st.sym("x");
    let sinx = st.func("sin", vec![x]);
    let cosx = st.func("cos", vec![x]);
    let two = st.int(2);
    let sin2 = st.pow(sinx, two);
    let cos2 = st.pow(cosx, two);
    let integrand = st.mul(vec![sin2, cos2]);

    let res = integrate(&mut st, integrand, "x").expect("∫ sin^2 cos^2");

    // Build expected via reduction
    fn reduce_sin_m_cos_n(st: &mut Store, x: ExprId, m: i64, n: i64) -> ExprId {
        if m == 0 {
            let cosx = st.func("cos", vec![x]);
            let n_e = st.int(n);
            let cos_n = st.pow(cosx, n_e);
            return integrate(st, cos_n, "x").unwrap();
        }
        if n == 0 {
            let sinx = st.func("sin", vec![x]);
            let m_e = st.int(m);
            let sin_m = st.pow(sinx, m_e);
            return integrate(st, sin_m, "x").unwrap();
        }
        let sinx = st.func("sin", vec![x]);
        let cosx = st.func("cos", vec![x]);
        let sin_m1_e = st.int(m - 1);
        let cos_n1_e = st.int(n + 1);
        let sin_m1 = st.pow(sinx, sin_m1_e);
        let cos_n1 = st.pow(cosx, cos_n1_e);
        let c1 = st.rat(-1, m + n);
        let term1 = st.mul(vec![c1, sin_m1, cos_n1]);
        let inner = reduce_sin_m_cos_n(st, x, m - 2, n);
        let c2 = st.rat(m - 1, m + n);
        let term2 = st.mul(vec![c2, inner]);
        let sum = st.add(vec![term1, term2]);
        simplify(st, sum)
    }
    let expected = reduce_sin_m_cos_n(&mut st, x, 2, 2);
    let res_s = simplify(&mut st, res);
    let expected_s = simplify(&mut st, expected);
    assert_eq!(st.get(res_s).digest, st.get(expected_s).digest);
}

#[test]
fn integrate_sin4_cos2_structure_check() {
    // ∫ sin^4(x) * cos^2(x) dx
    let mut st = Store::new();
    let x = st.sym("x");
    let sinx = st.func("sin", vec![x]);
    let cosx = st.func("cos", vec![x]);
    let four = st.int(4);
    let two = st.int(2);
    let sin4 = st.pow(sinx, four);
    let cos2 = st.pow(cosx, two);
    let integrand = st.mul(vec![sin4, cos2]);

    let res = integrate(&mut st, integrand, "x").expect("∫ sin^4 cos^2");

    // Build expected via reduction
    fn reduce_sin_m_cos_n(st: &mut Store, x: ExprId, m: i64, n: i64) -> ExprId {
        if m == 0 {
            let cosx = st.func("cos", vec![x]);
            let n_e = st.int(n);
            let cos_n = st.pow(cosx, n_e);
            return integrate(st, cos_n, "x").unwrap();
        }
        if n == 0 {
            let sinx = st.func("sin", vec![x]);
            let m_e = st.int(m);
            let sin_m = st.pow(sinx, m_e);
            return integrate(st, sin_m, "x").unwrap();
        }
        let sinx = st.func("sin", vec![x]);
        let cosx = st.func("cos", vec![x]);
        let sin_m1_e = st.int(m - 1);
        let cos_n1_e = st.int(n + 1);
        let sin_m1 = st.pow(sinx, sin_m1_e);
        let cos_n1 = st.pow(cosx, cos_n1_e);
        let c1 = st.rat(-1, m + n);
        let term1 = st.mul(vec![c1, sin_m1, cos_n1]);
        let inner = reduce_sin_m_cos_n(st, x, m - 2, n);
        let c2 = st.rat(m - 1, m + n);
        let term2 = st.mul(vec![c2, inner]);
        let sum = st.add(vec![term1, term2]);
        simplify(st, sum)
    }
    let expected = reduce_sin_m_cos_n(&mut st, x, 4, 2);
    let res_s = simplify(&mut st, res);
    let expected_s = simplify(&mut st, expected);
    assert_eq!(st.get(res_s).digest, st.get(expected_s).digest);
}
