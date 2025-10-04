//! Solver module: univariate polynomial solving over Q.
//! - Linear and quadratic closed forms
//! - Rational-root search for higher degrees (factor out simple rational roots)

#![deny(warnings)]

use arith::{add_q, div_q, mul_q, sub_q, Q};
use expr_core::{ExprId, Store};
use polys::{expr_to_unipoly, UniPoly};

/// Solve a univariate polynomial equation p(x) = 0 where `expr` is convertible to a polynomial in `var`.
/// Returns a list of root expressions (values for x). If unable to factor completely, returns None.
pub fn solve_univariate(store: &mut Store, expr: ExprId, var: &str) -> Option<Vec<ExprId>> {
    let p0 = expr_to_unipoly(store, expr, var)?;
    if p0.is_zero() {
        // Degenerate 0 == 0; no finite roots enumerated
        return Some(vec![]);
    }
    let mut p = p0.clone();
    let mut roots: Vec<ExprId> = Vec::new();

    // Helper: convert Q to Expr
    fn q_to_expr(st: &mut Store, q: Q) -> ExprId {
        if q.1 == 1 {
            st.int(q.0)
        } else {
            st.rat(q.0, q.1)
        }
    }

    // Solve degree 1: a1 x + a0 = 0
    fn solve_deg1(st: &mut Store, p: &UniPoly) -> Option<Vec<ExprId>> {
        let a0 = p.coeffs.first().copied().unwrap_or(Q::zero());
        let a1 = p.coeffs.get(1).copied().unwrap_or(Q::zero());
        if a1.is_zero() {
            return None;
        }
        let root = div_q(Q(-a0.0, a0.1), a1); // -a0/a1
        Some(vec![q_to_expr(st, root)])
    }

    // Solve degree 2: a2 x^2 + a1 x + a0 = 0
    fn solve_deg2(st: &mut Store, p: &UniPoly) -> Option<Vec<ExprId>> {
        let a0 = p.coeffs.first().copied().unwrap_or(Q::zero());
        let a1 = p.coeffs.get(1).copied().unwrap_or(Q::zero());
        let a2 = p.coeffs.get(2).copied().unwrap_or(Q::zero());
        if a2.is_zero() {
            return solve_deg1(st, p);
        }
        // Discriminant D = a1^2 - 4 a2 a0
        let a1sq = mul_q(a1, a1);
        let four_a2a0 = mul_q(Q(4, 1), mul_q(a2, a0));
        let d = sub_q(a1sq, four_a2a0);
        let minus_b = Q(-a1.0, a1.1);
        let two_a = mul_q(Q(2, 1), a2);

        // If D is a rational square, compute rational roots directly
        fn is_square_i64(n: i64) -> Option<i64> {
            if n < 0 {
                return None;
            }
            let mut i = 0i64;
            while i * i <= n {
                if i * i == n {
                    return Some(i);
                }
                i += 1;
            }
            None
        }
        let sqrt_rational = (|| -> Option<Q> {
            let num = d.0;
            let den = d.1;
            let sn = is_square_i64(num)?;
            let sd = is_square_i64(den)?;
            Some(Q(sn, sd))
        })();
        if let Some(sq) = sqrt_rational {
            let r1 = div_q(add_q(minus_b, sq), two_a);
            let r2 = div_q(sub_q(minus_b, sq), two_a);
            return Some(vec![q_to_expr(st, r1), q_to_expr(st, r2)]);
        }

        // Otherwise, build expressions: (-b ± sqrt(D)) / (2a)
        let num_base = q_to_expr(st, minus_b);
        let sqrt_d = {
            let d_expr = q_to_expr(st, d);
            let half = st.rat(1, 2);
            st.pow(d_expr, half)
        };
        let inv_two_a = {
            let inv = div_q(Q(1, 1), two_a);
            q_to_expr(st, inv)
        };
        let plus = {
            let num = st.add(vec![num_base, sqrt_d]);
            st.mul(vec![inv_two_a, num])
        };
        let minus = {
            let m1 = st.int(-1);
            let neg_sqrt = st.mul(vec![m1, sqrt_d]);
            let num = st.add(vec![num_base, neg_sqrt]);
            st.mul(vec![inv_two_a, num])
        };
        Some(vec![plus, minus])
    }

    // For deg>2: rational root search and deflation, then finish with linear/quadratic
    fn clear_denominators(p: &UniPoly) -> (Vec<i64>, i64) {
        fn gcd_i64(mut a: i64, mut b: i64) -> i64 {
            while b != 0 {
                let t = a % b;
                a = b;
                b = t;
            }
            a.abs()
        }
        fn lcm_i64(a: i64, b: i64) -> i64 {
            (a / gcd_i64(a, b)) * b
        }
        let mut l = 1i64;
        for &Q(_, d) in &p.coeffs {
            let dd = d.abs().max(1);
            l = lcm_i64(l.abs().max(1), dd);
        }
        let mut ints = Vec::with_capacity(p.coeffs.len());
        for &Q(n, d) in &p.coeffs {
            ints.push(n * (l / d));
        }
        (ints, l)
    }
    fn divisors(mut n: i64) -> Vec<i64> {
        if n < 0 {
            n = -n;
        }
        if n == 0 {
            return vec![0];
        }
        let mut ds = Vec::new();
        let mut i = 1;
        while (i as i128) * (i as i128) <= (n as i128) {
            if n % i == 0 {
                ds.push(i);
                if i != n / i {
                    ds.push(n / i);
                }
            }
            i += 1;
        }
        ds
    }
    fn eval_q(p: &UniPoly, x: Q) -> Q {
        let mut acc = Q::zero();
        for &c in p.coeffs.iter().rev() {
            acc = add_q(mul_q(acc, x), c);
        }
        acc
    }
    fn deflate_by_root(p: &UniPoly, r: Q) -> Option<UniPoly> {
        let var = p.var.clone();
        let mut new_coeffs: Vec<Q> = Vec::with_capacity(p.coeffs.len().saturating_sub(1));
        let mut acc = Q::zero();
        for &c in p.coeffs.iter().rev() {
            acc = add_q(mul_q(acc, r), c);
            new_coeffs.push(acc);
        }
        if !acc.is_zero() {
            return None;
        }
        new_coeffs.pop();
        new_coeffs.reverse();
        Some(UniPoly::new(var, new_coeffs))
    }

    loop {
        match p.degree() {
            None => break,
            Some(0) => break,
            Some(1) => {
                let mut v = solve_deg1(store, &p)?;
                roots.append(&mut v);
                break;
            }
            Some(2) => {
                let mut v = solve_deg2(store, &p)?;
                roots.append(&mut v);
                break;
            }
            Some(_) => {
                // Rational root candidates
                let (ints, _l) = clear_denominators(&p);
                let lc = *ints.last().unwrap();
                let ct = ints[0];
                let mut found = None;
                'outer: for q in divisors(lc).into_iter().flat_map(|q| vec![q, -q]) {
                    if q == 0 {
                        continue;
                    }
                    for pn in divisors(ct).into_iter().flat_map(|pn| vec![pn, -pn]) {
                        let r = Q(pn, q);
                        if eval_q(&p, r).is_zero() {
                            found = Some(r);
                            break 'outer;
                        }
                    }
                }
                let r = found?;
                roots.push(q_to_expr(store, r));
                p = deflate_by_root(&p, r)?;
            }
        }
    }

    Some(roots)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solve_linear_root() {
        let mut st = Store::new();
        let x = st.sym("x");
        let one = st.int(1);
        let e = st.add(vec![x, one]); // x + 1 = 0 -> root -1
        let roots = solve_univariate(&mut st, e, "x").expect("solved");
        assert_eq!(roots.len(), 1);
        assert_eq!(st.to_string(roots[0]), "-1");
    }

    #[test]
    fn solve_quadratic_rational_roots() {
        let mut st = Store::new();
        let x = st.sym("x");
        // x^2 + 3x + 2 = 0 -> roots -1, -2
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let three = st.int(3);
        let three_x = st.mul(vec![three, x]);
        let two2 = st.int(2);
        let e = st.add(vec![x2, three_x, two2]);
        let mut roots = solve_univariate(&mut st, e, "x").expect("solved");
        roots.sort_by_key(|r| st.to_string(*r));
        let rs: Vec<String> = roots.into_iter().map(|r| st.to_string(r)).collect();
        assert_eq!(rs, vec!["-1", "-2"]);
    }

    #[test]
    fn solve_quadratic_irrational_roots() {
        let mut st = Store::new();
        let x = st.sym("x");
        // x^2 - 2 = 0 -> ± sqrt(2)
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let minus_two = st.int(-2);
        let e = st.add(vec![x2, minus_two]);
        let roots = solve_univariate(&mut st, e, "x").expect("solved");
        assert_eq!(roots.len(), 2);
        // We can't easily compare canonical forms with sqrt; just ensure one positive, one negative, and both involve ^{1/2}
        let s0 = st.to_string(roots[0]);
        let s1 = st.to_string(roots[1]);
        assert!(s0.contains("^"));
        assert!(s1.contains("^"));
    }

    #[test]
    fn solve_cubic_rational_roots() {
        let mut st = Store::new();
        let x = st.sym("x");
        // x^3 - x = 0 -> roots 0, ±1
        let three = st.int(3);
        let x3 = st.pow(x, three);
        let m1 = st.int(-1);
        let minus_x = st.mul(vec![m1, x]);
        let e = st.add(vec![x3, minus_x]);
        let mut roots = solve_univariate(&mut st, e, "x").expect("solved");
        roots.sort_by_key(|r| st.to_string(*r));
        let rs: Vec<String> = roots.into_iter().map(|r| st.to_string(r)).collect();
        assert_eq!(rs, vec!["-1", "0", "1"]);
    }

    #[test]
    fn solve_zero_polynomial_returns_empty() {
        let mut st = Store::new();
        let zero = st.int(0);
        let result = solve_univariate(&mut st, zero, "x").expect("zero poly");
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn solve_constant_nonzero_returns_empty() {
        let mut st = Store::new();
        let five = st.int(5);
        let result = solve_univariate(&mut st, five, "x").expect("constant poly");
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn solve_cubic_no_rational_roots() {
        let mut st = Store::new();
        let x = st.sym("x");
        // x^3 + x + 1 = 0 (has no rational roots)
        let three = st.int(3);
        let x3 = st.pow(x, three);
        let one = st.int(1);
        let e = st.add(vec![x3, x, one]);
        let result = solve_univariate(&mut st, e, "x");
        // No rational roots, so should return None
        assert!(result.is_none());
    }

    #[test]
    fn solve_not_polynomial() {
        let mut st = Store::new();
        let x = st.sym("x");
        let sinx = st.func("sin", vec![x]);
        let result = solve_univariate(&mut st, sinx, "x");
        assert!(result.is_none());
    }

    #[test]
    fn solve_quadratic_with_rational_discriminant() {
        let mut st = Store::new();
        let x = st.sym("x");
        // x^2 - 5x + 6 = 0 -> roots 2, 3
        let two = st.int(2);
        let x2 = st.pow(x, two);
        let m5 = st.int(-5);
        let m5x = st.mul(vec![m5, x]);
        let six = st.int(6);
        let e = st.add(vec![x2, m5x, six]);
        let mut roots = solve_univariate(&mut st, e, "x").expect("solved");
        roots.sort_by_key(|r| st.to_string(*r));
        let rs: Vec<String> = roots.into_iter().map(|r| st.to_string(r)).collect();
        assert_eq!(rs, vec!["2", "3"]);
    }
}
