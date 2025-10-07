#![deny(warnings)]
use assumptions::{Context, Prop};
use calculus::diff;
use evalf::{eval, EvalContext};
use expr_core::{ExprId, Op, Payload, Store};
use grobner::{buchberger, MonomialOrder};
use simplify::{simplify, simplify_with};
use summation::{sum_arithmetic, sum_geometric};

fn is_func(store: &Store, id: ExprId, name: &str) -> bool {
    matches!(
        (&store.get(id).op, &store.get(id).payload),
        (Op::Function, Payload::Func(n)) if n == name
    )
}

#[test]
fn trig_double_angle_and_pythagorean() {
    let mut st = Store::new();
    let x = st.sym("x");
    let two = st.int(2);

    // 2*sin(x)*cos(x) -> sin(2x)
    let sinx = st.func("sin", vec![x]);
    let cosx = st.func("cos", vec![x]);
    let prod = st.mul(vec![two, sinx, cosx]);
    let s = simplify(&mut st, prod);
    assert!(is_func(&st, s, "sin"));
    // inner should contain 2 and x
    let arg = st.get(s).children[0];
    let arg_str = st.to_string(arg);
    assert!(arg_str.contains("2") && arg_str.contains("x"));

    // sin^2(x) + cos^2(x) -> 1
    let sinx_base = st.func("sin", vec![x]);
    let sinx2 = st.pow(sinx_base, two);
    let cosx_base = st.func("cos", vec![x]);
    let cosx2 = st.pow(cosx_base, two);
    let sum = st.add(vec![sinx2, cosx2]);
    let res = simplify(&mut st, sum);
    assert!(matches!((&st.get(res).op, &st.get(res).payload), (Op::Integer, Payload::Int(1))));
}

#[test]
fn logs_expand_with_positivity_and_contract_back() {
    let mut st = Store::new();
    let mut ctx = Context::new();
    let x = st.sym("x");
    let y = st.sym("y");
    ctx.assume("x", Prop::Positive);
    ctx.assume("y", Prop::Positive);

    let prod = st.mul(vec![x, y]);
    let ln_prod = st.func("ln", vec![prod]);
    let expanded = simplify_with(&mut st, ln_prod, &ctx);
    // Should be ln(x) + ln(y)
    assert_eq!(st.get(expanded).op, Op::Add);

    // Now contract ln(x) + ln(y) -> ln(x*y)
    let ln_x = st.func("ln", vec![x]);
    let ln_y = st.func("ln", vec![y]);
    let sum = st.add(vec![ln_x, ln_y]);
    let contracted = simplify(&mut st, sum); // pipeline includes contraction
    assert!(is_func(&st, contracted, "ln"));
}

#[test]
fn radical_simplify_perfect_square_rational() {
    let mut st = Store::new();
    let four_ninths = st.rat(4, 9);
    let half = st.rat(1, 2);
    let sqrt_expr = st.pow(four_ninths, half);
    let s = simplify(&mut st, sqrt_expr);
    assert!(matches!((&st.get(s).op, &st.get(s).payload), (Op::Rational, Payload::Rat(2, 3))));
}

#[test]
fn special_functions_eval_and_diff() {
    let mut st = Store::new();
    let x = st.sym("x");

    // erf(1/2) ~ 0.5205
    let half = st.rat(1, 2);
    let erf_half = st.func("erf", vec![half]);
    let ctx = EvalContext::new();
    let val = eval(&st, erf_half, &ctx).unwrap();
    assert!((val - 0.5205).abs() < 1e-3);

    // d/dx erf(x) contains exp(-x^2)
    let erf_x = st.func("erf", vec![x]);
    let d = diff(&mut st, erf_x, "x");
    let d_str = st.to_string(d);
    assert!(d_str.contains("exp"));

    // Gamma(1) = 1, Gamma(1/2) = sqrt(pi)
    let one = st.int(1);
    let gamma1 = st.func("Gamma", vec![one]);
    let val1 = eval(&st, gamma1, &ctx).unwrap();
    assert_eq!(val1, 1.0);

    let gamma_half = st.func("Gamma", vec![half]);
    let valh = eval(&st, gamma_half, &ctx).unwrap();
    assert!((valh - std::f64::consts::PI.sqrt()).abs() < 1e-10);

    // Ei(0) is a domain error in our placeholder
    let zero = st.int(0);
    let ei0 = st.func("Ei", vec![zero]);
    assert!(eval(&st, ei0, &ctx).is_err());
}

#[test]
fn summation_arithmetic_and_geometric() {
    let mut st = Store::new();
    let a = st.int(5);
    let d = st.int(3);
    let n = st.sym("n");

    let arith = sum_arithmetic(&mut st, a, d, n).unwrap();
    let s1 = st.to_string(arith);
    assert!(s1.contains("n"));

    let r = st.int(2);
    let geom = sum_geometric(&mut st, a, r, n).unwrap();
    let s2 = st.to_string(geom);
    assert!(s2.contains("1/2") || s2.contains("2^"));
}

#[test]
fn grobner_buchberger_smoke() {
    let mut st = Store::new();
    let x = st.sym("x");
    let basis = buchberger(&mut st, vec![x], vec!["x".to_string()], MonomialOrder::Lex);
    assert_eq!(basis.len(), 1);

    let empty = buchberger(&mut st, vec![], vec![], MonomialOrder::Lex);
    assert!(empty.is_empty());
}
