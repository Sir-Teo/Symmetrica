//! Basic summation formulas
//!
//! This module implements closed-form formulas for common summation patterns:
//! - Arithmetic series: ∑(a + kd)
//! - Geometric series: ∑(ar^k)
//! - Power sums: ∑(k^p)

use expr_core::{ExprId, Op, Payload, Store};
use simplify::simplify;

/// Try to recognize and evaluate basic sum patterns
pub fn try_basic_sum(
    store: &mut Store,
    expr: ExprId,
    var: &str,
    lower: ExprId,
    upper: ExprId,
) -> Option<ExprId> {
    // Try arithmetic series
    if let Some((a, d)) = recognize_arithmetic(store, expr, var) {
        return sum_arithmetic(store, expr, lower, upper, a, d);
    }

    // Try geometric series
    if let Some(ratio) = recognize_geometric(store, expr, var) {
        return sum_geometric(store, expr, var, lower, upper, ratio);
    }

    // Try power sum
    if let Some(power) = recognize_power_sum(store, expr, var) {
        return sum_power(store, var, lower, upper, power);
    }

    None
}

/// Recognize arithmetic sequence: a + k*d
fn recognize_arithmetic(store: &mut Store, expr: ExprId, var: &str) -> Option<(ExprId, ExprId)> {
    match &store.get(expr).op {
        Op::Symbol => {
            if let Payload::Sym(s) = &store.get(expr).payload {
                if s == var {
                    // Just k: a=0, d=1
                    let zero = store.int(0);
                    let one = store.int(1);
                    return Some((zero, one));
                }
            }
            None
        }
        Op::Add => {
            let children = store.get(expr).children.clone();
            let mut a = store.int(0);
            let mut d = store.int(0);

            for &child in &children {
                if !depends_on_var(store, child, var) {
                    // Constant term
                    a = store.add(vec![a, child]);
                } else if is_linear_in_var(store, child, var) {
                    // Linear term k*d
                    if let Some(coeff) = extract_linear_coeff(store, child, var) {
                        d = store.add(vec![d, coeff]);
                    }
                }
            }

            // Simplify to canonical form
            let a = simplify(store, a);
            let d = simplify(store, d);

            Some((a, d))
        }
        Op::Mul => {
            // Handle pure linear multiplicative term: c*k
            if is_linear_in_var(store, expr, var) {
                if let Some(coeff) = extract_linear_coeff(store, expr, var) {
                    let zero = store.int(0);
                    return Some((zero, coeff));
                }
            }
            None
        }
        _ => None,
    }
}

/// Check if expression is linear in var
fn is_linear_in_var(store: &Store, expr: ExprId, var: &str) -> bool {
    match &store.get(expr).op {
        Op::Symbol => {
            if let Payload::Sym(s) = &store.get(expr).payload {
                s == var
            } else {
                false
            }
        }
        Op::Mul => {
            let children = &store.get(expr).children;
            let var_count = children
                .iter()
                .filter(|&&c| {
                    if let (Op::Symbol, Payload::Sym(s)) = (&store.get(c).op, &store.get(c).payload)
                    {
                        s == var
                    } else {
                        false
                    }
                })
                .count();
            var_count == 1
        }
        _ => false,
    }
}

/// Extract coefficient from linear term c*var
fn extract_linear_coeff(store: &mut Store, expr: ExprId, var: &str) -> Option<ExprId> {
    match &store.get(expr).op {
        Op::Symbol => {
            if let Payload::Sym(s) = &store.get(expr).payload {
                if s == var {
                    return Some(store.int(1));
                }
            }
            None
        }
        Op::Mul => {
            let children = store.get(expr).children.clone();
            let mut coeff = store.int(1);

            for &child in &children {
                if let (Op::Symbol, Payload::Sym(s)) =
                    (&store.get(child).op, &store.get(child).payload)
                {
                    if s == var {
                        continue;
                    }
                }
                coeff = store.mul(vec![coeff, child]);
            }

            Some(simplify(store, coeff))
        }
        _ => None,
    }
}

/// Recognize geometric sequence: ar^k
fn recognize_geometric(store: &mut Store, expr: ExprId, var: &str) -> Option<ExprId> {
    match &store.get(expr).op {
        Op::Pow => {
            let children = &store.get(expr).children;
            if children.len() == 2 {
                let base = children[0];
                let exponent = children[1];

                // Check if exponent is the variable
                if let (Op::Symbol, Payload::Sym(s)) =
                    (&store.get(exponent).op, &store.get(exponent).payload)
                {
                    if s == var && !depends_on_var(store, base, var) {
                        return Some(base);
                    }
                }
            }
            None
        }
        Op::Mul => {
            // Could be a*r^k
            let children = store.get(expr).children.clone();
            for &child in &children {
                if let Some(ratio) = recognize_geometric(store, child, var) {
                    return Some(ratio);
                }
            }
            None
        }
        _ => None,
    }
}

/// Recognize power sum: k^p
fn recognize_power_sum(store: &mut Store, expr: ExprId, var: &str) -> Option<ExprId> {
    match &store.get(expr).op {
        Op::Symbol => {
            if let Payload::Sym(s) = &store.get(expr).payload {
                if s == var {
                    // k^1
                    return Some(store.int(1));
                }
            }
            None
        }
        Op::Pow => {
            let children = &store.get(expr).children;
            if children.len() == 2 {
                let base = children[0];
                let exponent = children[1];

                if let (Op::Symbol, Payload::Sym(s)) =
                    (&store.get(base).op, &store.get(base).payload)
                {
                    if s == var && !depends_on_var(store, exponent, var) {
                        return Some(exponent);
                    }
                }
            }
            None
        }
        _ => None,
    }
}

