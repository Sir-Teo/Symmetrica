//! Special Functions Module (Phase 3, v1.2)
//!
//! This module provides special mathematical functions:
//! - Gamma function and related functions (factorial, double factorial, binomial)
//! - Error functions (erf, erfc, erfi)
//! - Exponential integrals (Ei, E1)
//! - Bessel functions (future)
//! - Hypergeometric functions (future)
//!
//! Each function includes:
//! - Symbolic representation in expression trees
//! - Numerical evaluation (evalf integration)
//! - Symbolic differentiation rules
//! - Series expansions where applicable

#![deny(warnings)]

use expr_core::{ExprId, Store};

pub mod erf;
pub mod expint;
pub mod gamma;

/// Special function trait for uniform handling
pub trait SpecialFunction {
    /// Function name (e.g., "Gamma", "erf")
    fn name(&self) -> &str;

    /// Number of arguments
    fn arity(&self) -> usize;

    /// Numerical evaluation at a point (if possible)
    fn eval(&self, args: &[f64]) -> Option<f64>;

    /// Symbolic derivative with respect to argument index
    fn derivative(&self, store: &mut Store, args: &[ExprId], arg_index: usize) -> Option<ExprId>;

    /// Series expansion around a point (if applicable)
    fn series(&self, store: &mut Store, args: &[ExprId], order: usize) -> Option<ExprId>;
}

/// Register special functions in the expression system
pub fn register_special_functions() {
    // TODO: Register special functions with the function registry
    // This will allow them to be recognized in differentiation, integration, etc.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaffold_compiles() {
        // Placeholder test to ensure crate compiles
        register_special_functions();
    }
}
