//! Tests for even-power single trig integrals: sin^(2k)(x), cos^(2k)(x)

use calculus::integrate;
use expr_core::{ExprId, Store};
use simplify::simplify;

#[test]
fn integrate_sin_four_structure_check() {
    let mut st = Store::new();
    let x = st.sym("x");
    let sinx = st.func("sin", vec![x]);
    let four = st.int(4);
    let sin4 = st.pow(sinx, four);

    let res = integrate(&mut st, sin4, "x").expect("∫ sin^4(x) dx");
    // Build expected via the same reduction recursion used in the impl
    fn red_sin_even(st: &mut Store, x: ExprId, n: i64) -> ExprId {
        if n == 0 {
            return x;
        }
        let cosx = st.func("cos", vec![x]);
        let sinx = st.func("sin", vec![x]);
        let exp_e = st.int(n - 1);
        let sin_pow = st.pow(sinx, exp_e);
        let r1 = st.rat(-1, n);
        let term1 = st.mul(vec![r1, cosx, sin_pow]);
        let inner = red_sin_even(st, x, n - 2);
        let r2 = st.rat(n - 1, n);
        let term2 = st.mul(vec![r2, inner]);
        let sum = st.add(vec![term1, term2]);
        simplify(st, sum)
    }
    let expected = red_sin_even(&mut st, x, 4);
    let res_s = simplify(&mut st, res);
    let expected_s = simplify(&mut st, expected);
    assert_eq!(st.get(res_s).digest, st.get(expected_s).digest);
}

#[test]
fn integrate_cos_six_structure_check() {
    let mut st = Store::new();
    let x = st.sym("x");
    let cosx = st.func("cos", vec![x]);
    let six = st.int(6);
    let cos6 = st.pow(cosx, six);

    let res = integrate(&mut st, cos6, "x").expect("∫ cos^6(x) dx");
    // Build expected via the same reduction recursion used in the impl
    fn red_cos_even(st: &mut Store, x: ExprId, n: i64) -> ExprId {
        if n == 0 {
            return x;
        }
        let sinx = st.func("sin", vec![x]);
        let cosx = st.func("cos", vec![x]);
        let exp_e = st.int(n - 1);
        let cos_pow = st.pow(cosx, exp_e);
        let r1 = st.rat(1, n);
        let term1 = st.mul(vec![r1, sinx, cos_pow]);
        let inner = red_cos_even(st, x, n - 2);
        let r2 = st.rat(n - 1, n);
        let term2 = st.mul(vec![r2, inner]);
        let sum = st.add(vec![term1, term2]);
        simplify(st, sum)
    }
    let expected = red_cos_even(&mut st, x, 6);
    let res_s = simplify(&mut st, res);
    let expected_s = simplify(&mut st, expected);
    assert_eq!(st.get(res_s).digest, st.get(expected_s).digest);
}
