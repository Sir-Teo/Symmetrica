//! Einstein Summation Notation (einsum)
//!
//! Implements generalized tensor contraction using Einstein notation.
//! Example: "ij,jk->ik" represents matrix multiplication

use crate::Tensor;
use std::ops::{Add, Mul};

/// Parse Einstein summation notation string
/// Format: "input1,input2,...->output"
/// Example: "ij,jk->ik" for matrix multiplication
fn parse_einsum_spec(spec: &str) -> Result<(Vec<String>, String), String> {
    let parts: Vec<&str> = spec.split("->").collect();
    if parts.len() != 2 {
        return Err("Invalid einsum spec: must contain exactly one '->'".to_string());
    }

    let inputs: Vec<String> = parts[0].split(',').map(|s| s.trim().to_string()).collect();
    let output = parts[1].trim().to_string();

    Ok((inputs, output))
}

/// Einstein summation for two tensors
/// This is a simplified implementation for common cases
pub fn einsum<T>(spec: &str, a: &Tensor<T>, b: &Tensor<T>) -> Result<Tensor<T>, String>
where
    T: Clone + Default + Add<Output = T> + Mul<Output = T>,
{
    let (inputs, output) = parse_einsum_spec(spec)?;

    if inputs.len() != 2 {
        return Err("Currently only supports two input tensors".to_string());
    }

    let spec_a = &inputs[0];
    let spec_b = &inputs[1];

    // Validate input specs match tensor ranks
    if spec_a.len() != a.rank() {
        return Err(format!(
            "Input spec '{}' length doesn't match tensor rank {}",
            spec_a,
            a.rank()
        ));
    }
    if spec_b.len() != b.rank() {
        return Err(format!(
            "Input spec '{}' length doesn't match tensor rank {}",
            spec_b,
            b.rank()
        ));
    }

    // Find contracted indices (appear in both inputs but not in output)
    let mut contracted_indices = Vec::new();
    for (i, c) in spec_a.chars().enumerate() {
        if spec_b.contains(c) && !output.contains(c) {
            // Find position in spec_b
            if let Some(j) = spec_b.chars().position(|x| x == c) {
                contracted_indices.push((i, j, c));
            }
        }
    }

    // Handle common patterns
    match (spec_a.as_str(), spec_b.as_str(), output.as_str()) {
        // Matrix multiplication: "ij,jk->ik"
        ("ij", "jk", "ik") => {
            if a.rank() == 2 && b.rank() == 2 {
                return Ok(a.matmul(b));
            }
        }
        // Dot product: "i,i->"
        ("i", "i", "") => {
            if a.rank() == 1 && b.rank() == 1 {
                let result = a.dot(b);
                return Ok(Tensor::from_vec(vec![1], vec![result]));
            }
        }
        // Outer product: "i,j->ij"
        ("i", "j", "ij") => {
            if a.rank() == 1 && b.rank() == 1 {
                return Ok(a.outer(b));
            }
        }
        // Element-wise multiplication: "ij,ij->ij"
        _ if spec_a == spec_b && spec_a == &output => {
            return Ok(a.elem_mul(b));
        }
        _ => {}
    }

    // General contraction (simplified)
    if contracted_indices.len() == 1 {
        let (axis_a, axis_b, _) = contracted_indices[0];
        return Ok(a.contract(b, axis_a, axis_b));
    }

    Err(format!("Unsupported einsum pattern: {}", spec))
}

/// Einstein summation for a single tensor (trace, transpose, etc.)
pub fn einsum_single<T>(spec: &str, a: &Tensor<T>) -> Result<Tensor<T>, String>
where
    T: Clone + Default + Add<Output = T> + Mul<Output = T>,
{
    let (inputs, output) = parse_einsum_spec(spec)?;

    if inputs.len() != 1 {
        return Err("einsum_single requires exactly one input".to_string());
    }

    let spec_a = &inputs[0];

    // Validate
    if spec_a.len() != a.rank() {
        return Err(format!(
            "Input spec '{}' length doesn't match tensor rank {}",
            spec_a,
            a.rank()
        ));
    }

    // Handle common patterns
    match (spec_a.as_str(), output.as_str()) {
        // Trace: "ii->"
        ("ii", "") if a.rank() == 2 => {
            let trace_result = a.trace_pair(0, 1);
            Ok(trace_result)
        }
        // Transpose: "ij->ji"
        ("ij", "ji") if a.rank() == 2 => Ok(a.transpose2()),
        // Permutation: general case
        _ => {
            // Build permutation from spec
            let mut perm = vec![0; a.rank()];
            for (out_idx, out_char) in output.chars().enumerate() {
                if let Some(in_idx) = spec_a.chars().position(|c| c == out_char) {
                    perm[out_idx] = in_idx;
                } else {
                    return Err(format!("Output index '{}' not in input", out_char));
                }
            }
            Ok(a.permute_axes(&perm))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_einsum_matrix_multiply() {
        // Matrix multiplication: C_ik = A_ij * B_jk
        let a = Tensor::from_vec(vec![2, 3], vec![1i64, 2, 3, 4, 5, 6]);
        let b = Tensor::from_vec(vec![3, 2], vec![7i64, 8, 9, 10, 11, 12]);

        let c = einsum("ij,jk->ik", &a, &b).unwrap();
        assert_eq!(c.shape(), &[2, 2]);

        // Verify against matmul
        let expected = a.matmul(&b);
        assert_eq!(c, expected);
    }

    #[test]
    fn test_einsum_dot_product() {
        let a = Tensor::from_vec(vec![3], vec![1i64, 2, 3]);
        let b = Tensor::from_vec(vec![3], vec![4i64, 5, 6]);

        let c = einsum("i,i->", &a, &b).unwrap();
        assert_eq!(c.shape(), &[1]);
        assert_eq!(*c.get(&[0]), 32); // 1*4 + 2*5 + 3*6 = 32
    }

    #[test]
    fn test_einsum_outer_product() {
        let a = Tensor::from_vec(vec![2], vec![1i64, 2]);
        let b = Tensor::from_vec(vec![3], vec![3i64, 4, 5]);

        let c = einsum("i,j->ij", &a, &b).unwrap();
        assert_eq!(c.shape(), &[2, 3]);
        assert_eq!(*c.get(&[0, 0]), 3);
        assert_eq!(*c.get(&[1, 2]), 10);
    }

    #[test]
    fn test_einsum_single_transpose() {
        let a = Tensor::from_vec(vec![2, 3], vec![1i64, 2, 3, 4, 5, 6]);

        let b = einsum_single("ij->ji", &a).unwrap();
        assert_eq!(b.shape(), &[3, 2]);
        assert_eq!(*b.get(&[0, 0]), 1);
        assert_eq!(*b.get(&[2, 1]), 6);
    }

    #[test]
    fn test_einsum_single_trace() {
        let a = Tensor::from_vec(vec![2, 2], vec![1i64, 2, 3, 4]);

        let b = einsum_single("ii->", &a).unwrap();
        assert_eq!(b.shape(), &[1]);
        assert_eq!(*b.get(&[0]), 5); // trace = 1 + 4
    }

    #[test]
    fn test_einsum_element_wise() {
        let a = Tensor::from_vec(vec![2, 2], vec![1i64, 2, 3, 4]);
        let b = Tensor::from_vec(vec![2, 2], vec![5i64, 6, 7, 8]);

        let c = einsum("ij,ij->ij", &a, &b).unwrap();
        assert_eq!(c.shape(), &[2, 2]);
        assert_eq!(*c.get(&[0, 0]), 5);
        assert_eq!(*c.get(&[1, 1]), 32);
    }
}
