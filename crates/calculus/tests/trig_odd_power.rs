//! Tests for generalized sin^m(x) * cos^n(x) integration with odd exponents

use calculus::{diff, integrate};
use expr_core::Store;
use simplify::simplify;

#[test]
fn integrate_sin_cubed() {
    let mut st = Store::new();
    let x = st.sym("x");
    let sinx = st.func("sin", vec![x]);
    let three = st.int(3);
    let sin3 = st.pow(sinx, three);

    let int = integrate(&mut st, sin3, "x").expect("∫ sin^3(x) dx");
    let d = diff(&mut st, int, "x");
    let d_s = simplify(&mut st, d);
    // Expected structural form: sin(x) - sin(x) * cos(x)^2
    let sinx2 = st.func("sin", vec![x]);
    let cosx = st.func("cos", vec![x]);
    let two = st.int(2);
    let cos2 = st.pow(cosx, two);
    let neg1 = st.int(-1);
    let sin_cos2 = st.mul(vec![neg1, sinx2, cos2]);
    let expected = st.add(vec![sinx2, sin_cos2]);
    let expected_s = simplify(&mut st, expected);
    assert_eq!(st.get(d_s).digest, st.get(expected_s).digest);
}

#[test]
fn integrate_cos_cubed() {
    let mut st = Store::new();
    let x = st.sym("x");
    let cosx = st.func("cos", vec![x]);
    let three = st.int(3);
    let cos3 = st.pow(cosx, three);

    let int = integrate(&mut st, cos3, "x").expect("∫ cos^3(x) dx");
    let d = diff(&mut st, int, "x");
    let d_s = simplify(&mut st, d);
    // Expected structural form: cos(x) - cos(x) * sin(x)^2
    let cosx2 = st.func("cos", vec![x]);
    let sinx = st.func("sin", vec![x]);
    let two = st.int(2);
    let sin2 = st.pow(sinx, two);
    let neg1 = st.int(-1);
    let cos_sin2 = st.mul(vec![neg1, cosx2, sin2]);
    let expected = st.add(vec![cosx2, cos_sin2]);
    let expected_s = simplify(&mut st, expected);
    assert_eq!(st.get(d_s).digest, st.get(expected_s).digest);
}

#[test]
fn integrate_sin3_cos2() {
    // ∫ sin^3(x) * cos^2(x) dx
    let mut st = Store::new();
    let x = st.sym("x");
    let sinx = st.func("sin", vec![x]);
    let cosx = st.func("cos", vec![x]);
    let three = st.int(3);
    let two = st.int(2);
    let sin3 = st.pow(sinx, three);
    let cos2 = st.pow(cosx, two);
    let integrand = st.mul(vec![sin3, cos2]);

    let int = integrate(&mut st, integrand, "x").expect("∫ sin^3 * cos^2");
    let d = diff(&mut st, int, "x");
    let d_s = simplify(&mut st, d);
    // Expected structural form: sin(x) * cos(x)^2 - sin(x) * cos(x)^4
    let sinx2 = st.func("sin", vec![x]);
    let cosx = st.func("cos", vec![x]);
    let two = st.int(2);
    let four = st.int(4);
    let cos2 = st.pow(cosx, two);
    let cos4 = st.pow(cosx, four);
    let neg1 = st.int(-1);
    let term1 = st.mul(vec![sinx2, cos2]);
    let term2 = st.mul(vec![neg1, sinx2, cos4]);
    let expected = st.add(vec![term1, term2]);
    let expected_s = simplify(&mut st, expected);
    assert_eq!(st.get(d_s).digest, st.get(expected_s).digest);
}

#[test]
fn integrate_sin2_cos() {
    // ∫ sin^2(x) * cos(x) dx = (1/3) sin^3(x)
    let mut st = Store::new();
    let x = st.sym("x");
    let sinx = st.func("sin", vec![x]);
    let cosx = st.func("cos", vec![x]);
    let two = st.int(2);
    let sin2 = st.pow(sinx, two);
    let integrand = st.mul(vec![sin2, cosx]);

    let int = integrate(&mut st, integrand, "x").expect("∫ sin^2 * cos");
    let d = diff(&mut st, int, "x");
    let d_s = simplify(&mut st, d);
    let orig_s = simplify(&mut st, integrand);
    assert_eq!(st.get(d_s).digest, st.get(orig_s).digest);
}

#[test]
fn integrate_sin_cos2() {
    // ∫ sin(x) * cos^2(x) dx = -cos^3(x)/3
    let mut st = Store::new();
    let x = st.sym("x");
    let sinx = st.func("sin", vec![x]);
    let cosx = st.func("cos", vec![x]);
    let two = st.int(2);
    let cos2 = st.pow(cosx, two);
    let integrand = st.mul(vec![sinx, cos2]);

    let int = integrate(&mut st, integrand, "x").expect("∫ sin * cos^2");
    let d = diff(&mut st, int, "x");
    let d_s = simplify(&mut st, d);
    let orig_s = simplify(&mut st, integrand);
    assert_eq!(st.get(d_s).digest, st.get(orig_s).digest);
}