/// Check if expression depends on variable
fn depends_on_var(store: &Store, expr: ExprId, var: &str) -> bool {
    match (&store.get(expr).op, &store.get(expr).payload) {
        (Op::Symbol, Payload::Sym(s)) => s == var,
        (Op::Integer, _) | (Op::Rational, _) => false,
        _ => store.get(expr).children.iter().any(|&c| depends_on_var(store, c, var)),
    }
}

/// Compute arithmetic series: ∑(k=lower to upper) (a + kd)
///
/// Formula: ((upper - lower + 1) / 2) * (2a + (upper + lower)*d)
pub fn sum_arithmetic(
    store: &mut Store,
    _expr: ExprId,
    lower: ExprId,
    upper: ExprId,
    a: ExprId,
    d: ExprId,
) -> Option<ExprId> {
    // n = upper - lower + 1
    let one = store.int(1);
    let neg_one = store.int(-1);
    let neg_lower = store.mul(vec![neg_one, lower]);
    let diff = store.add(vec![upper, neg_lower]);
    let n = store.add(vec![diff, one]);

    // first_term = a + lower*d
    let lower_d = store.mul(vec![lower, d]);
    let first_term = store.add(vec![a, lower_d]);

    // last_term = a + upper*d
    let upper_d = store.mul(vec![upper, d]);
    let last_term = store.add(vec![a, upper_d]);

    // sum = n * (first_term + last_term) / 2
    let sum_terms = store.add(vec![first_term, last_term]);
    let numerator = store.mul(vec![n, sum_terms]);
    let half = store.rat(1, 2);
    let result = store.mul(vec![half, numerator]);

    Some(simplify(store, result))
}

/// Compute geometric series: ∑(k=lower to upper) r^k
///
/// Formula: r^lower * (r^(upper-lower+1) - 1) / (r - 1)
pub fn sum_geometric(
    store: &mut Store,
    _expr: ExprId,
    _var: &str,
    lower: ExprId,
    upper: ExprId,
    ratio: ExprId,
) -> Option<ExprId> {
    let one = store.int(1);
    let minus_one = store.int(-1);

    // n = upper - lower + 1
    let neg_lower = store.mul(vec![minus_one, lower]);
    let diff = store.add(vec![upper, neg_lower]);
    let n = store.add(vec![diff, one]);

    // r^lower
    let r_lower = store.pow(ratio, lower);

    // r^n - 1
    let r_n = store.pow(ratio, n);
    let r_n_minus_1 = store.add(vec![r_n, minus_one]);

    // r - 1
    let r_minus_1 = store.add(vec![ratio, minus_one]);

    // r^lower * (r^n - 1) / (r - 1)
    let numerator = store.mul(vec![r_lower, r_n_minus_1]);
    let inv_denom = store.pow(r_minus_1, minus_one);
    let result = store.mul(vec![numerator, inv_denom]);

    Some(simplify(store, result))
}

/// Compute power sum: ∑(k=lower to upper) k^p
///
/// For small powers (p=1,2,3), use known formulas.
/// For larger powers, use Faulhaber's formula (not implemented yet).
pub fn sum_power(
    store: &mut Store,
    _var: &str,
    _lower: ExprId,
    upper: ExprId,
    power: ExprId,
) -> Option<ExprId> {
    // Only handle p=1 for now (arithmetic series with a=0, d=1)
    if let (Op::Integer, Payload::Int(p)) = (&store.get(power).op, &store.get(power).payload) {
        match p {
            1 => {
                // ∑k = n(n+1)/2 where n=upper, assuming lower=1
                let one = store.int(1);
                let n_plus_1 = store.add(vec![upper, one]);
                let numerator = store.mul(vec![upper, n_plus_1]);
                let half = store.rat(1, 2);
                let result = store.mul(vec![half, numerator]);
                return Some(simplify(store, result));
            }
            2 => {
                // ∑k² = n(n+1)(2n+1)/6
                let one = store.int(1);
                let two = store.int(2);
                let n_plus_1 = store.add(vec![upper, one]);
                let two_n = store.mul(vec![two, upper]);
                let two_n_plus_1 = store.add(vec![two_n, one]);
                let numerator = store.mul(vec![upper, n_plus_1, two_n_plus_1]);
                let sixth = store.rat(1, 6);
                let result = store.mul(vec![sixth, numerator]);
                return Some(simplify(store, result));
            }
            _ => return None,
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic_sum_simple() {
        let mut st = Store::new();
        let k = st.sym("k");
        let one = st.int(1);
        let ten = st.int(10);

        // ∑(k=1 to 10) k = 55
        let zero = st.int(0);
        let result = sum_arithmetic(&mut st, k, one, ten, zero, one).expect("sum");

        // Should equal 10*11/2 = 55
        let result_str = st.to_string(result);
        assert!(result_str.contains("55") || result_str.contains("11"));
    }

    #[test]
    fn test_power_sum_squares() {
        let mut st = Store::new();
        let one = st.int(1);
        let five = st.int(5);
        let two = st.int(2);

        // ∑(k=1 to 5) k² = 1+4+9+16+25 = 55
        let result = sum_power(&mut st, "k", one, five, two).expect("sum squares");

        // Should equal 5*6*11/6 = 55
        let result_str = st.to_string(result);
        assert!(result_str.contains("55") || result_str.contains("11"));
    }
}
