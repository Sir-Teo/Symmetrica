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

// ---------- Power series (Maclaurin) ----------

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Series {
    // coeffs[k] = coefficient of x^k, each as reduced rational (num, den), den>0
    pub coeffs: Vec<(i64, i64)>,
}

impl Series {
    pub fn zero(order: usize) -> Self {
        Self { coeffs: vec![(0, 1); order + 1] }
    }
    pub fn one(order: usize) -> Self {
        let mut c = vec![(0, 1); order + 1];
        c[0] = (1, 1);
        Self { coeffs: c }
    }
    pub fn const_q(num: i64, den: i64, order: usize) -> Self {
        let mut c = vec![(0, 1); order + 1];
        c[0] = q_norm(num, den);
        Self { coeffs: c }
    }
    pub fn x(order: usize) -> Self {
        let mut c = vec![(0, 1); order + 1];
        if order >= 1 {
            c[1] = (1, 1);
        }
        Self { coeffs: c }
    }
    pub fn truncate(mut self, order: usize) -> Self {
        self.coeffs.truncate(order + 1);
        while self.coeffs.last().is_some_and(|c| c.0 == 0) && self.coeffs.len() > 1 {
            self.coeffs.pop();
        }
        self
    }
    pub fn add(&self, rhs: &Self, order: usize) -> Self {
        let n = order + 1;
        let mut out = vec![(0, 1); n];
        for (k, out_k) in out.iter_mut().enumerate().take(n) {
            let a = self.coeffs.get(k).copied().unwrap_or((0, 1));
            let b = rhs.coeffs.get(k).copied().unwrap_or((0, 1));
            *out_k = q_add(a, b);
        }
        Self { coeffs: out }
    }
    pub fn sub(&self, rhs: &Self, order: usize) -> Self {
        let n = order + 1;
        let mut out = vec![(0, 1); n];
        for (k, out_k) in out.iter_mut().enumerate().take(n) {
            let a = self.coeffs.get(k).copied().unwrap_or((0, 1));
            let b = rhs.coeffs.get(k).copied().unwrap_or((0, 1));
            *out_k = q_sub(a, b);
        }
        Self { coeffs: out }
    }
    pub fn mul(&self, rhs: &Self, order: usize) -> Self {
        let n = order + 1;
        let mut out = vec![(0, 1); n];
        for (i, out_i) in out.iter_mut().enumerate().take(n) {
            for j in 0..=i {
                let a = self.coeffs.get(j).copied().unwrap_or((0, 1));
                let b = rhs.coeffs.get(i - j).copied().unwrap_or((0, 1));
                *out_i = q_add(*out_i, q_mul(a, b));
            }
        }
        Self { coeffs: out }
    }
    // Compose s(inner): requires inner.c0 == 0
    pub fn compose(&self, inner: &Self, order: usize) -> Option<Self> {
        if inner.coeffs.first().copied().unwrap_or((0, 1)) != (0, 1) {
            return None;
        }
        let n = order + 1;
        let mut out = Series::zero(order);
        // p = inner^k
        let mut p = Series::one(order);
        for k in 0..n {
            let a_k = self.coeffs.get(k).copied().unwrap_or((0, 1));
            if a_k.0 != 0 {
                let term = p.scale(a_k, order);
                out = out.add(&term, order);
            }
            p = p.mul(inner, order);
        }
        Some(out)
    }
    pub fn scale(&self, q: (i64, i64), order: usize) -> Self {
        let n = order + 1;
        let mut out = vec![(0, 1); n];
        for (k, out_k) in out.iter_mut().enumerate().take(n) {
            *out_k = q_mul(self.coeffs.get(k).copied().unwrap_or((0, 1)), q);
        }
        Self { coeffs: out }
    }
}

fn gcd_i64(mut a: i64, mut b: i64) -> i64 {
    if a == 0 {
        return b.abs();
    }
    if b == 0 {
        return a.abs();
    }
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a.abs()
}
fn q_norm(n: i64, d: i64) -> (i64, i64) {
    assert!(d != 0, "zero denominator");
    let mut nn = n;
    let mut dd = d;
    if dd < 0 {
        nn = -nn;
        dd = -dd;
    }
    if nn == 0 {
        return (0, 1);
    }
    let g = gcd_i64(nn.abs(), dd);
    (nn / g, dd / g)
}
fn q_add(a: (i64, i64), b: (i64, i64)) -> (i64, i64) {
    q_norm(a.0 * b.1 + b.0 * a.1, a.1 * b.1)
}
fn q_sub(a: (i64, i64), b: (i64, i64)) -> (i64, i64) {
    q_add(a, (-b.0, b.1))
}
fn q_mul(a: (i64, i64), b: (i64, i64)) -> (i64, i64) {
    q_norm(a.0 * b.0, a.1 * b.1)
}
fn q_div(a: (i64, i64), b: (i64, i64)) -> (i64, i64) {
    q_norm(a.0 * b.1, a.1 * b.0)
}

