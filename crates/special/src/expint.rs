//! Exponential Integrals
//!
//! Implements:
//! - Ei(z): Exponential integral Ei(z) = -∫_{-z}^∞ e^(-t)/t dt
//! - E₁(z): E₁(z) = ∫_z^∞ e^(-t)/t dt
//! - Related: li(z) = Ei(ln z) (logarithmic integral)
//!
//! Properties:
//! - Ei(z) = -E₁(-z) for real z
//! - d/dz Ei(z) = e^z/z
//! - d/dz E₁(z) = -e^(-z)/z

use super::SpecialFunction;
use expr_core::{ExprId, Store};

pub struct EiFunction;

impl SpecialFunction for EiFunction {
    fn name(&self) -> &str {
        "Ei"
    }

    fn arity(&self) -> usize {
        1
    }

    /// Numerical evaluation (placeholder)
    fn eval(&self, args: &[f64]) -> Option<f64> {
        if args.len() != 1 {
            return None;
        }

        let z = args[0];

        if z == 0.0 {
            // Ei(0) is undefined (logarithmic singularity)
            return None;
        }

        // Series: Ei(z) = gamma + ln|z| + Σ_{k=1..∞} z^k / (k·k!)
        const EULER_GAMMA: f64 = 0.577_215_664_901_532_9_f64;
        let ln_term = z.abs().ln();

        // Accumulate using term recurrence for z^k/k!: term_{k+1} = term_k * z/(k+1)
        let mut k: u64 = 1;
        let mut term = z; // z^1/1!
        let mut sum = term / (k as f64);
        let mut prev_sum = sum;
        let max_iter = 200usize;
        for i in 1..max_iter {
            let kk = (k + 1) as f64;
            term *= z / kk; // update to z^{k+1}/(k+1)!
            k += 1;
            sum += term / (k as f64);
            if (sum - prev_sum).abs() <= 1e-16 * sum.abs().max(1.0) {
                break;
            }
            prev_sum = sum;
            if i + 1 == max_iter {
                // If not converged, still return best effort
                break;
            }
        }

        Some(EULER_GAMMA + ln_term + sum)
    }

    /// Derivative: d/dz Ei(z) = e^z/z
    fn derivative(&self, store: &mut Store, args: &[ExprId], _arg_index: usize) -> Option<ExprId> {
        if args.len() != 1 {
            return None;
        }

        let z = args[0];

        // exp(z) / z = exp(z) * z^(-1)
        let exp_z = store.func("exp", vec![z]);
        let neg_one = store.int(-1);
        let z_inv = store.pow(z, neg_one);

        Some(store.mul(vec![exp_z, z_inv]))
    }

    /// Series expansion (not implemented yet)
    fn series(&self, store: &mut Store, args: &[ExprId], order: usize) -> Option<ExprId> {
        if args.len() != 1 {
            return None;
        }

        let z = args[0];

        if order == 0 {
            // Truncate to constant: γ + ln|z| ~ undefined at 0, but return γ for formal series
            // Keep it simple: return Gamma constant approximation
            const GAMMA: f64 = 0.577_215_664_901_532_9;
            let scale = 1_000_000.0f64;
            return Some(store.rat((GAMMA * scale) as i64, scale as i64));
        }

        // γ (Euler–Mascheroni constant) as rational approximation
        const GAMMA: f64 = 0.577_215_664_901_532_9;
        let scale = 1_000_000.0f64;
        let gamma_c = store.rat((GAMMA * scale) as i64, scale as i64);

        // ln|z|
        let abs_z = store.func("abs", vec![z]);
        let ln_abs_z = store.func("ln", vec![abs_z]);

        // Σ_{k=1..N} z^k / (k * k!)
        let mut sum_terms: Vec<ExprId> = vec![gamma_c, ln_abs_z];
        let mut fact: i128 = 1; // k!
        for k in 1..=order {
            fact = fact.saturating_mul(k as i128);
            if fact == 0 || fact.unsigned_abs() > i64::MAX as u128 {
                break;
            }
            let denom = fact.saturating_mul(k as i128);
            if denom == 0 || denom.unsigned_abs() > i64::MAX as u128 {
                break;
            }
            let denom_i64 = denom as i64;
            let coeff = store.rat(1, denom_i64);
            let k_i = store.int(k as i64);
            let pow = store.pow(z, k_i);
            sum_terms.push(store.mul(vec![coeff, pow]));
        }

        Some(store.add(sum_terms))
    }
}

/// Create an Ei function expression
pub fn ei(store: &mut Store, arg: ExprId) -> ExprId {
    store.func("Ei", vec![arg])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ei_undefined_at_zero() {
        let e = EiFunction;
        assert_eq!(e.eval(&[0.0]), None);
    }

    #[test]
    fn ei_symbolic() {
        let mut st = Store::new();
        let x = st.sym("x");
        let ex = ei(&mut st, x);

        assert!(st.to_string(ex).contains("Ei"));
    }

    #[test]
    fn ei_one_numeric() {
        let e = EiFunction;
        // Ei(1) ≈ 1.8951178163559368
        let v = e.eval(&[1.0]).unwrap();
        assert!((v - 1.895_117_816_355_936_8_f64).abs() < 1e-12);
    }

    #[test]
    fn ei_minus_one_numeric() {
        let e = EiFunction;
        // Ei(-1) ≈ -0.2193839343955203
        let v = e.eval(&[-1.0]).unwrap();
        assert!((v + 0.219_383_934_395_520_3_f64).abs() < 1e-12);
    }

    #[test]
    fn ei_derivative_symbolic() {
        let mut st = Store::new();
        let x = st.sym("x");

        let e = EiFunction;
        let deriv = e.derivative(&mut st, &[x], 0).unwrap();

        // Should be exp(x)/x
        let result = st.to_string(deriv);
        assert!(result.contains("exp"));
        assert!(result.contains("x"));
    }
}
