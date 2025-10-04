//! Matrix/linear algebra module: exact matrices over Q and fraction-free methods.
#![deny(warnings)]

use arith::{add_q, div_q, mul_q, sub_q, Q};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatrixQ {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<Q>, // row-major
}

impl MatrixQ {
    pub fn new(rows: usize, cols: usize, data: Vec<Q>) -> Self {
        assert_eq!(data.len(), rows * cols, "data size mismatch");
        Self { rows, cols, data }
    }
    pub fn from_i64(rows: usize, cols: usize, data: &[i64]) -> Self {
        assert_eq!(data.len(), rows * cols);
        let v = data.iter().map(|&k| Q(k, 1)).collect();
        Self::new(rows, cols, v)
    }
    pub fn identity(n: usize) -> Self {
        let mut v = vec![Q::zero(); n * n];
        for i in 0..n {
            v[i * n + i] = Q::one();
        }
        Self::new(n, n, v)
    }
    #[inline]
    fn idx(&self, r: usize, c: usize) -> usize {
        r * self.cols + c
    }
    pub fn get(&self, r: usize, c: usize) -> Q {
        self.data[self.idx(r, c)]
    }
    pub fn set(&mut self, r: usize, c: usize, v: Q) {
        let i = self.idx(r, c);
        self.data[i] = v;
    }

