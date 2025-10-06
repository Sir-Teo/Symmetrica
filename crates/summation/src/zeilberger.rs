//! Zeilberger's Algorithm for Creative Telescoping
//!
//! Zeilberger's algorithm generates recurrence relations for definite sums.
//! Given a sum S(n) = ∑(k) F(n,k), it finds a recurrence:
//!   a₀(n)S(n) + a₁(n)S(n+1) + ... + aⱼ(n)S(n+j) = 0
//!
//! The algorithm also produces a certificate R(n,k) that proves the recurrence.
//!
//! Reference: Zeilberger, D. (1991). "The method of creative telescoping"

use expr_core::{ExprId, Store};

/// Certificate for Zeilberger's algorithm
///
/// Stores the rational function R(n,k) that certifies the recurrence.
#[derive(Debug, Clone)]
pub struct Certificate {
    /// The rational certificate R(n,k)
    pub rational_cert: ExprId,
    /// Recurrence coefficients [a₀(n), a₁(n), ..., aⱼ(n)]
    pub coefficients: Vec<ExprId>,
    /// Optional inhomogeneous term on the RHS (when a homogeneous recurrence is not found)
    pub inhom_term: Option<ExprId>,
}

/// Generate a recurrence relation using Zeilberger's algorithm
///
/// Returns a Certificate containing the recurrence coefficients and proof.
pub fn zeilberger_recurrence(
    store: &mut Store,
    term: ExprId,
    sum_var: &str,
    param_var: &str,
) -> Option<Certificate> {
    use expr_core::{Op, Payload};

    // Simple case: F(n,k) = r^k where r is independent of k and n
    if let Op::Pow = store.get(term).op {
        let ch = store.get(term).children.clone();
        if ch.len() == 2 {
            let base = ch[0];
            let exp = ch[1];
            if let (Op::Symbol, Payload::Sym(s)) = (&store.get(exp).op, &store.get(exp).payload) {
                if s == sum_var && !depends_on_var(store, base, sum_var) && !depends_on_var(store, base, param_var) {
                    let neg_one = store.int(-1);
                    let one = store.int(1);
                    let a0 = neg_one;
                    let a1 = one;
                    let coeffs = vec![a0, a1];

                    let n_sym = store.sym(param_var);
                    let n_plus_1 = store.add(vec![n_sym, one]);
                    let rhs = store.pow(base, n_plus_1);

                    let zero = store.int(0);
                    return Some(Certificate { rational_cert: zero, coefficients: coeffs, inhom_term: Some(rhs) });
                }
            }
        }
    }

    // Binomial times geometric: F(n,k) = binom(n,k) * r^k
    if let Op::Mul = store.get(term).op {
        let ch = store.get(term).children.clone();
        let mut bin_idx: Option<usize> = None;
        let mut pow_idx: Option<usize> = None;
        for (i, &c) in ch.iter().enumerate() {
            match store.get(c).op {
                Op::Function => {
                    if let Payload::Func(ref fname) = store.get(c).payload {
                        if fname == "binom" || fname == "C" { bin_idx = Some(i); }
                    }
                }
                Op::Pow => { pow_idx = Some(i); }
                _ => {}
            }
        }
        if let (Some(bi), Some(pi)) = (bin_idx, pow_idx) {
            let bin = ch[bi];
            let pow = ch[pi];
            let bch = store.get(bin).children.clone();
            let pch = store.get(pow).children.clone();
            if bch.len() == 2 && pch.len() == 2 {
                let n_arg = bch[0];
                let k_arg = bch[1];
                let r = pch[0];
                let e = pch[1];
                let k_ok = matches!((&store.get(k_arg).op, &store.get(k_arg).payload), (Op::Symbol, Payload::Sym(s)) if s == sum_var);
                let n_ok = matches!((&store.get(n_arg).op, &store.get(n_arg).payload), (Op::Symbol, Payload::Sym(s)) if s == param_var);
                let e_ok = matches!((&store.get(e).op, &store.get(e).payload), (Op::Symbol, Payload::Sym(s)) if s == sum_var);
                if k_ok && n_ok && e_ok && !depends_on_var(store, r, sum_var) && !depends_on_var(store, r, param_var) {
                    let one = store.int(1);
                    let a1 = one;
                    let one_plus_r = store.add(vec![one, r]);
                    let neg_one = store.int(-1);
                    let a0 = store.mul(vec![neg_one, one_plus_r]);
                    let ksym = store.sym(sum_var);
                    let n_sym = store.sym(param_var);
                    let n_plus_1 = store.add(vec![n_sym, one]);
                    let neg_ksym = store.mul(vec![neg_one, ksym]);
                    let denom = store.add(vec![n_plus_1, neg_ksym]);
                    let inv = store.pow(denom, neg_one);
                    let neg_k = store.mul(vec![neg_one, ksym]);
                    let rat_cert = store.mul(vec![neg_k, inv]);
                    return Some(Certificate { rational_cert: rat_cert, coefficients: vec![a0, a1], inhom_term: None });
                }
            }
        }
    }

    // Binomial: F(n,k) = binom(n,k)
    if let Op::Function = store.get(term).op {
        let fname = match &store.get(term).payload { Payload::Func(s) => s.clone(), _ => String::new() };
        if fname == "binom" || fname == "C" {
            let ch = store.get(term).children.clone();
            if ch.len() == 2 {
                let n_arg = ch[0];
                let k_arg = ch[1];
                let k_ok = matches!((&store.get(k_arg).op, &store.get(k_arg).payload), (Op::Symbol, Payload::Sym(s)) if s == sum_var);
                let n_ok = matches!((&store.get(n_arg).op, &store.get(n_arg).payload), (Op::Symbol, Payload::Sym(s)) if s == param_var);
                if k_ok && n_ok {
                    let one = store.int(1);
                    let neg_two = store.int(-2);
                    let a0 = neg_two;
                    let a1 = one;
                    let ksym = store.sym(sum_var);
                    let n_sym = store.sym(param_var);
                    let n_plus_1 = store.add(vec![n_sym, one]);
                    let neg_one = store.int(-1);
                    let neg_ksym = store.mul(vec![neg_one, ksym]);
                    let denom = store.add(vec![n_plus_1, neg_ksym]);
                    let inv = store.pow(denom, neg_one);
                    let neg_k = store.mul(vec![neg_one, ksym]);
                    let rat_cert = store.mul(vec![neg_k, inv]);
                    return Some(Certificate { rational_cert: rat_cert, coefficients: vec![a0, a1], inhom_term: None });
                }
            }
        }
    }

    // Placeholder for general case: not implemented
    None
}

