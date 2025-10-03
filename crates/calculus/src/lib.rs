//! Calculus v1 (minimal): structural differentiation for Add/Mul/Pow.

use expr_core::{ExprId, Op, Payload, Store};
use simplify::simplify;

/// Differentiate expression `id` with respect to symbol `var`.
/// Supported: Add (linearity), Mul (product rule), Pow with integer exponent (chain rule).
pub fn diff(store: &mut Store, id: ExprId, var: &str) -> ExprId {
    match store.get(id).op {
        Op::Integer | Op::Rational => store.int(0),
        Op::Symbol => match &store.get(id).payload {
            Payload::Sym(s) if s == var => store.int(1),
            _ => store.int(0),
        },
        Op::Add => {
            let ids = store.get(id).children.clone();
            let terms = ids.into_iter().map(|t| diff(store, t, var)).collect::<Vec<_>>();
            let sum = store.add(terms);
            simplify(store, sum)
        }
        Op::Mul => {
            // Product rule over n factors: sum_i (f'_i * prod_{j!=i} f_j)
            let fs = store.get(id).children.clone();
            let mut sum_terms: Vec<ExprId> = Vec::new();
            for i in 0..fs.len() {
                let mut factors: Vec<ExprId> = Vec::with_capacity(fs.len());
                for (j, &f) in fs.iter().enumerate() {
                    if i == j {
                        factors.push(diff(store, f, var));
                    } else {
                        factors.push(f);
                    }
                }
                let prod = store.mul(factors);
                sum_terms.push(prod);
            }
            let sum = store.add(sum_terms);
            simplify(store, sum)
        }
        Op::Pow => {
            // d/dx u^n = n * u^(n-1) * u' when n is integer
            let n = store.get(id);
            let base = n.children[0];
            let exp = n.children[1];
            let (exp_op, exp_payload) = {
                let en = store.get(exp);
                (en.op.clone(), en.payload.clone())
            };
            match (exp_op, exp_payload) {
                (Op::Integer, Payload::Int(k)) => {
                    if k == 0 {
                        return store.int(0);
                    }
                    let k_val = store.int(k);
                    let k_minus_1 = store.int(k - 1);
                    let pow_term = store.pow(base, k_minus_1);
                    let dbase = diff(store, base, var);
                    let term = store.mul(vec![k_val, pow_term, dbase]);
                    simplify(store, term)
                }
                _ => store.int(0),
            }
        }
        Op::Function => {
            // Chain rule for common functions with a single argument.
            let (fname, args) = {
                let n = store.get(id);
                let name = match &n.payload {
                    Payload::Func(s) => s.clone(),
                    _ => String::new(),
                };
                (name, n.children.clone())
            };
            if args.len() != 1 {
                return store.int(0);
            }
            let u = args[0];
            let du = diff(store, u, var);
            let out = match fname.as_str() {
                "sin" => {
                    // (sin u)' = cos(u) * u'
                    let cos_u = store.func("cos", vec![u]);
                    store.mul(vec![cos_u, du])
                }
                "cos" => {
                    // (cos u)' = -sin(u) * u'
                    let sin_u = store.func("sin", vec![u]);
                    let neg_one = store.int(-1);
                    store.mul(vec![neg_one, sin_u, du])
                }
                "exp" => {
                    // (exp u)' = exp(u) * u'
                    let exp_u = store.func("exp", vec![u]);
                    store.mul(vec![exp_u, du])
                }
                "ln" | "log" => {
                    // (ln u)' = u' / u = u' * u^{-1}
                    let neg_one = store.int(-1);
                    let inv = store.pow(u, neg_one);
                    store.mul(vec![du, inv])
                }
                _ => store.int(0),
            };
            simplify(store, out)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let neg_one = st.int(-1);
        let sinx2 = st.func("sin", vec![x]);
        let neg_sinx = st.mul(vec![neg_one, sinx2]);
        assert_eq!(dcosx, neg_sinx);

        // d/dx exp(x) = exp(x)
        let expx = st.func("exp", vec![x]);
        let dexpx = super::diff(&mut st, expx, "x");
        let expx2 = st.func("exp", vec![x]);
        assert_eq!(dexpx, expx2);

        // d/dx ln(x) = 1/x = x^-1
        let lnx = st.func("ln", vec![x]);
        let dlnx = super::diff(&mut st, lnx, "x");
        let neg_one = st.int(-1);
        let invx = st.pow(x, neg_one);
        assert_eq!(dlnx, invx);

        // Chain rule: d/dx sin(x^2) = cos(x^2) * 2x
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let sin_x2 = st.func("sin", vec![x2]);
        let d_sin_x2 = super::diff(&mut st, sin_x2, "x");
        let two2 = st.int(2);
        let x2_again = st.pow(x, two2);
        let cos_x2 = st.func("cos", vec![x2_again]);
        let two3 = st.int(2);
        let two_x = st.mul(vec![two3, x]);
        let expected = st.mul(vec![cos_x2, two_x]);
        assert_eq!(d_sin_x2, expected);
    }
}
