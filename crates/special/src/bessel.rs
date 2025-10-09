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

    fn eval(&self, _args: &[f64]) -> Option<f64> {
        None
    }

    fn derivative(
        &self,
        _store: &mut Store,
        _args: &[ExprId],
        _arg_index: usize,
    ) -> Option<ExprId> {
        None
    }

    fn series(&self, _store: &mut Store, _args: &[ExprId], _order: usize) -> Option<ExprId> {
        None
    }
}

impl SpecialFunction for BesselYFunction {
    fn name(&self) -> &str {
        "BesselY"
    }

    fn arity(&self) -> usize {
        2
    }

    fn eval(&self, _args: &[f64]) -> Option<f64> {
        None
    }

    fn derivative(
        &self,
        _store: &mut Store,
        _args: &[ExprId],
        _arg_index: usize,
    ) -> Option<ExprId> {
        None
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

    fn eval(&self, _args: &[f64]) -> Option<f64> {
        None
    }

    fn derivative(
        &self,
        _store: &mut Store,
        _args: &[ExprId],
        _arg_index: usize,
    ) -> Option<ExprId> {
        None
    }

    fn series(&self, _store: &mut Store, _args: &[ExprId], _order: usize) -> Option<ExprId> {
        None
    }
}

impl SpecialFunction for BesselKFunction {
    fn name(&self) -> &str {
        "BesselK"
    }

    fn arity(&self) -> usize {
        2
    }

    fn eval(&self, _args: &[f64]) -> Option<f64> {
        None
    }

    fn derivative(
        &self,
        _store: &mut Store,
        _args: &[ExprId],
        _arg_index: usize,
    ) -> Option<ExprId> {
        None
    }

    fn series(&self, _store: &mut Store, _args: &[ExprId], _order: usize) -> Option<ExprId> {
        None
    }
}