/// Maclaurin series up to `order` (inclusive) for a subset of expressions.
/// Restrictions:
/// - Only supports one variable `var`.
/// - For `exp(u)`, `sin(u)`, `cos(u)`: requires u(0) = 0 for composition.
/// - For `ln(u)`: requires u(0) = 1.
pub fn maclaurin(store: &Store, id: ExprId, var: &str, order: usize) -> Option<Series> {
    match store.get(id).op {
        Op::Integer => {
            if let Payload::Int(k) = store.get(id).payload {
                Some(Series::const_q(k, 1, order))
            } else {
                None
            }
        }
        Op::Rational => {
            if let Payload::Rat(n, d) = store.get(id).payload {
                Some(Series::const_q(n, d, order))
            } else {
                None
            }
        }
        Op::Symbol => match &store.get(id).payload {
            Payload::Sym(s) if s == var => Some(Series::x(order)),
            _ => None,
        },
        Op::Add => {
            let mut acc = Series::zero(order);
            for &c in &store.get(id).children {
                let sc = maclaurin(store, c, var, order)?;
                acc = acc.add(&sc, order);
            }
            Some(acc)
        }
        Op::Mul => {
            let mut prod = Series::one(order);
            for &f in &store.get(id).children {
                let sf = maclaurin(store, f, var, order)?;
                prod = prod.mul(&sf, order);
            }
            Some(prod)
        }
        Op::Pow => {
            let base = store.get(id).children[0];
            let exp = store.get(id).children[1];
            let k = match (&store.get(exp).op, &store.get(exp).payload) {
                (Op::Integer, Payload::Int(m)) if *m >= 0 => *m as usize,
                _ => return None,
            };
            let mut s = Series::one(order);
            let b = maclaurin(store, base, var, order)?;
            for _ in 0..k {
                s = s.mul(&b, order);
            }
            Some(s)
        }
        Op::Function => {
            // Single-arg functions
            let n = store.get(id);
            let fname = match &n.payload {
                Payload::Func(s) => s.as_str(),
                _ => return None,
            };
            if n.children.len() != 1 {
                return None;
            }
            let u = n.children[0];
            let su = maclaurin(store, u, var, order)?;
            match fname {
                "exp" => {
                    // exp(z) = sum z^k/k!, require su.c0=0
                    if su.coeffs.first().copied().unwrap_or((0, 1)) != (0, 1) {
                        return None;
                    }
                    let mut base = Series::zero(order);
                    // fill base coeffs with 1/k!
                    base.coeffs = (0..=order)
                        .map(|k| (1i64, factorial(k as u32) as i64))
                        .map(|(n, d)| q_div((n, 1), (d, 1)))
                        .collect();
                    base.compose(&su, order)
                }
                "sin" => {
                    if su.coeffs.first().copied().unwrap_or((0, 1)) != (0, 1) {
                        return None;
                    }
                    let mut base = Series::zero(order);
                    base.coeffs = vec![(0, 1); order + 1];
                    for m in 0..=order {
                        // sin z = sum (-1)^m z^{2m+1}/(2m+1)!
                        let p = 2 * m + 1;
                        if p > order {
                            break;
                        }
                        let sign = if m % 2 == 0 { 1 } else { -1 };
                        base.coeffs[p] = q_mul((sign, 1), (1, factorial(p as u32) as i64));
                        base.coeffs[p] = q_div((base.coeffs[p].0, base.coeffs[p].1), (1, 1));
                        // above effectively sets  (+/-) 1/p!
                        base.coeffs[p] = (sign, factorial(p as u32) as i64);
                        base.coeffs[p] = q_div((sign, 1), (factorial(p as u32) as i64, 1));
                    }
                    base.compose(&su, order)
                }
                "cos" => {
                    if su.coeffs.first().copied().unwrap_or((0, 1)) != (0, 1) {
                        return None;
                    }
                    let mut base = Series::zero(order);
                    base.coeffs = vec![(0, 1); order + 1];
                    base.coeffs[0] = (1, 1);
                    for m in 0..=order {
                        // cos z = sum (-1)^m z^{2m}/(2m)!
                        let p = 2 * m;
                        if p == 0 {
                            continue;
                        }
                        if p > order {
                            break;
                        }
                        let sign = if m % 2 == 0 { 1 } else { -1 };
                        base.coeffs[p] = q_div((sign, 1), (factorial(p as u32) as i64, 1));
                    }
                    base.compose(&su, order)
                }
                "ln" | "log" => {
                    // ln(1+z) = z - z^2/2 + z^3/3 - ... requires su.c0 == 1
                    if su.coeffs.first().copied().unwrap_or((0, 1)) != (1, 1) {
                        return None;
                    }
                    // v = su - 1
                    let one = Series::one(order);
                    let v = su.sub(&one, order);
                    let mut out = Series::zero(order);
                    // accumulate z^k terms
                    let mut pow = Series::one(order); // z^0
                    for k in 1..=order {
                        pow = if k == 1 { v.clone() } else { pow.mul(&v, order) };
                        let sign = if k % 2 == 1 { 1 } else { -1 };
                        let coeff = q_div((sign, 1), (k as i64, 1));
                        out = out.add(&pow.scale(coeff, order), order);
                    }
                    Some(out)
                }
                _ => None,
            }
        }
    }
}

fn factorial(n: u32) -> u128 {
    (1..=n as u128).product::<u128>().max(1)
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
}
