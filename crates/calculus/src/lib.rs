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

// ---------- Limits (heuristic, polynomials only) ----------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LimitPoint {
    Zero,
    PosInf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LimitResult {
    Finite((i64, i64)),
    Infinity,
    Indeterminate,
    Unsupported,
}

/// Try to compute limit for polynomial-like expressions in `var`.
/// Supported:
/// - point = Zero: returns constant term c0 as rational.
/// - point = PosInf: if degree==0 returns constant; if degree>0 returns Infinity.
pub fn limit_poly(store: &Store, id: ExprId, var: &str, point: LimitPoint) -> LimitResult {
    fn const_term(store: &Store, id: ExprId, var: &str) -> Option<(i64, i64)> {
        match (&store.get(id).op, &store.get(id).payload) {
            (Op::Integer, Payload::Int(k)) => Some(((*k), 1)),
            (Op::Rational, Payload::Rat(n, d)) => Some(((*n), (*d))),
            (Op::Symbol, Payload::Sym(s)) => {
                if s == var {
                    Some((0, 1))
                } else {
                    None
                }
            }
            (Op::Add, _) => {
                let mut acc = (0, 1);
                for &c in &store.get(id).children {
                    let ct = const_term(store, c, var)?;
                    acc = q_add(acc, ct);
                }
                Some(acc)
            }
            (Op::Mul, _) => {
                let mut acc = (1, 1);
                for &f in &store.get(id).children {
                    let ct = const_term(store, f, var)?;
                    acc = q_mul(acc, ct);
                }
                Some(acc)
            }
            (Op::Pow, _) => {
                let n = store.get(id);
                let base = n.children[0];
                let exp = n.children[1];
                match (&store.get(exp).op, &store.get(exp).payload) {
                    (Op::Integer, Payload::Int(k)) if *k >= 0 => {
                        let ct = const_term(store, base, var)?;
                        // ct^k
                        let mut acc = (1, 1);
                        for _ in 0..(*k as usize) {
                            acc = q_mul(acc, ct);
                        }
                        Some(acc)
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn degree(store: &Store, id: ExprId, var: &str) -> Option<isize> {
        match (&store.get(id).op, &store.get(id).payload) {
            (Op::Integer, Payload::Int(k)) => Some(if *k == 0 { -1 } else { 0 }),
            (Op::Rational, Payload::Rat(n, _)) => Some(if *n == 0 { -1 } else { 0 }),
            (Op::Symbol, Payload::Sym(s)) => {
                if s == var {
                    Some(1)
                } else {
                    None
                }
            }
            (Op::Add, _) => {
                let mut deg = -1;
                for &c in &store.get(id).children {
                    let cd = degree(store, c, var)?;
                    if cd > deg {
                        deg = cd;
                    }
                }
                Some(deg)
            }
            (Op::Mul, _) => {
                let mut deg = 0isize;
                for &f in &store.get(id).children {
                    let fd = degree(store, f, var)?;
                    if fd < 0 {
                        return Some(-1);
                    }
                    deg += fd;
                }
                Some(deg)
            }
            (Op::Pow, _) => {
                let n = store.get(id);
                let base = n.children[0];
                let exp = n.children[1];
                match (&store.get(exp).op, &store.get(exp).payload) {
                    (Op::Integer, Payload::Int(k)) if *k >= 0 => {
                        let bd = degree(store, base, var)?;
                        if bd < 0 {
                            Some(-1)
                        } else {
                            Some(bd * (*k as isize))
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    match point {
        LimitPoint::Zero => match const_term(store, id, var) {
            Some(ct) => LimitResult::Finite(q_norm(ct.0, ct.1)),
            None => LimitResult::Unsupported,
        },
        LimitPoint::PosInf => match degree(store, id, var) {
            Some(d) if d < 0 => LimitResult::Finite((0, 1)),
            Some(0) => match const_term(store, id, var) {
                Some(ct) => LimitResult::Finite(q_norm(ct.0, ct.1)),
                None => LimitResult::Unsupported,
            },
            Some(_) => LimitResult::Infinity,
            None => LimitResult::Unsupported,
        },
    }
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

// ---------- Integration (v1, conservative) ----------

/// Try to integrate expression w.r.t. `var`. Returns None if rule not supported.
pub fn integrate(store: &mut Store, id: ExprId, var: &str) -> Option<ExprId> {
    // helper: does expr depend on var?
    fn depends_on_var(st: &Store, id: ExprId, var: &str) -> bool {
        match (&st.get(id).op, &st.get(id).payload) {
            (Op::Symbol, Payload::Sym(s)) => s == var,
            (Op::Integer, _) | (Op::Rational, _) => false,
            _ => st.get(id).children.iter().any(|&c| depends_on_var(st, c, var)),
        }
    }
    // helper: extract numeric coefficient and rest from a product
    fn split_coeff_mul(st: &mut Store, id: ExprId) -> ((i64, i64), ExprId) {
        match (&st.get(id).op, &st.get(id).payload) {
            (Op::Integer, Payload::Int(k)) => ((*k, 1), st.int(1)),
            (Op::Rational, Payload::Rat(n, d)) => ((*n, *d), st.int(1)),
            (Op::Mul, _) => {
                let mut coeff = (1i64, 1i64);
                let mut rest: Vec<ExprId> = Vec::new();
                let children = st.get(id).children.clone();
                for f in children {
                    match (&st.get(f).op, &st.get(f).payload) {
                        (Op::Integer, Payload::Int(k)) => {
                            coeff = q_mul(coeff, (*k, 1));
                        }
                        (Op::Rational, Payload::Rat(n, d)) => {
                            coeff = q_mul(coeff, (*n, *d));
                        }
                        _ => rest.push(f),
                    }
                }
                let rest_id = if rest.is_empty() { st.int(1) } else { st.mul(rest) };
                (coeff, rest_id)
            }
            _ => ((1, 1), id),
        }
    }
    // helper: build coeff * expr
    fn with_coeff(st: &mut Store, coeff: (i64, i64), expr: ExprId) -> ExprId {
        if coeff == (1, 1) {
            return expr;
        }
        let c = st.rat(coeff.0, coeff.1);
        let prod = st.mul(vec![c, expr]);
        simplify(st, prod)
    }

    match store.get(id).op {
        Op::Integer => {
            if let Payload::Int(k) = store.get(id).payload {
                let x = store.sym(var);
                let ck = store.int(k);
                Some(store.mul(vec![ck, x]))
            } else {
                None
            }
        }
        Op::Rational => {
            if let Payload::Rat(n, d) = store.get(id).payload {
                let x = store.sym(var);
                let c = store.rat(n, d);
                Some(store.mul(vec![c, x]))
            } else {
                None
            }
        }
        Op::Symbol => match &store.get(id).payload {
            Payload::Sym(s) if s == var => {
                // ∫ x dx = x^2/2
                let two = store.int(2);
                let x = store.sym(var);
                let x2 = store.pow(x, two);
                let half = store.rat(1, 2);
                Some(store.mul(vec![half, x2]))
            }
            _ => {
                // treat as constant symbol c: ∫ c dx = c*x
                let x = store.sym(var);
                Some(store.mul(vec![id, x]))
            }
        },
        Op::Add => {
            let mut terms: Vec<ExprId> = Vec::new();
            for &t in &store.get(id).children.clone() {
                let it = integrate(store, t, var)?;
                terms.push(it);
            }
            let sum = store.add(terms);
            Some(simplify(store, sum))
        }
        Op::Mul => {
            // factor out numeric coefficient
            let (coeff, rest) = split_coeff_mul(store, id);
            // f'/f pattern: look for a factor u^{-1} and check remaining equals u' up to numeric factor
            if store.get(rest).op == Op::Mul {
                let factors = store.get(rest).children.clone();
                // iterate all positions to find u^{-1}
                for (idx, &f) in factors.iter().enumerate() {
                    if store.get(f).op == Op::Pow {
                        let u_node = store.get(f);
                        if u_node.children.len() == 2 {
                            let u = u_node.children[0];
                            let e = u_node.children[1];
                            if matches!(
                                (&store.get(e).op, &store.get(e).payload),
                                (Op::Integer, Payload::Int(-1))
                            ) {
                                // build product of remaining factors
                                let mut others: Vec<ExprId> =
                                    Vec::with_capacity(factors.len().saturating_sub(1));
                                for (j, &g) in factors.iter().enumerate() {
                                    if j != idx {
                                        others.push(g);
                                    }
                                }
                                let others_id = if others.is_empty() {
                                    store.int(1)
                                } else {
                                    store.mul(others)
                                };
                                // compare to u' up to numeric coefficient
                                let du = diff(store, u, var);
                                let (coeff_o, rest_o) = split_coeff_mul(store, others_id);
                                let (coeff_d, rest_d) = split_coeff_mul(store, du);
                                if rest_o == rest_d {
                                    let scale = q_div(coeff_o, coeff_d);
                                    let total = q_mul(coeff, scale);
                                    let ln_u = store.func("ln", vec![u]);
                                    return Some(with_coeff(store, total, ln_u));
                                }
                            }
                        }
                    }
                }
            }
            // constant times integrable function, only if we truly factored something out
            if coeff != (1, 1) {
                let ir = integrate(store, rest, var)?;
                Some(with_coeff(store, coeff, ir))
            } else if rest != id {
                let ir = integrate(store, rest, var)?;
                Some(ir)
            } else {
                None
            }
        }
        Op::Pow => {
            // ∫ x^n dx rule
            let base = store.get(id).children[0];
            let exp = store.get(id).children[1];
            if let (Op::Symbol, Payload::Sym(s)) = (&store.get(base).op, &store.get(base).payload) {
                if s == var {
                    let k_value = match (&store.get(exp).op, &store.get(exp).payload) {
                        (Op::Integer, Payload::Int(k)) => Some(*k),
                        _ => None,
                    };
                    if let Some(k) = k_value {
                        if k == -1 {
                            // ∫ x^-1 dx = ln x
                            let ln = store.func("ln", vec![base]);
                            return Some(ln);
                        } else {
                            // x^(k+1)/(k+1)
                            let k1 = store.int(k + 1);
                            let xkp1 = store.pow(base, k1);
                            let coeff = q_div((1, 1), (k + 1, 1));
                            return Some(with_coeff(store, coeff, xkp1));
                        }
                    }
                }
            }
            None
        }
        Op::Function => {
            // exp(ax+b), sin(ax+b), cos(ax+b)
            let (fname, u) = {
                let n = store.get(id);
                let name = match &n.payload {
                    Payload::Func(s) => s.clone(),
                    _ => return None,
                };
                if n.children.len() != 1 {
                    return None;
                }
                (name, n.children[0])
            };
            // check du is constant
            let du = diff(store, u, var);
            let a = match (&store.get(du).op, &store.get(du).payload) {
                (Op::Integer, Payload::Int(k)) => (*k, 1),
                (Op::Rational, Payload::Rat(n, d)) => (*n, *d),
                _ => {
                    // if independent of var entirely, treat whole function as constant
                    if !depends_on_var(store, id, var) {
                        let x = store.sym(var);
                        return Some(store.mul(vec![id, x]));
                    }
                    return None;
                }
            };
            if a == (0, 1) {
                return None;
            }
            let inv_a = q_div((1, 1), a);
            let res = match fname.as_str() {
                "exp" => id,
                "sin" => {
                    let c = store.func("cos", vec![u]);
                    let neg1 = store.int(-1);
                    store.mul(vec![neg1, c])
                }
                "cos" => store.func("sin", vec![u]),
                _ => return None,
            };
            Some(with_coeff(store, inv_a, res))
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
}
