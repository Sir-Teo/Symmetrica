//! Gamma Function and Related Functions
//!
//! Implements:
//! - Gamma(z): Γ(z) = ∫₀^∞ t^(z-1) e^(-t) dt
//! - Factorial: n! = Γ(n+1)
//! - Binomial coefficients
//! - Digamma (Ψ) and polygamma functions (future)
//!
//! Properties:
//! - Γ(n+1) = n! for non-negative integers n
//! - Γ(z+1) = z·Γ(z) (recurrence relation)
//! - Γ(1/2) = √π
//! - Reflection formula: Γ(z)Γ(1-z) = π/sin(πz)

use super::SpecialFunction;
use expr_core::{ExprId, Store};

pub struct GammaFunction;

impl SpecialFunction for GammaFunction {
    fn name(&self) -> &str {
        "Gamma"
    }

    fn arity(&self) -> usize {
        1
    }

    /// Numerical evaluation using Lanczos approximation with reflection
    fn eval(&self, args: &[f64]) -> Option<f64> {
        if args.len() != 1 {
            return None;
        }

        let z = args[0];

        if z <= 0.0 && z.fract() == 0.0 {
            // Poles at non-positive integers
            return None;
        }
        if z == 1.0 {
            return Some(1.0);
        }
        if z == 0.5 {
            return Some(std::f64::consts::PI.sqrt());
        }

        // Lanczos approximation constants (g=7, n=9), from Numerical Recipes/Wikipedia
        // Coefficients for double precision
        const G: f64 = 7.0;
        const COEFFS: [f64; 9] = [
            0.999_999_999_999_809_9,
            676.520_368_121_885_1,
            -1_259.139_216_722_402_8,
            771.323_428_777_653_1,
            -176.615_029_162_140_6,
            12.507_343_278_686_905,
            -0.138_571_095_265_720_12,
            9.984_369_578_019_572e-6,
            1.505_632_735_149_311_6e-7,
        ];

        fn lanczos_gamma(x: f64) -> f64 {
            let x0 = x - 1.0;
            let mut a = COEFFS[0];
            for (k, c) in COEFFS.iter().enumerate().skip(1) {
                a += c / (x0 + k as f64);
            }
            let t = x0 + G + 0.5;
            (2.0 * std::f64::consts::PI).sqrt() * t.powf(x0 + 0.5) * (-t).exp() * a
        }

        if z < 0.5 {
            // Reflection formula: Γ(z) = π / (sin(πz) * Γ(1 - z))
            let sin_term = (std::f64::consts::PI * z).sin();
            if sin_term.abs() < 1e-15 {
                return None;
            }
            let one_minus_z = 1.0 - z;
            let gamma_1_minus_z = lanczos_gamma(one_minus_z);
            if !gamma_1_minus_z.is_finite() {
                return None;
            }
            Some(std::f64::consts::PI / (sin_term * gamma_1_minus_z))
        } else {
            Some(lanczos_gamma(z))
        }
    }

    /// Derivative: d/dz Γ(z) = Γ(z) · Ψ(z) where Ψ is the digamma function
    fn derivative(&self, store: &mut Store, args: &[ExprId], _arg_index: usize) -> Option<ExprId> {
        if args.len() != 1 {
            return None;
        }

        // d/dz Γ(z) = Γ(z) * Ψ(z)
        // For now, return symbolic form
        let gamma_z = store.func("Gamma", vec![args[0]]);
        let psi_z = store.func("Digamma", vec![args[0]]);
        Some(store.mul(vec![gamma_z, psi_z]))
    }

    /// Series expansion (not implemented yet)
    fn series(&self, _store: &mut Store, _args: &[ExprId], _order: usize) -> Option<ExprId> {
        // TODO: Implement series expansion around specific points
        None
    }
}

/// Create a Gamma function expression
pub fn gamma(store: &mut Store, arg: ExprId) -> ExprId {
    store.func("Gamma", vec![arg])
}

/// Factorial: n! = Gamma(n+1)
pub fn factorial(store: &mut Store, n: ExprId) -> ExprId {
    let one = store.int(1);
    let n_plus_1 = store.add(vec![n, one]);
    gamma(store, n_plus_1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gamma_special_values() {
        let g = GammaFunction;

        // Γ(1) = 1
        assert_eq!(g.eval(&[1.0]), Some(1.0));

        // Γ(1/2) = √π
        let result = g.eval(&[0.5]).unwrap();
        assert!((result - std::f64::consts::PI.sqrt()).abs() < 1e-10);
    }

    #[test]
    fn gamma_undefined_at_zero() {
        let g = GammaFunction;

        // Γ(0) is undefined
        assert_eq!(g.eval(&[0.0]), None);

        // Γ(-1) is undefined
        assert_eq!(g.eval(&[-1.0]), None);
    }

    #[test]
    fn gamma_symbolic() {
        let mut st = Store::new();
        let x = st.sym("x");
        let gx = gamma(&mut st, x);

        assert!(st.to_string(gx).contains("Gamma"));
        assert!(st.to_string(gx).contains("x"));
    }

    #[test]
    fn gamma_five_is_factorial_four() {
        let g = GammaFunction;
        // Γ(5) = 24
        let val = g.eval(&[5.0]).unwrap();
        assert!((val - 24.0).abs() < 1e-10);
    }

    #[test]
    fn gamma_two_point_five() {
        let g = GammaFunction;
        // Γ(2.5) ≈ 1.329340388
        let val = g.eval(&[2.5]).unwrap();
        assert!((val - 1.329_340_388_f64).abs() < 1e-9);
    }

    #[test]
    fn gamma_reflection_formula() {
        let g = GammaFunction;
        // Test reflection formula for z < 0.5
        let val = g.eval(&[0.3]).unwrap();
        assert!(val.is_finite() && val > 0.0);
    }

    #[test]
    fn gamma_derivative_symbolic() {
        let g = GammaFunction;
        let mut st = Store::new();
        let x = st.sym("x");
        let deriv = g.derivative(&mut st, &[x], 0);
        assert!(deriv.is_some());
    }

    #[test]
    fn gamma_series_unimplemented() {
        let g = GammaFunction;
        let mut st = Store::new();
        let x = st.sym("x");
        let series = g.series(&mut st, &[x], 5);
        assert!(series.is_none());
    }

    #[test]
    fn gamma_name_and_arity() {
        let g = GammaFunction;
        assert_eq!(g.name(), "Gamma");
        assert_eq!(g.arity(), 1);
    }

    #[test]
    fn gamma_wrong_arity() {
        let g = GammaFunction;
        assert!(g.eval(&[1.0, 2.0]).is_none());
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        assert!(g.derivative(&mut st, &[x, y], 0).is_none());
    }

    #[test]
    fn test_factorial() {
        let mut st = Store::new();
        let five = st.int(5);
        let fact = factorial(&mut st, five);
        assert!(st.to_string(fact).contains("Gamma"));
    }
}
