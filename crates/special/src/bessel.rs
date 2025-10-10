//! Bessel functions
//!
//! Placeholder for Bessel function implementations (Phase 3)

use crate::SpecialFunction;
use expr_core::{ExprId, Store};

pub struct BesselJFunction;
pub struct BesselYFunction;
pub struct BesselIFunction;
pub struct BesselKFunction;

impl SpecialFunction for BesselJFunction {
    fn name(&self) -> &str {
        "BesselJ"
    }

    fn arity(&self) -> usize {
        2
    }

    fn eval(&self, args: &[f64]) -> Option<f64> {
        if args.len() != 2 {
            return None;
        }
        let nu = args[0];
        let x = args[1];

        // Only support integer orders for now
        if nu.fract() != 0.0 || nu < 0.0 || nu > 20.0 {
            return None;
        }

        Some(bessel_j(nu as i32, x))
    }

    fn derivative(
        &self,
        store: &mut Store,
        args: &[ExprId],
        arg_index: usize,
    ) -> Option<ExprId> {
        if args.len() != 2 || arg_index != 1 {
            return None;
        }

        // d/dx BesselJ(nu, x) = (BesselJ(nu-1, x) - BesselJ(nu+1, x)) / 2
        let nu = args[0];
        let x = args[1];
        let one = store.int(1);
        let neg_one = store.int(-1);

        let nu_minus_1 = store.add(vec![nu, neg_one]);
        let nu_plus_1 = store.add(vec![nu, one]);

        let bessel_prev = store.func("BesselJ", vec![nu_minus_1, x]);
        let bessel_next = store.func("BesselJ", vec![nu_plus_1, x]);

        let neg_bessel_next = store.mul(vec![neg_one, bessel_next]);
        let diff = store.add(vec![bessel_prev, neg_bessel_next]);
        let half = store.rat(1, 2);
        Some(store.mul(vec![diff, half]))
    }

    fn series(&self, _store: &mut Store, _args: &[ExprId], _order: usize) -> Option<ExprId> {
        None
    }
}

/// Compute Bessel J function using series expansion
fn bessel_j(n: i32, x: f64) -> f64 {
    if n < 0 {
        return if n % 2 == 0 {
            bessel_j(-n, x)
        } else {
            -bessel_j(-n, x)
        };
    }

    // Series: J_n(x) = sum_{k=0}^inf (-1)^k / (k! * (n+k)!) * (x/2)^(n+2k)
    let mut sum = 0.0;
    let half_x = x / 2.0;

    for k in 0..100 {
        let term = half_x.powi(n + 2 * k) / (factorial(k) * factorial(n + k));
        if k % 2 == 0 {
            sum += term;
        } else {
            sum -= term;
        }

        if term.abs() < 1e-15 {
            break;
        }
    }

    sum
}

fn factorial(n: i32) -> f64 {
    if n <= 0 {
        return 1.0;
    }
    (1..=n).map(|i| i as f64).product()
}

impl SpecialFunction for BesselYFunction {
    fn name(&self) -> &str {
        "BesselY"
    }

    fn arity(&self) -> usize {
        2
    }

    fn eval(&self, _args: &[f64]) -> Option<f64> {
        // BesselY is more complex and requires BesselJ, skip for now
        None
    }

    fn derivative(
        &self,
        store: &mut Store,
        args: &[ExprId],
        arg_index: usize,
    ) -> Option<ExprId> {
        if args.len() != 2 || arg_index != 1 {
            return None;
        }

        // d/dx BesselY(nu, x) = (BesselY(nu-1, x) - BesselY(nu+1, x)) / 2
        let nu = args[0];
        let x = args[1];
        let one = store.int(1);
        let neg_one = store.int(-1);

        let nu_minus_1 = store.add(vec![nu, neg_one]);
        let nu_plus_1 = store.add(vec![nu, one]);

        let bessel_prev = store.func("BesselY", vec![nu_minus_1, x]);
        let bessel_next = store.func("BesselY", vec![nu_plus_1, x]);

        let neg_bessel_next = store.mul(vec![neg_one, bessel_next]);
        let diff = store.add(vec![bessel_prev, neg_bessel_next]);
        let half = store.rat(1, 2);
        Some(store.mul(vec![diff, half]))
    }

