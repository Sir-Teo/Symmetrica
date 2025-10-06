//! Calculus v1 (minimal): structural differentiation for Add/Mul/Pow.
//! Phase 3: definite integrals, limit evaluation, constant folding, and special functions
mod definite;
mod diff;
mod evaluate;
mod integrate;
mod limit;
mod risch;
mod series;

pub use definite::{definite_integrate, Bound, DefiniteResult};
pub use diff::diff;
pub use evaluate::{fold_constants, try_eval_constant, try_eval_float};
pub use integrate::integrate;
pub use limit::{limit, LimitPoint as LimitPointCalc, LimitResult as LimitResultCalc};
pub use risch::{
    build_tower, detect_extension, is_exponential, is_logarithm, logarithmic_derivative,
    try_integrate_logarithmic, ExtensionType, TowerElement,
};
pub use series::{limit_poly, maclaurin, LimitPoint, LimitResult, Series};

#[cfg(test)]
mod tests {
    use super::*;
    use expr_core::Store;

    #[test]
    fn diff_of_power_and_sum() {
        let mut st = Store::new();
        let x = st.sym("x");
        // f = x^3 + 2x
        let three = st.int(3);
        let p3 = st.pow(x, three);
        let two = st.int(2);
        let two_x = st.mul(vec![two, x]);
        let f = st.add(vec![p3, two_x]);
        let df = diff(&mut st, f, "x");
        // f' = 3x^2 + 2
        let three2 = st.int(3);
        let two2 = st.int(2);
        let two_exp = st.int(2);
        let p2 = st.pow(x, two_exp);
        let t1 = st.mul(vec![three2, p2]);
        let expected = st.add(vec![t1, two2]);
        assert_eq!(df, expected);
    }

