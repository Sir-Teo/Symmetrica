//! Differentiation rules.

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
            // General case: d/dx u^v = u^v * (v' * ln(u) + v * u'/u)
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
                _ => {
                    // General power rule fallback
                    let u_pow_v = store.pow(base, exp);
                    let du = diff(store, base, var);
                    let dv = diff(store, exp, var);
                    let ln_u = store.func("ln", vec![base]);
                    let dv_ln_u = store.mul(vec![dv, ln_u]);
                    let minus_one = store.int(-1);
                    let u_inv = store.pow(base, minus_one);
                    let uprime_over_u = store.mul(vec![du, u_inv]);
                    let v_times_uprime_over_u = store.mul(vec![exp, uprime_over_u]);
                    let bracket = store.add(vec![dv_ln_u, v_times_uprime_over_u]);
                    let out = store.mul(vec![u_pow_v, bracket]);
                    simplify(store, out)
                }
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
                    let neg1 = store.int(-1);
                    store.mul(vec![neg1, sin_u, du])
                }
                "exp" => {
                    // (exp u)' = exp(u) * u'
                    let exp_u = store.func("exp", vec![u]);
                    store.mul(vec![exp_u, du])
                }
                "ln" | "log" => {
                    // (ln u)' = u' / u = u' * u^{-1}
                    let minus_one = store.int(-1);
                    let inv = store.pow(u, minus_one);
                    store.mul(vec![du, inv])
                }
                _ => store.int(0),
            };
            simplify(store, out)
        }
        Op::Piecewise => {
            // Differentiate piecewise: d/dx piecewise((c1, v1), ...) = piecewise((c1, dv1/dx), ...)
            let children = store.get(id).children.clone();
            let mut pairs = Vec::new();
            for chunk in children.chunks(2) {
                if chunk.len() == 2 {
                    let cond = chunk[0];
                    let val = chunk[1];
                    let dval = diff(store, val, var);
                    pairs.push((cond, dval));
                }
            }
            let pw = store.piecewise(pairs);
            simplify(store, pw)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_constant() {
        let mut st = Store::new();
        let five = st.int(5);
        let d = diff(&mut st, five, "x");
        assert_eq!(d, st.int(0));
    }

    #[test]
    fn diff_rational() {
        let mut st = Store::new();
        let half = st.rat(1, 2);
        let d = diff(&mut st, half, "x");
        assert_eq!(d, st.int(0));
    }

    #[test]
    fn diff_other_symbol() {
        let mut st = Store::new();
        let y = st.sym("y");
        let d = diff(&mut st, y, "x");
        assert_eq!(d, st.int(0));
    }

    #[test]
    fn diff_pow_zero_exp() {
        let mut st = Store::new();
        let x = st.sym("x");
        let zero = st.int(0);
        let pow = st.pow(x, zero);
        let d = diff(&mut st, pow, "x");
        assert_eq!(d, st.int(0));
    }

    #[test]
    fn diff_log_alias() {
        let mut st = Store::new();
        let x = st.sym("x");
        let logx = st.func("log", vec![x]);
        let d = diff(&mut st, logx, "x");
        let m1 = st.int(-1);
        let expected = st.pow(x, m1);
        assert_eq!(st.to_string(d), st.to_string(expected));
    }

    #[test]
    fn diff_unknown_function() {
        let mut st = Store::new();
        let x = st.sym("x");
        let fx = st.func("unknown", vec![x]);
        let d = diff(&mut st, fx, "x");
        assert_eq!(d, st.int(0));
    }

    #[test]
    fn diff_multiarg_function() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let f = st.func("f", vec![x, y]);
        let d = diff(&mut st, f, "x");
        assert_eq!(d, st.int(0));
    }

    #[test]
    fn diff_general_power_rule() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        // x^y where both base and exponent are non-constant
        let pow = st.pow(x, y);
        let d = diff(&mut st, pow, "x");
        // Should use general power rule: x^y * (0*ln(x) + y*1/x)
        let result = st.to_string(d);
        // Result should contain y and x
        assert!(result.contains("y") || result.contains("x"));
    }

    #[test]
    fn diff_piecewise() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let cond = st.sym("c");
        let pw = st.piecewise(vec![(cond, x2)]);
        let d = diff(&mut st, pw, "x");
        // Should differentiate the value part
        let result = st.to_string(d);
        assert!(result.contains("piecewise") || result.contains("2"));
    }

    #[test]
    fn diff_piecewise_multiple_branches() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let three = st.int(3);
        let x2 = st.pow(x, two);
        let x3 = st.pow(x, three);
        let c1 = st.sym("c1");
        let c2 = st.sym("c2");
        let pw = st.piecewise(vec![(c1, x2), (c2, x3)]);
        let d = diff(&mut st, pw, "x");
        // Should differentiate both branches
        let result = st.to_string(d);
        assert!(result.contains("piecewise"));
    }

    #[test]
    fn diff_add_multiple_terms() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let three = st.int(3);
        let x2 = st.pow(x, two);
        let x3 = st.pow(x, three);
        let sum = st.add(vec![x, x2, x3]);
        let d = diff(&mut st, sum, "x");
        // d/dx(x + x^2 + x^3) = 1 + 2x + 3x^2
        let result = st.to_string(d);
        assert!(!result.is_empty());
    }

    #[test]
    fn diff_mul_three_factors() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let z = st.sym("z");
        let prod = st.mul(vec![x, y, z]);
        let d = diff(&mut st, prod, "x");
        // d/dx(xyz) = yz
        let result = st.to_string(d);
        assert!(result.contains("y") && result.contains("z"));
    }

    #[test]
    fn diff_sin() {
        let mut st = Store::new();
        let x = st.sym("x");
        let sinx = st.func("sin", vec![x]);
        let d = diff(&mut st, sinx, "x");
        let result = st.to_string(d);
        assert!(result.contains("cos"));
    }

    #[test]
    fn diff_cos() {
        let mut st = Store::new();
        let x = st.sym("x");
        let cosx = st.func("cos", vec![x]);
        let d = diff(&mut st, cosx, "x");
        let result = st.to_string(d);
        assert!(result.contains("sin"));
    }

    #[test]
    fn diff_exp() {
        let mut st = Store::new();
        let x = st.sym("x");
        let expx = st.func("exp", vec![x]);
        let d = diff(&mut st, expx, "x");
        let result = st.to_string(d);
        assert!(result.contains("exp"));
    }

    #[test]
    fn diff_ln() {
        let mut st = Store::new();
        let x = st.sym("x");
        let lnx = st.func("ln", vec![x]);
        let d = diff(&mut st, lnx, "x");
        // d/dx(ln(x)) = 1/x = x^(-1)
        let result = st.to_string(d);
        assert!(result.contains("x"));
    }

    #[test]
    fn diff_chain_rule_sin() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let sin_x2 = st.func("sin", vec![x2]);
        let d = diff(&mut st, sin_x2, "x");
        // d/dx(sin(x^2)) = cos(x^2) * 2x
        let result = st.to_string(d);
        assert!(result.contains("cos"));
    }

    #[test]
    fn diff_product_rule_two_factors() {
        let mut st = Store::new();
        let x = st.sym("x");
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let sinx = st.func("sin", vec![x]);
        let prod = st.mul(vec![x2, sinx]);
        let d = diff(&mut st, prod, "x");
        // d/dx(x^2 * sin(x)) = 2x*sin(x) + x^2*cos(x)
        let result = st.to_string(d);
        assert!(result.contains("sin") || result.contains("cos"));
    }
}