fn depends_on_var(store: &Store, expr: ExprId, var: &str) -> bool {
    use expr_core::{Op, Payload};
    match (&store.get(expr).op, &store.get(expr).payload) {
        (Op::Symbol, Payload::Sym(s)) => s == var,
        (Op::Integer, _) | (Op::Rational, _) => false,
        _ => store.get(expr).children.iter().any(|&c| depends_on_var(store, c, var)),
    }
}

// Note: creative_telescope would implement the telescoping technique
// but is not needed for the current placeholder implementation

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zeilberger_placeholder() {
        let mut st = Store::new();
        let n = st.sym("n");
        let _k = st.sym("k");

        // Binomial coefficient C(n,k)
        // This would generate the recurrence for sum of binomial coefficients
        // For now, just test that it doesn't crash
        let result = zeilberger_recurrence(&mut st, n, "k", "n");
        assert!(result.is_none()); // Not implemented yet
    }

    #[test]
    fn test_zeilberger_geometric() {
        let mut st = Store::new();
        let k = st.sym("k");
        let n_name = "n";
        let two = st.int(2);
        let term = st.pow(two, k); // F(n,k) = 2^k

        let cert = zeilberger_recurrence(&mut st, term, "k", n_name).expect("geo rec");
        assert_eq!(cert.coefficients.len(), 2);
        // inhom term should reference 2^(n+1)
        let rhs_str = st.to_string(cert.inhom_term.expect("inhom"));
        assert!(rhs_str.contains("2"));
    }

    #[test]
    fn test_zeilberger_binomial_sum() {
        let mut st = Store::new();
        let n = st.sym("n");
        let k = st.sym("k");
        let term = st.func("binom", vec![n, k]);

        let cert = zeilberger_recurrence(&mut st, term, "k", "n").expect("binom rec");
        assert_eq!(cert.coefficients.len(), 2);
        let c0 = st.to_string(cert.coefficients[0]);
        let c1 = st.to_string(cert.coefficients[1]);
        assert!(c0.contains("-2"));
        assert!(c1.contains("1"));
        assert!(cert.inhom_term.is_none());
    }
}
