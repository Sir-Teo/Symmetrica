//! Matrix/linear algebra module: exact matrices over Q and fraction-free methods.
#![deny(warnings)]

use arith::{div_q, mul_q, sub_q, Q};

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
}