    #[test]
    fn diff_product_rule() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let p2 = st.pow(x, two);
        let one = st.int(1);
        let xp1 = st.add(vec![x, one]);
        let f = st.mul(vec![p2, xp1]);
        let df = diff(&mut st, f, "x");
        // d/dx (x^2 * (x+1)) = 2x*(x+1) + x^2*1
        let two2 = st.int(2);
        let term1 = st.mul(vec![two2, x, xp1]);
        let two_exp = st.int(2);
        let term2 = st.pow(x, two_exp);
        let expected = st.add(vec![term1, term2]);
        assert_eq!(df, expected);
    }

    #[test]
    fn diff_trig_exp_log_chain_rule() {
        let mut st = Store::new();
        let x = st.sym("x");

        // d/dx sin(x) = cos(x)
        let sinx = st.func("sin", vec![x]);
        let dsinx = super::diff(&mut st, sinx, "x");
        let cosx = st.func("cos", vec![x]);
        assert_eq!(dsinx, cosx);

        // d/dx cos(x) = -sin(x)
        let cosx2 = st.func("cos", vec![x]);
        let dcosx = super::diff(&mut st, cosx2, "x");
        let neg1 = st.int(-1);
        let sinx2 = st.func("sin", vec![x]);
        let neg_sinx = st.mul(vec![neg1, sinx2]);
        assert_eq!(dcosx, neg_sinx);

        // d/dx exp(x) = exp(x)
        let expx = st.func("exp", vec![x]);
        let dexpx = super::diff(&mut st, expx, "x");
        let expx2 = st.func("exp", vec![x]);
        assert_eq!(dexpx, expx2);

        // d/dx ln(x) = 1/x = x^-1
        let lnx = st.func("ln", vec![x]);
        let dlnx = super::diff(&mut st, lnx, "x");
        let minus_one = st.int(-1);
        let invx = st.pow(x, minus_one);
        assert_eq!(dlnx, invx);

        // Chain rule: d/dx sin(x^2) = cos(x^2) * 2x
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let sin_x2 = st.func("sin", vec![x2]);
        let d_sin_x2 = super::diff(&mut st, sin_x2, "x");
        let two_exp = st.int(2);
        let x2_again = st.pow(x, two_exp);
        let cos_x2 = st.func("cos", vec![x2_again]);
        let two2 = st.int(2);
        let two_x = st.mul(vec![two2, x]);
        let expected = st.mul(vec![cos_x2, two_x]);
        assert_eq!(d_sin_x2, expected);
    }

    #[test]
    fn maclaurin_basic_functions() {
        let mut st = Store::new();
        let x = st.sym("x");
        let order = 6;

        // exp(x)
        let expx = st.func("exp", vec![x]);
        let s_exp = maclaurin(&st, expx, "x", order).expect("exp series");
        assert_eq!(s_exp.coeffs[0], (1, 1));
        assert_eq!(s_exp.coeffs[1], (1, 1));
        assert_eq!(s_exp.coeffs[2], (1, 2));
        assert_eq!(s_exp.coeffs[3], (1, 6));

        // sin(x)
        let sinx = st.func("sin", vec![x]);
        let s_sin = maclaurin(&st, sinx, "x", order).expect("sin series");
        assert_eq!(s_sin.coeffs[0], (0, 1));
        assert_eq!(s_sin.coeffs[1], (1, 1));
        assert_eq!(s_sin.coeffs[2], (0, 1));
        assert_eq!(s_sin.coeffs[3], (-1, 6));

        // cos(x)
        let cosx = st.func("cos", vec![x]);
        let s_cos = maclaurin(&st, cosx, "x", order).expect("cos series");
        assert_eq!(s_cos.coeffs[0], (1, 1));
        assert_eq!(s_cos.coeffs[2], (-1, 2));
        assert_eq!(s_cos.coeffs[4], (1, 24));

        // ln(1 + x)
        let one = st.int(1);
        let one_plus_x = st.add(vec![one, x]);
        let lnx = st.func("ln", vec![one_plus_x]);
        let s_ln = maclaurin(&st, lnx, "x", order).expect("ln series");
        assert_eq!(s_ln.coeffs[0], (0, 1));
        assert_eq!(s_ln.coeffs[1], (1, 1));
        assert_eq!(s_ln.coeffs[2], (-1, 2));
        assert_eq!(s_ln.coeffs[3], (1, 3));
    }

    #[test]
    fn maclaurin_composition_sin_x2() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let sinx2 = st.func("sin", vec![x2]);
        let s = maclaurin(&st, sinx2, "x", 6).expect("series for sin(x^2)");
        assert_eq!(s.coeffs[0], (0, 1));
        assert_eq!(s.coeffs[1], (0, 1));
        assert_eq!(s.coeffs[2], (1, 1));
        assert_eq!(s.coeffs[3], (0, 1));
        assert_eq!(s.coeffs[4], (0, 1));
    }

    #[test]
    fn limit_poly_zero_and_infinity() {
        let mut st = Store::new();
        let x = st.sym("x");
        // f(x) = x^2 + 3x + 2
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let three = st.int(3);
        let three_x = st.mul(vec![three, x]);
        let two2 = st.int(2);
        let f = st.add(vec![x2, three_x, two2]);
        let l0 = limit_poly(&st, f, "x", LimitPoint::Zero);
        assert_eq!(l0, LimitResult::Finite((2, 1)));
        let linf = limit_poly(&st, f, "x", LimitPoint::PosInf);
        assert_eq!(linf, LimitResult::Infinity);

        // g(x) = 5
        let g = st.int(5);
        let g0 = limit_poly(&st, g, "x", LimitPoint::Zero);
        assert_eq!(g0, LimitResult::Finite((5, 1)));
        let ginf = limit_poly(&st, g, "x", LimitPoint::PosInf);
        assert_eq!(ginf, LimitResult::Finite((5, 1)));
    }

    #[test]
    fn diff_x_pow_x() {
        let mut st = Store::new();
        let x = st.sym("x");
        let x_pow_x = st.pow(x, x);
        let d = diff(&mut st, x_pow_x, "x");
        // Expected: x^x * (ln x + 1)
        let lnx = st.func("ln", vec![x]);
        let one = st.int(1);
        let bracket = st.add(vec![lnx, one]);
        let x_pow_x_again = st.pow(x, x);
        let expected = st.mul(vec![x_pow_x_again, bracket]);
        assert_eq!(d, expected);
    }

    #[test]
    fn integrate_power_and_linear_trig_exp() {
        let mut st = Store::new();
        let x = st.sym("x");

        // ∫ x^2 dx = x^3/3
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let ix2 = super::integrate(&mut st, x2, "x").expect("integrable");
        let three = st.int(3);
        let x3 = st.pow(x, three);
        let one_over_three_test = st.rat(1, 3);
        let expected = st.mul(vec![one_over_three_test, x3]);
        assert_eq!(ix2, expected);

        // ∫ 1/x dx = ln x
        let minus_one = st.int(-1);
        let invx = st.pow(x, minus_one);
        let i_invx = super::integrate(&mut st, invx, "x").expect("integrable");
        let lnx = st.func("ln", vec![x]);
        assert_eq!(i_invx, lnx);

        // ∫ exp(3x+1) dx = (1/3) exp(3x+1)
        let three2 = st.int(3);
        let one = st.int(1);
        let three2x = st.mul(vec![three2, x]);
        let inner = st.add(vec![three2x, one]);
        let exp_inner = st.func("exp", vec![inner]);
        let i_exp = super::integrate(&mut st, exp_inner, "x").expect("integrable");
        let three3 = st.int(3);
        let three3x = st.mul(vec![three3, x]);
        let one2 = st.int(1);
        let inner2 = st.add(vec![three3x, one2]);
        let exp_inner2 = st.func("exp", vec![inner2]);
        let one_over_three = st.rat(1, 3);
        let expected_exp = st.mul(vec![one_over_three, exp_inner2]);
        assert_eq!(i_exp, expected_exp);

        // ∫ sin(2x) dx = -1/2 cos(2x)
        let two_a = st.int(2);
        let two_a_x = st.mul(vec![two_a, x]);
        let sin2x = st.func("sin", vec![two_a_x]);
        let i_sin = super::integrate(&mut st, sin2x, "x").expect("integrable");
        let two_b = st.int(2);
        let two_b_x = st.mul(vec![two_b, x]);
        let cos2x = st.func("cos", vec![two_b_x]);
        let minus_half = st.rat(-1, 2);
        let expected_sin = st.mul(vec![minus_half, cos2x]);
        assert_eq!(i_sin, expected_sin);

        // ∫ cos(2x) dx = 1/2 sin(2x)
        let two_c = st.int(2);
        let two_c_x = st.mul(vec![two_c, x]);
        let cos2x2 = st.func("cos", vec![two_c_x]);
        let i_cos = super::integrate(&mut st, cos2x2, "x").expect("integrable");
        let two_d = st.int(2);
        let two_d_x = st.mul(vec![two_d, x]);
        let sin2x2 = st.func("sin", vec![two_d_x]);
        let half = st.rat(1, 2);
        let expected_cos = st.mul(vec![half, sin2x2]);
        assert_eq!(i_cos, expected_cos);
    }

    #[test]
    fn integrate_du_over_u_ln() {
        let mut st = Store::new();
        let x = st.sym("x");
        let one = st.int(1);
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let u = st.add(vec![x2, one]); // u = x^2 + 1
        let du = super::diff(&mut st, u, "x"); // du = 2x
        let minus_one = st.int(-1);
        let u_inv = st.pow(u, minus_one);
        let e = st.mul(vec![du, u_inv]);
        let ie = super::integrate(&mut st, e, "x").expect("integrable");
        let lnu = st.func("ln", vec![u]);
        assert_eq!(ie, lnu);
    }

    #[test]
    fn integrate_rational_via_partial_fractions_and_diff_check() {
        let mut st = Store::new();
        let x = st.sym("x");
        // f(x) = (2x + 3) / (x^2 + 3x + 2)
        let two = st.int(2);
        let two_x = st.mul(vec![two, x]);
        let three = st.int(3);
        let num = st.add(vec![two_x, three]);
        let two2 = st.int(2);
        let three2 = st.int(3);
        let two_exp = st.int(2);
        let x2 = st.pow(x, two_exp);
        let three_x = st.mul(vec![three2, x]);
        let den = st.add(vec![x2, three_x, two2]);
        let minus_one = st.int(-1);
        let inv_den = st.pow(den, minus_one);
        let f = st.mul(vec![num, inv_den]);
        let f_s = simplify::simplify(&mut st, f); // canonicalize integrand

        // Integrate and compare with ln(x+1)+ln(x+2)
        let int = super::integrate(&mut st, f_s, "x").expect("pf integrable");
        let one = st.int(1);
        let xp1 = st.add(vec![x, one]);
        let lnxp1 = st.func("ln", vec![xp1]);
        let two_c = st.int(2);
        let xp2 = st.add(vec![x, two_c]);
        let lnxp2 = st.func("ln", vec![xp2]);
        let expected = st.add(vec![lnxp1, lnxp2]);
        assert_eq!(st.to_string(int), st.to_string(expected));
    }

    #[test]
    fn integrate_rational_another_case() {
        let mut st = Store::new();
        let x = st.sym("x");
        // f(x) = (3x + 5) / (x^2 + 3x + 2) -> 2*ln(x+1) + ln(x+2)
        let three = st.int(3);
        let three_x = st.mul(vec![three, x]);
        let five = st.int(5);
        let num = st.add(vec![three_x, five]);
        let two = st.int(2);
        let three2 = st.int(3);
        let three_x2 = st.mul(vec![three2, x]);
        let two_exp = st.int(2);
        let x2 = st.pow(x, two_exp);
        let den = st.add(vec![x2, three_x2, two]);
        let minus_one = st.int(-1);
        let inv_den = st.pow(den, minus_one);
        let f = st.mul(vec![num, inv_den]);
        let f_s = simplify::simplify(&mut st, f);
        let int = super::integrate(&mut st, f_s, "x").expect("integrable");
        let s = st.to_string(int);
        assert!(s.contains("ln"));
    }

    #[test]
    fn integrate_sin_cos_exp() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let two_x = st.mul(vec![two, x]);
        let sinx = st.func("sin", vec![two_x]);
        let cosx = st.func("cos", vec![two_x]);
        let expx = st.func("exp", vec![two_x]);
        // sin(2x), cos(2x), exp(2x)
        let int_sin = super::integrate(&mut st, sinx, "x").expect("sin integrable");
        assert!(st.to_string(int_sin).contains("cos"));
        let int_cos = super::integrate(&mut st, cosx, "x").expect("cos integrable");
        assert!(st.to_string(int_cos).contains("sin"));
        let int_exp = super::integrate(&mut st, expx, "x").expect("exp integrable");
        assert!(st.to_string(int_exp).contains("exp"));
    }

    #[test]
    fn integrate_ln_product_power() {
        let mut st = Store::new();
        let x = st.sym("x");
        // ln(x) * x^2 now works with integration by parts
        let lnx = st.func("ln", vec![x]);
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let prod = st.mul(vec![lnx, x2]);
        let result = super::integrate(&mut st, prod, "x").expect("integrable with IBP");
        // Verify by differentiation
        let deriv = super::diff(&mut st, result, "x");
        let simplified = simplify::simplify(&mut st, deriv);
        let original_simplified = simplify::simplify(&mut st, prod);
        assert_eq!(st.get(simplified).digest, st.get(original_simplified).digest);
    }

    #[test]
    fn integrate_polynomial_quotient() {
        let mut st = Store::new();
        let x = st.sym("x");
        // Test simpler case: (x + 1) / x = 1 + 1/x
        // ∫ (1 + 1/x) dx = x + ln(x)
        let one = st.int(1);
        let m1 = st.int(-1);
        let inv_x = st.pow(x, m1);
        let sum = st.add(vec![one, inv_x]);
        let int = super::integrate(&mut st, sum, "x").expect("integrable");
        let s = st.to_string(int);
        // Result should contain both x and ln
        assert!(s.contains("ln"));
        assert!(s.contains("x"));
    }

    // ========== Hyperbolic Function Tests (v1.1) ==========

    #[test]
    fn diff_hyperbolic_functions() {
        let mut st = Store::new();
        let x = st.sym("x");

        // d/dx sinh(x) = cosh(x)
        let sinhx = st.func("sinh", vec![x]);
        let d_sinh = diff(&mut st, sinhx, "x");
        let coshx = st.func("cosh", vec![x]);
        assert_eq!(st.to_string(d_sinh), st.to_string(coshx));

        // d/dx cosh(x) = sinh(x)
        let coshx2 = st.func("cosh", vec![x]);
        let d_cosh = diff(&mut st, coshx2, "x");
        let sinhx2 = st.func("sinh", vec![x]);
        assert_eq!(st.to_string(d_cosh), st.to_string(sinhx2));

        // d/dx tanh(x) = 1 - tanh^2(x)
        let tanhx = st.func("tanh", vec![x]);
        let d_tanh = diff(&mut st, tanhx, "x");
        let result = st.to_string(d_tanh);
        assert!(result.contains("tanh"));
    }

    #[test]
    fn integrate_hyperbolic_functions() {
        let mut st = Store::new();
        let x = st.sym("x");

        // ∫ sinh(x) dx = cosh(x)
        let sinhx = st.func("sinh", vec![x]);
        let i_sinh = integrate(&mut st, sinhx, "x").expect("sinh integrable");
        let coshx = st.func("cosh", vec![x]);
        assert_eq!(st.to_string(i_sinh), st.to_string(coshx));

        // ∫ cosh(x) dx = sinh(x)
        let coshx2 = st.func("cosh", vec![x]);
        let i_cosh = integrate(&mut st, coshx2, "x").expect("cosh integrable");
        let sinhx2 = st.func("sinh", vec![x]);
        assert_eq!(st.to_string(i_cosh), st.to_string(sinhx2));

        // ∫ tanh(x) dx = ln(cosh(x))
        let tanhx = st.func("tanh", vec![x]);
        let i_tanh = integrate(&mut st, tanhx, "x").expect("tanh integrable");
        let result = st.to_string(i_tanh);
        assert!(result.contains("ln"));
        assert!(result.contains("cosh"));
    }

    #[test]
    fn integrate_hyperbolic_differential_check() {
        let mut st = Store::new();
        let x = st.sym("x");

        // Verify sinh: d/dx(∫ sinh(x) dx) = sinh(x)
        let sinhx = st.func("sinh", vec![x]);
        let integral = integrate(&mut st, sinhx, "x").expect("sinh");
        let derivative = diff(&mut st, integral, "x");
        let simplified = simplify::simplify(&mut st, derivative);
        assert_eq!(st.get(simplified).digest, st.get(sinhx).digest);

        // Verify cosh: d/dx(∫ cosh(x) dx) = cosh(x)
        let coshx = st.func("cosh", vec![x]);
        let integral2 = integrate(&mut st, coshx, "x").expect("cosh");
        let derivative2 = diff(&mut st, integral2, "x");
        let simplified2 = simplify::simplify(&mut st, derivative2);
        assert_eq!(st.get(simplified2).digest, st.get(coshx).digest);
    }
}
