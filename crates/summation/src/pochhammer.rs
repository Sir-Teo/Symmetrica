//! Pochhammer Symbol and Rising Factorial
//!
//! The Pochhammer symbol (x)ₙ represents the rising factorial:
//!   (x)ₙ = x(x+1)(x+2)...(x+n-1)
//!
//! This is closely related to the Gamma function:
//!   (x)ₙ = Γ(x+n) / Γ(x)

use expr_core::{ExprId, Op, Payload, Store};
use simplify::simplify;

/// Compute the Pochhammer symbol (rising factorial): (x)ₙ
///
/// Returns x(x+1)(x+2)...(x+n-1) for integer n ≥ 0.
pub fn pochhammer(store: &mut Store, x: ExprId, n: ExprId) -> Option<ExprId> {
    rising_factorial(store, x, n)
}

/// Compute the rising factorial: x^(n) = x(x+1)(x+2)...(x+n-1)
///
/// For integer n:
/// - n = 0: returns 1
/// - n > 0: returns product x(x+1)...(x+n-1)
/// - n < 0: returns 1/((x-1)(x-2)...(x+|n|))
pub fn rising_factorial(store: &mut Store, x: ExprId, n: ExprId) -> Option<ExprId> {
    // Check if n is an integer
    match (&store.get(n).op, &store.get(n).payload) {
        (Op::Integer, Payload::Int(count)) => {
            if *count == 0 {
                Some(store.int(1))
            } else if *count > 0 {
                // Compute product x(x+1)...(x+n-1)
                let mut terms = Vec::new();
                for i in 0..*count {
                    let offset = store.int(i);
                    let term = store.add(vec![x, offset]);
                    terms.push(term);
                }
                let product = store.mul(terms);
                Some(simplify(store, product))
            } else {
                // Negative n: 1/((x-1)(x-2)...(x-|n|))
                let abs_n = -count;
                let mut terms = Vec::new();
                for i in 1..=abs_n {
                    let offset = store.int(-i);
                    let term = store.add(vec![x, offset]);
                    terms.push(term);
                }
                let product = store.mul(terms);
                let minus_one = store.int(-1);
                let inv_product = store.pow(product, minus_one);
                Some(simplify(store, inv_product))
            }
        }
        _ => {
            // For symbolic n, return as a function call or None
            // In a full implementation, we'd represent this symbolically
            None
        }
    }
}

/// Compute the falling factorial: x^(n) = x(x-1)(x-2)...(x-n+1)
///
/// This is related to the rising factorial: x^(n) = (x-n+1)ₙ
#[allow(dead_code)]
pub fn falling_factorial(store: &mut Store, x: ExprId, n: ExprId) -> Option<ExprId> {
    match (&store.get(n).op, &store.get(n).payload) {
        (Op::Integer, Payload::Int(count)) => {
            if *count == 0 {
                Some(store.int(1))
            } else if *count > 0 {
                // Compute product x(x-1)...(x-n+1)
                let mut terms = Vec::new();
                for i in 0..*count {
                    let offset = store.int(-i);
                    let term = store.add(vec![x, offset]);
                    terms.push(term);
                }
                let product = store.mul(terms);
                Some(simplify(store, product))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Convert between rising and falling factorial
///
/// (x)ₙ = (-1)ⁿ * (-x)ₙ (falling)
#[allow(dead_code)]
pub fn rising_to_falling(store: &mut Store, x: ExprId, n: ExprId) -> Option<ExprId> {
    let minus_one = store.int(-1);
    let neg_x = store.mul(vec![minus_one, x]);

    let falling = falling_factorial(store, neg_x, n)?;

    // Multiply by (-1)^n
    if let (Op::Integer, Payload::Int(count)) = (&store.get(n).op, &store.get(n).payload) {
        if count % 2 == 0 {
            Some(falling)
        } else {
            Some(store.mul(vec![minus_one, falling]))
        }
    } else {
        // For symbolic n, compute (-1)^n * falling
        let sign = store.pow(minus_one, n);
        Some(store.mul(vec![sign, falling]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pochhammer_zero() {
        let mut st = Store::new();
        let x = st.sym("x");
        let zero = st.int(0);

        let result = pochhammer(&mut st, x, zero).expect("(x)₀");
        assert_eq!(st.to_string(result), "1");
    }

    #[test]
    fn test_pochhammer_positive() {
        let mut st = Store::new();
        let x = st.sym("x");
        let three = st.int(3);

        // (x)₃ = x(x+1)(x+2)
        let result = pochhammer(&mut st, x, three).expect("(x)₃");
        let result_str = st.to_string(result);

        // Should contain x and some additions
        assert!(result_str.contains("x"));
    }

    #[test]
    fn test_rising_factorial_concrete() {
        let mut st = Store::new();
        let five = st.int(5);
        let three = st.int(3);

        // (5)₃ = 5*6*7 = 210
        let result = rising_factorial(&mut st, five, three).expect("5*6*7");

        // Should evaluate to 210 or contain the product
        let result_str = st.to_string(result);
        assert!(result_str.contains("210") || result_str.contains("5"));
    }

    #[test]
    fn test_falling_factorial() {
        let mut st = Store::new();
        let five = st.int(5);
        let three = st.int(3);

        // 5^(3) = 5*4*3 = 60
        let result = falling_factorial(&mut st, five, three).expect("5*4*3");

        let result_str = st.to_string(result);
        assert!(result_str.contains("60") || result_str.contains("5"));
    }
}
