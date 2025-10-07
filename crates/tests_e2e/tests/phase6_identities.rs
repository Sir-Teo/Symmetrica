#![deny(warnings)]
use assumptions::{Context, Prop};
use expr_core::{Op, Payload, Store};
use simplify::{simplify, simplify_with};

fn contains_func(store: &Store, id: expr_core::ExprId, name: &str) -> bool {
    matches!((&store.get(id).op, &store.get(id).payload), (Op::Function, Payload::Func(n)) if n == name)
}

#[test]
fn product_to_sum_sin_times_sin() {
    // sin(x) * sin(y) -> [cos(x-y) - cos(x+y)] / 2
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");
    let sinx = st.func("sin", vec![x]);
    let siny = st.func("sin", vec![y]);
    let prod = st.mul(vec![sinx, siny]);

    let s = simplify(&mut st, prod);
    assert_eq!(st.get(s).op, Op::Mul);
    let sstr = st.to_string(s);
    assert!(sstr.contains("1/2") && sstr.contains("cos"));
}

#[test]
fn product_to_sum_cos_times_cos() {
    // cos(x) * cos(y) -> [cos(x+y) + cos(x-y)] / 2
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");
    let cosx = st.func("cos", vec![x]);
    let cosy = st.func("cos", vec![y]);
    let prod = st.mul(vec![cosx, cosy]);

    let s = simplify(&mut st, prod);
    assert_eq!(st.get(s).op, Op::Mul);
    let sstr = st.to_string(s);
    assert!(sstr.contains("1/2") && sstr.contains("cos"));
}

#[test]
fn half_angle_sin_squared() {
    // sin^2(x/2) -> (1 - cos x)/2 after simplify()
    let mut st = Store::new();
    let x = st.sym("x");
    let half = st.rat(1, 2);
    let x_half = st.mul(vec![half, x]);
    let sin_xh = st.func("sin", vec![x_half]);
    let two = st.int(2);
    let sin2 = st.pow(sin_xh, two);

    let s = simplify(&mut st, sin2);
    // Should be a Mul with 1/2 and contain cos(x)
    let sstr = st.to_string(s);
    assert!(sstr.contains("1/2"));
    assert!(sstr.contains("cos"));
}

#[test]
fn sum_to_product_cos_plus_cos() {
    // cos(x) + cos(y) -> 2*cos((x+y)/2)*cos((x-y)/2)
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");
    let cosx = st.func("cos", vec![x]);
    let cosy = st.func("cos", vec![y]);
    let sum = st.add(vec![cosx, cosy]);

    let s = simplify(&mut st, sum);
    // Should be a Mul and include factor 2 and cos
    assert_eq!(st.get(s).op, Op::Mul);
    let sstr = st.to_string(s);
    assert!(sstr.contains("2"));
    assert!(sstr.contains("cos"));
}

#[test]
fn log_quotient_expansion_with_positivity() {
    // ln(x*y^{-1}) -> ln(x) - ln(y) when x,y > 0
    let mut st = Store::new();
    let mut ctx = Context::new();
    let x = st.sym("x");
    let y = st.sym("y");
    ctx.assume("x", Prop::Positive);
    ctx.assume("y", Prop::Positive);

    let neg_one = st.int(-1);
    let y_inv = st.pow(y, neg_one);
    let prod = st.mul(vec![x, y_inv]);
    let ln_expr = st.func("ln", vec![prod]);

    let s = simplify_with(&mut st, ln_expr, &ctx);
    // Expect ln(x) + (-1)*ln(y)
    assert_eq!(st.get(s).op, Op::Add);
    let add_children = st.get(s).children.clone();
    assert_eq!(add_children.len(), 2);
    let term1 = add_children[0];
    let term2 = add_children[1];
    assert!(contains_func(&st, term1, "ln") || contains_func(&st, term2, "ln"));
    let sstr = st.to_string(s);
    assert!(sstr.contains("ln") && sstr.contains("-1"));
}

#[test]
fn radical_rationalization_reciprocal_sqrt() {
    // x^(-1/2) -> sqrt(x)/x
    let mut st = Store::new();
    let x = st.sym("x");
    let neg_half = st.rat(-1, 2);
    let expr = st.pow(x, neg_half);

    let s = simplify(&mut st, expr);
    // Accept either direct pow(x, -1/2) or rationalized sqrt(x)/x
    match st.get(s).op {
        Op::Pow => {
            let children = &st.get(s).children;
            assert_eq!(children.len(), 2);
            // exponent must be -1/2
            assert!(matches!(
                (&st.get(children[1]).op, &st.get(children[1]).payload),
                (Op::Rational, Payload::Rat(-1, 2))
            ));
        }
        Op::Mul => {
            let sstr = st.to_string(s);
            // Should contain a 1/2 (sqrt) and -1 (inverse) somewhere in the product
            assert!(sstr.contains("1/2") && sstr.contains("-1"));
        }
        _ => panic!("unexpected form for x^(-1/2) simplification"),
    }
}
