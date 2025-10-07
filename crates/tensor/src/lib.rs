#![deny(warnings)]

use std::ops::{Add, Mul};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tensor<T> {
    shape: Vec<usize>,
    strides: Vec<usize>,
    data: Vec<T>,
}

impl<T: Clone> Tensor<T> {
    pub fn new(shape: Vec<usize>, fill: T) -> Self {
        assert!(!shape.is_empty(), "tensor must have rank >= 1");
        let size = shape.iter().product();
        let strides = compute_strides(&shape);
        Self { shape, strides, data: vec![fill; size] }
    }

    pub fn from_vec(shape: Vec<usize>, data: Vec<T>) -> Self {
        let size: usize = shape.iter().product();
        assert_eq!(size, data.len(), "data length does not match shape");
        let strides = compute_strides(&shape);
        Self { shape, strides, data }
    }

    pub fn rank(&self) -> usize {
        self.shape.len()
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    pub fn shape(&self) -> &[usize] {
        &self.shape
    }

    fn offset(&self, idx: &[usize]) -> usize {
        assert_eq!(idx.len(), self.shape.len(), "index rank mismatch");
        let mut off = 0usize;
        for (i, &v) in idx.iter().enumerate() {
            assert!(v < self.shape[i], "index out of bounds");
            off += v * self.strides[i];
        }
        off
    }

    pub fn get(&self, idx: &[usize]) -> &T {
        let o = self.offset(idx);
        &self.data[o]
    }

    pub fn set(&mut self, idx: &[usize], val: T) {
        let o = self.offset(idx);
        self.data[o] = val;
    }

    /// Return a new tensor with a different shape but the same data layout.
    pub fn reshape(&self, new_shape: Vec<usize>) -> Self {
        assert!(!new_shape.is_empty(), "new shape must have rank >= 1");
        let old: usize = self.shape.iter().product();
        let newp: usize = new_shape.iter().product();
        assert_eq!(old, newp, "reshape must keep total size");
        let strides = compute_strides(&new_shape);
        Self { shape: new_shape, strides, data: self.data.clone() }
    }

    /// Permute axes in the given order, e.g. `[1,0]` swaps two axes.
    pub fn permute_axes(&self, perm: &[usize]) -> Self {
        assert_eq!(perm.len(), self.rank(), "perm length must equal rank");
        // Validate permutation
        let mut seen = vec![false; perm.len()];
        for &p in perm {
            assert!(p < perm.len(), "perm index out of range");
            assert!(!seen[p], "perm has duplicates");
            seen[p] = true;
        }
        // New shape
        let mut new_shape = vec![0usize; perm.len()];
        for i in 0..perm.len() {
            new_shape[i] = self.shape[perm[i]];
        }
        let out_size: usize = new_shape.iter().product();
        let mut out_data = Vec::with_capacity(out_size);
        for flat in 0..out_size {
            let new_idx = unflatten_index(flat, &new_shape);
            // Build original index: old_axis = perm[new_axis]
            let mut old_idx = vec![0usize; perm.len()];
            for (new_axis, &new_val) in new_idx.iter().enumerate() {
                let old_axis = perm[new_axis];
                old_idx[old_axis] = new_val;
            }
            out_data.push(self.get(&old_idx).clone());
        }
        Tensor::from_vec(new_shape, out_data)
    }
}

impl<T> Tensor<T>
where
    T: Clone + Mul<Output = T>,
{
    pub fn outer(&self, other: &Tensor<T>) -> Tensor<T> {
        let mut shape = self.shape.clone();
        shape.extend_from_slice(&other.shape);
        let mut data = Vec::with_capacity(self.len() * other.len());
        for a in &self.data {
            for b in &other.data {
                data.push(a.clone() * b.clone());
            }
        }
        Tensor::from_vec(shape, data)
    }

    /// Elementwise multiplication; shapes must match.
    pub fn elem_mul(&self, other: &Tensor<T>) -> Tensor<T> {
        assert_eq!(self.shape, other.shape, "shape mismatch");
        let data =
            self.data.iter().zip(other.data.iter()).map(|(a, b)| a.clone() * b.clone()).collect();
        Tensor::from_vec(self.shape.clone(), data)
    }
}

impl<T> Tensor<T>
where
    T: Clone + Default + Add<Output = T> + Mul<Output = T>,
{
    /// Contract along `axis_a` of self and `axis_b` of other (tensordot with one axis).
    pub fn contract(&self, other: &Tensor<T>, axis_a: usize, axis_b: usize) -> Tensor<T> {
        assert!(axis_a < self.rank() && axis_b < other.rank(), "axis out of range");
        let k = self.shape[axis_a];
        assert_eq!(k, other.shape[axis_b], "contracted dimensions must match");

        // Build output shape
        let mut out_shape = Vec::new();
        out_shape.extend_from_slice(&self.shape[..axis_a]);
        out_shape.extend_from_slice(&self.shape[axis_a + 1..]);
        out_shape.extend_from_slice(&other.shape[..axis_b]);
        out_shape.extend_from_slice(&other.shape[axis_b + 1..]);
        if out_shape.is_empty() {
            // represent scalar as [1] to avoid rank-0 corner cases
            out_shape.push(1);
        }
        let out_size: usize = out_shape.iter().product();
        let mut out_data = Vec::with_capacity(out_size);

        let a_pre = axis_a;
        let a_post = self.rank() - axis_a - 1;
        let b_pre = axis_b;
        let b_post = other.rank() - axis_b - 1;

        for flat in 0..out_size {
            let idx = unflatten_index(flat, &out_shape);
            let mut p = 0usize;
            let a_idx_pre = &idx[p..p + a_pre];
            p += a_pre;
            let a_idx_post = &idx[p..p + a_post];
            p += a_post;
            let b_idx_pre = &idx[p..p + b_pre];
            p += b_pre;
            let b_idx_post = &idx[p..p + b_post];

            let mut sum = T::default();
            for kk in 0..k {
                let mut full_a = Vec::with_capacity(self.rank());
                full_a.extend_from_slice(a_idx_pre);
                full_a.push(kk);
                full_a.extend_from_slice(a_idx_post);

                let mut full_b = Vec::with_capacity(other.rank());
                full_b.extend_from_slice(b_idx_pre);
                full_b.push(kk);
                full_b.extend_from_slice(b_idx_post);

                let prod = self.get(&full_a).clone() * other.get(&full_b).clone();
                sum = sum + prod;
            }
            out_data.push(sum);
        }

        Tensor::from_vec(out_shape, out_data)
    }

    /// Elementwise addition; shapes must match.
    pub fn elem_add(&self, other: &Tensor<T>) -> Tensor<T> {
        assert_eq!(self.shape, other.shape, "shape mismatch");
        let data =
            self.data.iter().zip(other.data.iter()).map(|(a, b)| a.clone() + b.clone()).collect();
        Tensor::from_vec(self.shape.clone(), data)
    }

    /// Sum across an axis, reducing rank by 1.
    pub fn sum_axis(&self, axis: usize) -> Tensor<T> {
        assert!(axis < self.rank(), "axis out of range");
        let k = self.shape[axis];
        let mut out_shape = Vec::new();
        out_shape.extend_from_slice(&self.shape[..axis]);
        out_shape.extend_from_slice(&self.shape[axis + 1..]);
        if out_shape.is_empty() {
            out_shape.push(1);
        }
        let out_size: usize = out_shape.iter().product();
        let mut out_data = vec![T::default(); out_size];
        for (flat, slot) in out_data.iter_mut().enumerate() {
            let idx = unflatten_index(flat, &out_shape);
            let pre = axis;
            let post = self.rank() - axis - 1;
            let a_idx_pre = &idx[..pre];
            let a_idx_post = &idx[pre..(pre + post)];
            let mut sum = T::default();
            for kk in 0..k {
                let mut full = Vec::with_capacity(self.rank());
                full.extend_from_slice(a_idx_pre);
                full.push(kk);
                full.extend_from_slice(a_idx_post);
                sum = sum + self.get(&full).clone();
            }
            *slot = sum;
        }
        Tensor::from_vec(out_shape, out_data)
    }

    /// Dot product for 1-D tensors (vectors).
    pub fn dot(&self, other: &Tensor<T>) -> T {
        assert_eq!(self.shape, other.shape, "shape mismatch");
        assert_eq!(self.rank(), 1, "dot expects rank-1 tensors");
        let mut sum = T::default();
        for (a, b) in self.data.iter().zip(other.data.iter()) {
            sum = sum + a.clone() * b.clone();
        }
        sum
    }

    /// Transpose of a rank-2 tensor (matrix), swapping axes 0 and 1.
    pub fn transpose2(&self) -> Tensor<T> {
        assert_eq!(self.rank(), 2, "transpose2 expects rank-2 tensor");
        self.permute_axes(&[1, 0])
    }

    /// Matrix multiplication for rank-2 tensors via contraction.
    pub fn matmul(&self, other: &Tensor<T>) -> Tensor<T> {
        assert_eq!(self.rank(), 2, "matmul expects rank-2 tensors");
        assert_eq!(other.rank(), 2, "matmul expects rank-2 tensors");
        assert_eq!(self.shape[1], other.shape[0], "inner dimensions must match");
        self.contract(other, 1, 0)
    }

    /// Trace along two axes with equal dimensions (sums diagonal elements over those axes).
    /// For a matrix, use `trace_pair(0, 1)` which yields a scalar represented as shape `[1]`.
    pub fn trace_pair(&self, axis1: usize, axis2: usize) -> Tensor<T> {
        assert!(axis1 < self.rank() && axis2 < self.rank(), "axis out of range");
        assert!(axis1 != axis2, "axes must be distinct");
        assert_eq!(self.shape[axis1], self.shape[axis2], "traced axes must have equal length");

        // Keep axes other than axis1, axis2
        let keep: Vec<usize> = (0..self.rank()).filter(|&a| a != axis1 && a != axis2).collect();
        let mut out_shape: Vec<usize> = keep.iter().map(|&a| self.shape[a]).collect();
        if out_shape.is_empty() {
            out_shape.push(1);
        }
        let out_size: usize = out_shape.iter().product();
        let mut out_data = vec![T::default(); out_size];

        let kdim = self.shape[axis1];
        for (flat, slot) in out_data.iter_mut().enumerate() {
            let idx_keep = unflatten_index(flat, &out_shape);
            // Build full index template
            let mut full = vec![0usize; self.rank()];
            for (i, &ax) in keep.iter().enumerate() {
                full[ax] = idx_keep[i];
            }
            let mut sum = T::default();
            for kk in 0..kdim {
                full[axis1] = kk;
                full[axis2] = kk;
                sum = sum + self.get(&full).clone();
            }
            *slot = sum;
        }
        Tensor::from_vec(out_shape, out_data)
    }
}

fn compute_strides(shape: &[usize]) -> Vec<usize> {
    let mut strides = vec![0; shape.len()];
    let mut acc = 1usize;
    for i in (0..shape.len()).rev() {
        strides[i] = acc;
        acc *= shape[i];
    }
    strides
}

fn unflatten_index(mut i: usize, shape: &[usize]) -> Vec<usize> {
    let mut idx = vec![0; shape.len()];
    for d in (0..shape.len()).rev() {
        let s = shape[d];
        idx[d] = i % s;
        i /= s;
    }
    idx
}

#[cfg(test)]
mod tests {
    use super::Tensor;

