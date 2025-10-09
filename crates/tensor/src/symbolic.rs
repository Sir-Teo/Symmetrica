//! Symbolic Tensor Operations
//!
//! This module provides tensors with symbolic (expression-based) components.
//! Useful for symbolic manipulation of tensor equations.

use std::fmt;

/// A symbolic tensor where each component is represented by a string expression
/// This is a lightweight wrapper for demonstration purposes
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolicTensor {
    shape: Vec<usize>,
    /// Components stored as symbolic expressions (strings for now)
    /// In a full implementation, these would be ExprId from expr_core
    components: Vec<String>,
}

impl SymbolicTensor {
    /// Create a new symbolic tensor with given shape
    /// Components are initialized to "0"
    pub fn new(shape: Vec<usize>) -> Self {
        let size: usize = shape.iter().product();
        Self { shape, components: vec!["0".to_string(); size] }
    }

    /// Create from shape and component expressions
    pub fn from_components(shape: Vec<usize>, components: Vec<String>) -> Self {
        let size: usize = shape.iter().product();
        assert_eq!(size, components.len(), "Component count must match shape");
        Self { shape, components }
    }

    /// Get the shape of the tensor
    pub fn shape(&self) -> &[usize] {
        &self.shape
    }

    /// Get the rank (number of dimensions)
    pub fn rank(&self) -> usize {
        self.shape.len()
    }

    /// Get component at given index
    pub fn get(&self, idx: &[usize]) -> &str {
        let offset = self.compute_offset(idx);
        &self.components[offset]
    }

    /// Set component at given index
    pub fn set(&mut self, idx: &[usize], expr: String) {
        let offset = self.compute_offset(idx);
        self.components[offset] = expr;
    }

    fn compute_offset(&self, idx: &[usize]) -> usize {
        assert_eq!(idx.len(), self.shape.len(), "Index rank mismatch");
        let mut offset = 0;
        let mut stride = 1;
        for i in (0..idx.len()).rev() {
            assert!(idx[i] < self.shape[i], "Index out of bounds");
            offset += idx[i] * stride;
            stride *= self.shape[i];
        }
        offset
    }

    /// Create a symbolic vector with named components
    /// Example: vector("v", 3) creates ["v_0", "v_1", "v_2"]
    pub fn vector(name: &str, dim: usize) -> Self {
        let components: Vec<String> = (0..dim).map(|i| format!("{}_{}", name, i)).collect();
        Self::from_components(vec![dim], components)
    }

    /// Create a symbolic matrix with named components
    /// Example: matrix("A", 2, 2) creates ["A_00", "A_01", "A_10", "A_11"]
    pub fn matrix(name: &str, rows: usize, cols: usize) -> Self {
        let mut components = Vec::new();
        for i in 0..rows {
            for j in 0..cols {
                components.push(format!("{}_{}_{}", name, i, j));
            }
        }
        Self::from_components(vec![rows, cols], components)
    }

    /// Symbolic addition (component-wise)
    pub fn add(&self, other: &SymbolicTensor) -> Result<SymbolicTensor, String> {
        if self.shape != other.shape {
            return Err("Shape mismatch for addition".to_string());
        }

        let components: Vec<String> = self
            .components
            .iter()
            .zip(&other.components)
            .map(|(a, b)| {
                if a == "0" {
                    b.clone()
                } else if b == "0" {
                    a.clone()
                } else {
                    format!("({} + {})", a, b)
                }
            })
            .collect();

        Ok(SymbolicTensor::from_components(self.shape.clone(), components))
    }

    /// Symbolic scalar multiplication
    pub fn scale(&self, scalar: &str) -> SymbolicTensor {
        let components: Vec<String> = self
            .components
            .iter()
            .map(|c| {
                if c == "0" {
                    "0".to_string()
                } else if scalar == "1" {
                    c.clone()
                } else {
                    format!("({}*{})", scalar, c)
                }
            })
            .collect();

        SymbolicTensor::from_components(self.shape.clone(), components)
    }