    /// Compute determinant using the Bareiss fraction-free algorithm.
    /// Returns 0 for singular matrices. Requires square matrix.
    pub fn det_bareiss(&self) -> Result<Q, &'static str> {
        if self.rows != self.cols {
            return Err("determinant requires square matrix");
        }
        let n = self.rows;
        if n == 0 {
            return Ok(Q::one());
        }
        // Rational Gaussian elimination with partial pivoting
        let mut a = self.clone();
        let mut sign = Q::one();
        for k in 0..n {
            // pivot
            let mut pr = k;
            while pr < n && a.get(pr, k).is_zero() {
                pr += 1;
            }
            if pr == n {
                return Ok(Q::zero());
            }
            if pr != k {
                for c in 0..n {
                    let t = a.get(k, c);
                    a.set(k, c, a.get(pr, c));
                    a.set(pr, c, t);
                }
                sign = mul_q(sign, Q(-1, 1));
            }
            // eliminate below
            let akk = a.get(k, k);
            for i in k + 1..n {
                let aik = a.get(i, k);
                if aik.is_zero() {
                    continue;
                }
                let factor = div_q(aik, akk);
                for j in k..n {
                    let val = sub_q(a.get(i, j), mul_q(factor, a.get(k, j)));
                    a.set(i, j, val);
                }
                a.set(i, k, Q::zero());
            }
        }
        // determinant is sign * product of diagonal
        let mut det = sign;
        for i in 0..n {
            det = mul_q(det, a.get(i, i));
        }
        Ok(det)
    }

    /// Solve A x = b using fraction-free Bareiss elimination.
    /// Returns Ok(Some(x)) if unique solution exists; Ok(None) if singular; Err on misuse.
    #[allow(clippy::needless_range_loop)]
    pub fn solve_bareiss(&self, b: &[Q]) -> Result<Option<Vec<Q>>, &'static str> {
        if self.rows != self.cols {
            return Err("solve requires square matrix");
        }
        let n = self.rows;
        if b.len() != n {
            return Err("rhs length must equal number of rows");
        }
        if n == 0 {
            return Ok(Some(vec![]));
        }
        // Cramer's rule using determinant; suitable for our small test sizes
        let det_a = self.det_bareiss()?;
        if det_a.is_zero() {
            return Ok(None);
        }
        let mut x = vec![Q::zero(); n];
        for col in 0..n {
            let mut a_col = self.clone();
            for (r, &br) in b.iter().enumerate() {
                a_col.set(r, col, br);
            }
            let det_i = a_col.det_bareiss()?;
            x[col] = div_q(det_i, det_a);
        }
        Ok(Some(x))
    }

    /// Add two matrices element-wise. Returns Err if dimensions mismatch.
    pub fn add(&self, other: &MatrixQ) -> Result<MatrixQ, &'static str> {
        if self.rows != other.rows || self.cols != other.cols {
            return Err("matrix dimensions must match for addition");
        }
        let data = self.data.iter().zip(&other.data).map(|(&a, &b)| add_q(a, b)).collect();
        Ok(MatrixQ::new(self.rows, self.cols, data))
    }

    /// Subtract two matrices element-wise. Returns Err if dimensions mismatch.
    pub fn sub(&self, other: &MatrixQ) -> Result<MatrixQ, &'static str> {
        if self.rows != other.rows || self.cols != other.cols {
            return Err("matrix dimensions must match for subtraction");
        }
        let data = self.data.iter().zip(&other.data).map(|(&a, &b)| sub_q(a, b)).collect();
        Ok(MatrixQ::new(self.rows, self.cols, data))
    }

    /// Multiply two matrices. Returns Err if dimensions are incompatible (self.cols != other.rows).
    pub fn mul(&self, other: &MatrixQ) -> Result<MatrixQ, &'static str> {
        if self.cols != other.rows {
            return Err("incompatible dimensions for matrix multiplication");
        }
        let m = self.rows;
        let n = other.cols;
        let p = self.cols;
        let mut data = vec![Q::zero(); m * n];
        for i in 0..m {
            for j in 0..n {
                let mut sum = Q::zero();
                for k in 0..p {
                    sum = add_q(sum, mul_q(self.get(i, k), other.get(k, j)));
                }
                data[i * n + j] = sum;
            }
        }
        Ok(MatrixQ::new(m, n, data))
    }

    /// Compute the inverse of a square matrix using Gauss-Jordan elimination.
    /// Returns Ok(Some(A^-1)) if invertible; Ok(None) if singular; Err if not square.
    pub fn inverse(&self) -> Result<Option<MatrixQ>, &'static str> {
        if self.rows != self.cols {
            return Err("inverse requires square matrix");
        }
        let n = self.rows;
        if n == 0 {
            return Ok(Some(MatrixQ::new(0, 0, vec![])));
        }

        // Check if matrix is singular first
        let det = self.det_bareiss()?;
        if det.is_zero() {
            return Ok(None);
        }

        // Create augmented matrix [A | I]
        let mut aug = MatrixQ::new(n, 2 * n, vec![Q::zero(); n * 2 * n]);
        for i in 0..n {
            for j in 0..n {
                aug.set(i, j, self.get(i, j));
                if i == j {
                    aug.set(i, j + n, Q::one());
                } else {
                    aug.set(i, j + n, Q::zero());
                }
            }
        }

        // Gauss-Jordan elimination
        for col in 0..n {
            // Find pivot
            let mut pivot_row = col;
            while pivot_row < n && aug.get(pivot_row, col).is_zero() {
                pivot_row += 1;
            }
            if pivot_row == n {
                return Ok(None); // Singular
            }

            // Swap rows if needed
            if pivot_row != col {
                for j in 0..2 * n {
                    let temp = aug.get(col, j);
                    aug.set(col, j, aug.get(pivot_row, j));
                    aug.set(pivot_row, j, temp);
                }
            }

            // Scale pivot row to make pivot = 1
            let pivot = aug.get(col, col);
            for j in 0..2 * n {
                let val = div_q(aug.get(col, j), pivot);
                aug.set(col, j, val);
            }

            // Eliminate column in all other rows
            for i in 0..n {
                if i == col {
                    continue;
                }
                let factor = aug.get(i, col);
                if factor.is_zero() {
                    continue;
                }
                for j in 0..2 * n {
                    let val = sub_q(aug.get(i, j), mul_q(factor, aug.get(col, j)));
                    aug.set(i, j, val);
                }
            }
        }

        // Extract the inverse from the right half
        let mut inv_data = vec![Q::zero(); n * n];
        for i in 0..n {
            for j in 0..n {
                inv_data[i * n + j] = aug.get(i, j + n);
            }
        }
        Ok(Some(MatrixQ::new(n, n, inv_data)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn det_2x2() {
        let m = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
        assert_eq!(m.det_bareiss().unwrap(), Q(-2, 1));
    }

    #[test]
    fn det_identity() {
        let m = MatrixQ::identity(4);
        assert_eq!(m.det_bareiss().unwrap(), Q(1, 1));
    }

    #[test]
    fn det_3x3_example() {
        // [[2,0,1],[1,1,0],[0,3,1]] -> det = 5
        let m = MatrixQ::from_i64(3, 3, &[2, 0, 1, 1, 1, 0, 0, 3, 1]);
        assert_eq!(m.det_bareiss().unwrap(), Q(5, 1));
    }

    #[test]
    fn det_singular() {
        // second row is multiple of first
        let m = MatrixQ::from_i64(2, 2, &[1, 2, 2, 4]);
        assert_eq!(m.det_bareiss().unwrap(), Q(0, 1));
    }

    #[test]
    fn solve_2x2_unique() {
        // [ [1,2], [3,4] ] x = [5,11] -> x = [1,2]
        let m = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
        let b = vec![Q(5, 1), Q(11, 1)];
        let x = m.solve_bareiss(&b).unwrap().expect("unique");
        assert_eq!(x, vec![Q(1, 1), Q(2, 1)]);
    }

    #[test]
    fn solve_3x3_unique() {
        // A = [[2,1,0],[1,3,1],[0,2,1]]; b=[5,10,7] -> x=[2,1,1]
        let m = MatrixQ::from_i64(3, 3, &[2, 1, 0, 1, 3, 1, 0, 2, 1]);
        let b = vec![Q(5, 1), Q(10, 1), Q(7, 1)];
        let x = m.solve_bareiss(&b).unwrap().expect("unique");
        assert_eq!(x, vec![Q(2, 1), Q(1, 1), Q(5, 1)]);
    }

    #[test]
    fn solve_singular_none() {
        let m = MatrixQ::from_i64(2, 2, &[1, 2, 2, 4]);
        let b = vec![Q(3, 1), Q(6, 1)];
        assert!(m.solve_bareiss(&b).unwrap().is_none());
    }

    #[test]
    fn det_non_square_error() {
        let m = MatrixQ::from_i64(2, 3, &[1, 2, 3, 4, 5, 6]);
        assert!(m.det_bareiss().is_err());
    }

    #[test]
    fn solve_non_square_error() {
        let m = MatrixQ::from_i64(2, 3, &[1, 2, 3, 4, 5, 6]);
        let b = vec![Q(1, 1), Q(2, 1)];
        assert!(m.solve_bareiss(&b).is_err());
    }

    #[test]
    fn solve_wrong_rhs_length() {
        let m = MatrixQ::identity(2);
        let b = vec![Q(1, 1)];
        assert!(m.solve_bareiss(&b).is_err());
    }

    #[test]
    fn det_zero_size() {
        let m = MatrixQ::new(0, 0, vec![]);
        assert_eq!(m.det_bareiss().unwrap(), Q(1, 1));
    }

    #[test]
    fn solve_zero_size() {
        let m = MatrixQ::new(0, 0, vec![]);
        let b = vec![];
        let x = m.solve_bareiss(&b).unwrap().expect("empty");
        assert_eq!(x.len(), 0);
    }

    // ========== Matrix Addition Tests ==========
    #[test]
    fn add_2x2() {
        let a = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
        let b = MatrixQ::from_i64(2, 2, &[5, 6, 7, 8]);
        let c = a.add(&b).unwrap();
        assert_eq!(c.get(0, 0), Q(6, 1));
        assert_eq!(c.get(0, 1), Q(8, 1));
        assert_eq!(c.get(1, 0), Q(10, 1));
        assert_eq!(c.get(1, 1), Q(12, 1));
    }

    #[test]
    fn add_different_sizes_error() {
        let a = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
        let b = MatrixQ::from_i64(2, 3, &[1, 2, 3, 4, 5, 6]);
        assert!(a.add(&b).is_err());
    }

    #[test]
    fn add_with_fractions() {
        let a = MatrixQ::new(2, 2, vec![Q(1, 2), Q(1, 3), Q(1, 4), Q(1, 5)]);
        let b = MatrixQ::new(2, 2, vec![Q(1, 2), Q(2, 3), Q(3, 4), Q(4, 5)]);
        let c = a.add(&b).unwrap();
        assert_eq!(c.get(0, 0), Q(1, 1)); // 1/2 + 1/2 = 1
        assert_eq!(c.get(0, 1), Q(1, 1)); // 1/3 + 2/3 = 1
        assert_eq!(c.get(1, 0), Q(1, 1)); // 1/4 + 3/4 = 1
        assert_eq!(c.get(1, 1), Q(1, 1)); // 1/5 + 4/5 = 1
    }

    // ========== Matrix Subtraction Tests ==========
    #[test]
    fn sub_2x2() {
        let a = MatrixQ::from_i64(2, 2, &[5, 6, 7, 8]);
        let b = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
        let c = a.sub(&b).unwrap();
        assert_eq!(c.get(0, 0), Q(4, 1));
        assert_eq!(c.get(0, 1), Q(4, 1));
        assert_eq!(c.get(1, 0), Q(4, 1));
        assert_eq!(c.get(1, 1), Q(4, 1));
    }

    #[test]
    fn sub_different_sizes_error() {
        let a = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
        let b = MatrixQ::from_i64(3, 2, &[1, 2, 3, 4, 5, 6]);
        assert!(a.sub(&b).is_err());
    }

    #[test]
    fn sub_to_zero() {
        let a = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
        let c = a.sub(&a).unwrap();
        assert_eq!(c.get(0, 0), Q(0, 1));
        assert_eq!(c.get(0, 1), Q(0, 1));
        assert_eq!(c.get(1, 0), Q(0, 1));
        assert_eq!(c.get(1, 1), Q(0, 1));
    }

    // ========== Matrix Multiplication Tests ==========
    #[test]
    fn mul_2x2() {
        let a = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
        let b = MatrixQ::from_i64(2, 2, &[5, 6, 7, 8]);
        let c = a.mul(&b).unwrap();
        // [[1,2],[3,4]] * [[5,6],[7,8]] = [[19,22],[43,50]]
        assert_eq!(c.get(0, 0), Q(19, 1));
        assert_eq!(c.get(0, 1), Q(22, 1));
        assert_eq!(c.get(1, 0), Q(43, 1));
        assert_eq!(c.get(1, 1), Q(50, 1));
    }

    #[test]
    fn mul_identity() {
        let a = MatrixQ::from_i64(3, 3, &[1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let i = MatrixQ::identity(3);
        let c = a.mul(&i).unwrap();
        assert_eq!(c, a);
    }

    #[test]
    fn mul_incompatible_dimensions_error() {
        let a = MatrixQ::from_i64(2, 3, &[1, 2, 3, 4, 5, 6]);
        let b = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
        assert!(a.mul(&b).is_err());
    }

    #[test]
    fn mul_rectangular() {
        // (2x3) * (3x2) = (2x2)
        let a = MatrixQ::from_i64(2, 3, &[1, 2, 3, 4, 5, 6]);
        let b = MatrixQ::from_i64(3, 2, &[1, 2, 3, 4, 5, 6]);
        let c = a.mul(&b).unwrap();
        assert_eq!(c.rows, 2);
        assert_eq!(c.cols, 2);
        // [[1,2,3],[4,5,6]] * [[1,2],[3,4],[5,6]]
        // = [[1+6+15, 2+8+18],[4+15+30, 8+20+36]]
        // = [[22,28],[49,64]]
        assert_eq!(c.get(0, 0), Q(22, 1));
        assert_eq!(c.get(0, 1), Q(28, 1));
        assert_eq!(c.get(1, 0), Q(49, 1));
        assert_eq!(c.get(1, 1), Q(64, 1));
    }

    // ========== Matrix Inverse Tests ==========
    #[test]
    fn inverse_2x2() {
        // [[1,2],[3,4]] has inverse [[-2,1],[3/2,-1/2]]
        let a = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
        let inv = a.inverse().unwrap().expect("invertible");
        assert_eq!(inv.get(0, 0), Q(-2, 1));
        assert_eq!(inv.get(0, 1), Q(1, 1));
        assert_eq!(inv.get(1, 0), Q(3, 2));
        assert_eq!(inv.get(1, 1), Q(-1, 2));

        // Verify A * A^-1 = I
        let product = a.mul(&inv).unwrap();
        let identity = MatrixQ::identity(2);
        assert_eq!(product, identity);
    }

    #[test]
    fn inverse_3x3() {
        // [[2,1,0],[1,3,1],[0,2,1]]
        let a = MatrixQ::from_i64(3, 3, &[2, 1, 0, 1, 3, 1, 0, 2, 1]);
        let inv = a.inverse().unwrap().expect("invertible");

        // Verify A * A^-1 = I
        let product = a.mul(&inv).unwrap();
        let identity = MatrixQ::identity(3);
        assert_eq!(product, identity);
    }

    #[test]
    fn inverse_singular_none() {
        let a = MatrixQ::from_i64(2, 2, &[1, 2, 2, 4]);
        let result = a.inverse().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn inverse_non_square_error() {
        let a = MatrixQ::from_i64(2, 3, &[1, 2, 3, 4, 5, 6]);
        assert!(a.inverse().is_err());
    }

    #[test]
    fn inverse_identity() {
        let i = MatrixQ::identity(4);
        let inv = i.inverse().unwrap().expect("invertible");
        assert_eq!(inv, i);
    }

    #[test]
    fn inverse_zero_size() {
        let a = MatrixQ::new(0, 0, vec![]);
        let inv = a.inverse().unwrap().expect("invertible");
        assert_eq!(inv.rows, 0);
        assert_eq!(inv.cols, 0);
    }

    #[test]
    fn inverse_then_solve() {
        // Test that solving via inverse gives same result as solve_bareiss
        let a = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
        let b = vec![Q(5, 1), Q(11, 1)];

        let x1 = a.solve_bareiss(&b).unwrap().expect("unique");
        let inv = a.inverse().unwrap().expect("invertible");

        // Compute x2 = A^-1 * b
        let b_mat = MatrixQ::new(2, 1, b.clone());
        let x2_mat = inv.mul(&b_mat).unwrap();
        let x2 = vec![x2_mat.get(0, 0), x2_mat.get(1, 0)];

        assert_eq!(x1, x2);
    }
}
