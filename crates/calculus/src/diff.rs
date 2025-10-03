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
    }
}