    /// Symbolic transpose (for rank-2 tensors)
    pub fn transpose(&self) -> Result<SymbolicTensor, String> {
        if self.rank() != 2 {
            return Err("Transpose only defined for rank-2 tensors".to_string());
        }

        let rows = self.shape[0];
        let cols = self.shape[1];
        let mut result = SymbolicTensor::new(vec![cols, rows]);

        for i in 0..rows {
            for j in 0..cols {
                let val = self.get(&[i, j]).to_string();
                result.set(&[j, i], val);
            }
        }

        Ok(result)
    }

    /// Symbolic contraction (trace for rank-2)
    pub fn trace(&self) -> Result<String, String> {
        if self.rank() != 2 {
            return Err("Trace only defined for rank-2 tensors".to_string());
        }
        if self.shape[0] != self.shape[1] {
            return Err("Trace requires square matrix".to_string());
        }

        let dim = self.shape[0];
        let mut terms = Vec::new();
        for i in 0..dim {
            let val = self.get(&[i, i]);
            if val != "0" {
                terms.push(val.to_string());
            }
        }

        if terms.is_empty() {
            Ok("0".to_string())
        } else {
            Ok(terms.join(" + "))
        }
    }
}

impl fmt::Display for SymbolicTensor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SymbolicTensor{:?}[", self.shape)?;
        for (i, comp) in self.components.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", comp)?;
        }
        write!(f, "]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbolic_vector_creation() {
        let v = SymbolicTensor::vector("v", 3);
        assert_eq!(v.shape(), &[3]);
        assert_eq!(v.get(&[0]), "v_0");
        assert_eq!(v.get(&[1]), "v_1");
        assert_eq!(v.get(&[2]), "v_2");
    }

    #[test]
    fn test_symbolic_matrix_creation() {
        let m = SymbolicTensor::matrix("A", 2, 2);
        assert_eq!(m.shape(), &[2, 2]);
        assert_eq!(m.get(&[0, 0]), "A_0_0");
        assert_eq!(m.get(&[0, 1]), "A_0_1");
        assert_eq!(m.get(&[1, 0]), "A_1_0");
        assert_eq!(m.get(&[1, 1]), "A_1_1");
    }

    #[test]
    fn test_symbolic_addition() {
        let v1 = SymbolicTensor::vector("v", 2);
        let v2 = SymbolicTensor::vector("w", 2);
        let sum = v1.add(&v2).unwrap();

        assert_eq!(sum.get(&[0]), "(v_0 + w_0)");
        assert_eq!(sum.get(&[1]), "(v_1 + w_1)");
    }

    #[test]
    fn test_symbolic_scale() {
        let v = SymbolicTensor::vector("v", 2);
        let scaled = v.scale("c");

        assert_eq!(scaled.get(&[0]), "(c*v_0)");
        assert_eq!(scaled.get(&[1]), "(c*v_1)");
    }

    #[test]
    fn test_symbolic_transpose() {
        let m = SymbolicTensor::matrix("A", 2, 3);
        let mt = m.transpose().unwrap();

        assert_eq!(mt.shape(), &[3, 2]);
        assert_eq!(mt.get(&[0, 0]), "A_0_0");
        assert_eq!(mt.get(&[1, 0]), "A_0_1");
        assert_eq!(mt.get(&[0, 1]), "A_1_0");
    }

    #[test]
    fn test_symbolic_trace() {
        let m = SymbolicTensor::matrix("A", 2, 2);
        let trace = m.trace().unwrap();

        assert_eq!(trace, "A_0_0 + A_1_1");
    }

    #[test]
    fn test_symbolic_zero_handling() {
        let mut v = SymbolicTensor::new(vec![2]);
        v.set(&[0], "x".to_string());
        // v[1] remains "0"

        let w = SymbolicTensor::vector("y", 2);
        let sum = v.add(&w).unwrap();

        assert_eq!(sum.get(&[0]), "(x + y_0)");
        assert_eq!(sum.get(&[1]), "y_1"); // 0 + y_1 = y_1
    }

    #[test]
    fn test_custom_components() {
        let comps = vec!["sin(x)".to_string(), "cos(x)".to_string()];
        let v = SymbolicTensor::from_components(vec![2], comps);

        assert_eq!(v.get(&[0]), "sin(x)");
        assert_eq!(v.get(&[1]), "cos(x)");
    }
}