    fn series(&self, _store: &mut Store, _args: &[ExprId], _order: usize) -> Option<ExprId> {
        None
    }
}

impl SpecialFunction for BesselIFunction {
    fn name(&self) -> &str {
        "BesselI"
    }

    fn arity(&self) -> usize {
        2
    }

    fn eval(&self, args: &[f64]) -> Option<f64> {
        if args.len() != 2 {
            return None;
        }
        let nu = args[0];
        let x = args[1];

        // Only support integer orders for now
        if nu.fract() != 0.0 || nu < 0.0 || nu > 20.0 {
            return None;
        }

        Some(bessel_i(nu as i32, x))
    }

    fn derivative(
        &self,
        store: &mut Store,
        args: &[ExprId],
        arg_index: usize,
    ) -> Option<ExprId> {
        if args.len() != 2 || arg_index != 1 {
            return None;
        }

        // d/dx BesselI(nu, x) = (BesselI(nu-1, x) + BesselI(nu+1, x)) / 2
        let nu = args[0];
        let x = args[1];
        let one = store.int(1);
        let neg_one = store.int(-1);

        let nu_minus_1 = store.add(vec![nu, neg_one]);
        let nu_plus_1 = store.add(vec![nu, one]);

        let bessel_prev = store.func("BesselI", vec![nu_minus_1, x]);
        let bessel_next = store.func("BesselI", vec![nu_plus_1, x]);

        let sum = store.add(vec![bessel_prev, bessel_next]);
        let half = store.rat(1, 2);
        Some(store.mul(vec![sum, half]))
    }

    fn series(&self, _store: &mut Store, _args: &[ExprId], _order: usize) -> Option<ExprId> {
        None
    }
}

/// Compute Modified Bessel I function using series expansion
fn bessel_i(n: i32, x: f64) -> f64 {
    if n < 0 {
        return bessel_i(-n, x);
    }

    // Series: I_n(x) = sum_{k=0}^inf 1 / (k! * (n+k)!) * (x/2)^(n+2k)
    let mut sum = 0.0;
    let half_x = x / 2.0;

    for k in 0..100 {
        let term = half_x.powi(n + 2 * k) / (factorial(k) * factorial(n + k));
        sum += term;

        if term.abs() < 1e-15 {
            break;
        }
    }

    sum
}

impl SpecialFunction for BesselKFunction {
    fn name(&self) -> &str {
        "BesselK"
    }

    fn arity(&self) -> usize {
        2
    }

    fn eval(&self, _args: &[f64]) -> Option<f64> {
        // BesselK is complex, skip numerical evaluation for now
        None
    }

    fn derivative(
        &self,
        store: &mut Store,
        args: &[ExprId],
        arg_index: usize,
    ) -> Option<ExprId> {
        if args.len() != 2 || arg_index != 1 {
            return None;
        }

        // d/dx BesselK(nu, x) = -(BesselK(nu-1, x) + BesselK(nu+1, x)) / 2
        let nu = args[0];
        let x = args[1];
        let one = store.int(1);
        let neg_one = store.int(-1);

        let nu_minus_1 = store.add(vec![nu, neg_one]);
        let nu_plus_1 = store.add(vec![nu, one]);

        let bessel_prev = store.func("BesselK", vec![nu_minus_1, x]);
        let bessel_next = store.func("BesselK", vec![nu_plus_1, x]);

        let sum = store.add(vec![bessel_prev, bessel_next]);
        let neg_sum = store.mul(vec![neg_one, sum]);
        let half = store.rat(1, 2);
        Some(store.mul(vec![neg_sum, half]))
    }

    fn series(&self, _store: &mut Store, _args: &[ExprId], _order: usize) -> Option<ExprId> {
        None
    }
}