    #[test]
    fn indexing_set_get() {
        let mut t = Tensor::new(vec![2, 3], 0i64);
        t.set(&[0, 0], 1);
        t.set(&[1, 2], 7);
        assert_eq!(*t.get(&[0, 0]), 1);
        assert_eq!(*t.get(&[1, 2]), 7);
    }

    #[test]
    fn outer_product() {
        let a = Tensor::from_vec(vec![2], vec![1i64, 2]);
        let b = Tensor::from_vec(vec![3], vec![3i64, 4, 5]);
        let o = a.outer(&b);
        assert_eq!(o.shape(), &[2, 3]);
        assert_eq!(o, Tensor::from_vec(vec![2, 3], vec![3, 4, 5, 6, 8, 10]));
    }

    #[test]
    fn contract_matrix_multiplication() {
        // A: [2x3]
        let a = Tensor::from_vec(vec![2, 3], vec![1i64, 2, 3, 4, 5, 6]);
        // B: [3x2]
        let b = Tensor::from_vec(vec![3, 2], vec![7i64, 8, 9, 10, 11, 12]);
        // Contract A axis=1 with B axis=0 -> [2x2]
        let c = a.contract(&b, 1, 0);
        assert_eq!(c.shape(), &[2, 2]);
        assert_eq!(c, Tensor::from_vec(vec![2, 2], vec![58, 64, 139, 154]));
    }

