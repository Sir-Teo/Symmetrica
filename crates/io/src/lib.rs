//! I/O crate (stub): lightweight parser/printers will live here.
#![allow(unused)]

/// LaTeX printer for expressions.
pub mod latex;

pub use latex::to_latex;
/// S-expression serializer and parser.
pub mod sexpr;

pub use sexpr::{from_sexpr, to_sexpr};
/// JSON serializer (no external deps)
pub mod json;

pub use json::{from_json, to_json};
