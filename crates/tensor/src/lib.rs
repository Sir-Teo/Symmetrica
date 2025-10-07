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
        Self {
            shape,
            strides,
            data: vec![fill; size],
        }
    }

    pub fn from_vec(shape: Vec<usize>, data: Vec<T>) -> Self {
        let size: usize = shape.iter().product();
        assert_eq!(size, data.len(), "data length does not match shape");
        let strides = compute_strides(&shape);
        Self { shape, strides, data }
    }

    pub fn rank(&self) -> usize { self.shape.len() }
    pub fn len(&self) -> usize { self.data.len() }
    pub fn is_empty(&self) -> bool { self.data.is_empty() }
    pub fn shape(&self) -> &[usize] { &self.shape }

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
}