    #[test]
    fn reshape_and_permute() {
        let t = Tensor::from_vec(vec![2, 3], vec![1i64, 2, 3, 4, 5, 6]);
        let tr = t.reshape(vec![3, 2]);
        assert_eq!(tr.shape(), &[3, 2]);
        // Permute axes [1,0] on original is a transpose-like view with data reorder
        let tp = t.permute_axes(&[1, 0]);
        assert_eq!(tp.shape(), &[3, 2]);
        assert_eq!(tp.get(&[0, 0]), &1);
        assert_eq!(tp.get(&[2, 1]), &6);
    }

    #[test]
    fn elem_add_mul_and_sum() {
        let a = Tensor::from_vec(vec![2, 2], vec![1i64, 2, 3, 4]);
        let b = Tensor::from_vec(vec![2, 2], vec![5i64, 6, 7, 8]);
        let add = a.elem_add(&b);
        assert_eq!(add, Tensor::from_vec(vec![2, 2], vec![6, 8, 10, 12]));
        let mul = a.elem_mul(&b);
        assert_eq!(mul, Tensor::from_vec(vec![2, 2], vec![5, 12, 21, 32]));
        let sum0 = a.sum_axis(0);
        assert_eq!(sum0, Tensor::from_vec(vec![2], vec![4, 6]));
        let sum1 = a.sum_axis(1);
        assert_eq!(sum1, Tensor::from_vec(vec![2], vec![3, 7]));
    }

