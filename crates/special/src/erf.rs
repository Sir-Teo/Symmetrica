//! Error Functions
//!
//! Implements:
//! - erf(z): Error function = (2/√π) ∫₀^z e^(-t²) dt
//! - erfc(z): Complementary error function = 1 - erf(z)
//! - erfi(z): Imaginary error function = -i·erf(iz)
//!
//! Properties:
//! - erf(-z) = -erf(z) (odd function)
//! - erf(0) = 0
//! - erf(∞) = 1
//! - erfc(z) = 1 - erf(z)
//! - d/dz erf(z) = (2/√π) e^(-z²)
use super::SpecialFunction;
use expr_core::{ExprId, Store};

pub struct ErfFunction;

impl SpecialFunction for ErfFunction {
    fn name(&self) -> &str {
        "erf"
    }

    fn arity(&self) -> usize {
        1
    }

    /// Numerical evaluation using series expansion for small |z| or continued fraction for large |z|
    fn eval(&self, args: &[f64]) -> Option<f64> {
        if args.len() != 1 {
            return None;
        }

        let z = args[0];

        // Special cases
        if z == 0.0 {
            return Some(0.0);
        }

        if z.is_infinite() {
            return Some(if z > 0.0 { 1.0 } else { -1.0 });
        }

        // For small |z|, use Maclaurin series (fast convergence around 0)
        // erf(z) ≈ (2/√π) Σ((-1)^n z^(2n+1))/(n!(2n+1))
        if z.abs() < 2.0 {
            let sqrt_pi = std::f64::consts::PI.sqrt();
            let two_over_sqrt_pi = 2.0 / sqrt_pi;

            let mut sum = 0.0;
            let mut term = z;
            let z_sq = z * z;

            for n in 0..30 {
                sum += term / (2 * n + 1) as f64;
                term *= -z_sq / (n + 1) as f64;

                if term.abs() < 1e-15 {
                    break;
                }
            }

            return Some(two_over_sqrt_pi * sum);
        }

        // Large |z|: use Abramowitz & Stegun 7.1.26 approximation
        // erf(x) ≈ sign(x) * [1 - (((((a5 t + a4) t + a3) t + a2) t + a1) t) e^{-x^2}],
        // where t = 1/(1 + p x), p = 0.3275911, and
        // a1=0.254829592, a2=-0.284496736, a3=1.421413741, a4=-1.453152027, a5=1.061405429
        let x = z.abs();
        let sign = if z < 0.0 { -1.0 } else { 1.0 };
        let p = 0.327_591_1_f64;
        let a1 = 0.254_829_592_f64;
        let a2 = -0.284_496_736_f64;
        let a3 = 1.421_413_741_f64;
        let a4 = -1.453_152_027_f64;
        let a5 = 1.061_405_429_f64;
        let t = 1.0 / (1.0 + p * x);
        let poly = ((((a5 * t + a4) * t + a3) * t + a2) * t + a1) * t;
        let y = 1.0 - poly * (-x * x).exp();
        Some(sign * y)
    }

    /// Derivative: d/dz erf(z) = (2/√π) exp(-z²)
    fn derivative(&self, store: &mut Store, args: &[ExprId], _arg_index: usize) -> Option<ExprId> {
        if args.len() != 1 {
            return None;
        }

        let z = args[0];

        // (2/√π) * exp(-z²)
        let sqrt_pi = std::f64::consts::PI.sqrt();
        let coeff = store.rat((2.0 * 1_000_000.0) as i64, (sqrt_pi * 1_000_000.0) as i64);

        let two = store.int(2);
        let z_sq = store.pow(z, two);
        let neg_one = store.int(-1);
        let neg_z_sq = store.mul(vec![neg_one, z_sq]);
        let exp_term = store.func("exp", vec![neg_z_sq]);

        Some(store.mul(vec![coeff, exp_term]))
    }

    /// Series expansion around z=0 up to the given order (inclusive).
    fn series(&self, store: &mut Store, args: &[ExprId], order: usize) -> Option<ExprId> {
        if args.len() != 1 {
            return None;
        }

        let z = args[0];

        if order == 0 {
            return Some(store.int(0));
        }

        // Prefactor 2/sqrt(pi) as a rational approximation.
        let sqrt_pi = std::f64::consts::PI.sqrt();
        let scale = 1_000_000.0f64;
        let two_over_sqrt_pi = store.rat((2.0 * scale) as i64, (sqrt_pi * scale) as i64);

        // Sum terms up to the highest odd power <= order.
        // term_n = (-1)^n z^(2n+1) / (n! * (2n+1))
        let mut terms: Vec<ExprId> = Vec::new();
        let mut n: usize = 0;
        let mut fact: i64 = 1; // n!
        loop {
            let p = 2 * n + 1;
            if p > order {
                break;
            }
            if n > 0 {
                // update factorial: n! from (n-1)! -> n! = n * (n-1)!
                let n_i = n as i64;
                fact = fact.saturating_mul(n_i);
                if fact == 0 {
                    return None;
                }
            }
            let denom = (fact as i128) * (p as i128);
            // guard overflow for i64
            if denom.unsigned_abs() > i64::MAX as u128 {
                break;
            }
            let denom_i64 = denom as i64;
            let sign = if n.is_multiple_of(2) { 1 } else { -1 };
            let coeff = store.rat(sign, denom_i64);
            let p_i = store.int(p as i64);
            let pow = store.pow(z, p_i);
            let term = store.mul(vec![two_over_sqrt_pi, coeff, pow]);
            terms.push(term);
            n += 1;
            // prepare next factorial increment in next loop iteration
        }

        if terms.is_empty() {
            return Some(store.int(0));
        }
        Some(store.add(terms))
    }
}

/// Create an erf function expression
pub fn erf(store: &mut Store, arg: ExprId) -> ExprId {
    store.func("erf", vec![arg])
}

/// Complementary error function: erfc(z) = 1 - erf(z)
pub fn erfc(store: &mut Store, arg: ExprId) -> ExprId {
    let one = store.int(1);
    let erf_z = erf(store, arg);
    let neg_one = store.int(-1);
    let neg_erf = store.mul(vec![neg_one, erf_z]);
    store.add(vec![one, neg_erf])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erf_at_zero() {
        let e = ErfFunction;
        assert_eq!(e.eval(&[0.0]), Some(0.0));
    }

    #[test]
    fn erf_small_values() {
        let e = ErfFunction;

        // erf(0.5) ≈ 0.5205
        let result = e.eval(&[0.5]).unwrap();
        assert!((result - 0.5205).abs() < 0.001);

        // erf is odd: erf(-x) = -erf(x)
        let result_neg = e.eval(&[-0.5]).unwrap();
        assert!((result + result_neg).abs() < 1e-10);
    }

    #[test]
    fn erf_symbolic() {
        let mut st = Store::new();
        let x = st.sym("x");
        let ex = erf(&mut st, x);

        assert!(st.to_string(ex).contains("erf"));
    }

    #[test]
    fn erf_derivative_symbolic() {
        let mut st = Store::new();
        let x = st.sym("x");

        let e = ErfFunction;
        let deriv = e.derivative(&mut st, &[x], 0).unwrap();

        // Should contain exp(-x²)
        let result = st.to_string(deriv);
        assert!(result.contains("exp"));
    }
}