    #[test]
    fn dot_product() {
        let a = Tensor::from_vec(vec![3], vec![1i64, 2, 3]);
        let b = Tensor::from_vec(vec![3], vec![4i64, 5, 6]);
        let d = a.dot(&b);
        assert_eq!(d, 32);
    }

    #[test]
    fn transpose_and_matmul() {
        // A: [2x3], B: [3x2]
        let a = Tensor::from_vec(vec![2, 3], vec![1i64, 2, 3, 4, 5, 6]);
        let _bt = Tensor::from_vec(vec![2, 3], vec![7i64, 9, 11, 8, 10, 12]).transpose2();
        // Reconstruct B (3x2)
        let b = Tensor::from_vec(vec![3, 2], vec![7i64, 8, 9, 10, 11, 12]);
        let c1 = a.matmul(&b);
        let c2 = a.contract(&b, 1, 0);
        assert_eq!(c1, c2);
        // Transpose of transpose yields original
        let a_tt = a.transpose2().transpose2();
        assert_eq!(a_tt, a);
    }

    #[test]
    fn trace_matrix() {
        let m = Tensor::from_vec(vec![2, 2], vec![1i64, 2, 3, 4]);
        let t = m.trace_pair(0, 1);
        // Scalar represented as [1]
        assert_eq!(t.shape(), &[1]);
        assert_eq!(*t.get(&[0]), 1 + 4);
    }
}
