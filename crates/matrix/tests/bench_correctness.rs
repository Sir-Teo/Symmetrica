//! Tests to verify correctness of benchmark operations
//! Ensures benchmark code paths are exercised and produce valid results

use arith::Q;
use matrix::MatrixQ;

// ========== Arithmetic Tests ==========

#[test]
fn test_matrix_add_correctness() {
    let a = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
    let b = MatrixQ::from_i64(2, 2, &[5, 6, 7, 8]);
    let c = a.add(&b).unwrap();

    assert_eq!(c.get(0, 0), Q(6, 1));
    assert_eq!(c.get(0, 1), Q(8, 1));
    assert_eq!(c.get(1, 0), Q(10, 1));
    assert_eq!(c.get(1, 1), Q(12, 1));
}

#[test]
fn test_matrix_sub_correctness() {
    let a = MatrixQ::from_i64(2, 2, &[10, 8, 6, 4]);
    let b = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
    let c = a.sub(&b).unwrap();

    assert_eq!(c.get(0, 0), Q(9, 1));
    assert_eq!(c.get(0, 1), Q(6, 1));
    assert_eq!(c.get(1, 0), Q(3, 1));
    assert_eq!(c.get(1, 1), Q(0, 1));
}

#[test]
fn test_matrix_mul_correctness() {
    // [[1,2],[3,4]] * [[5,6],[7,8]] = [[19,22],[43,50]]
    let a = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
    let b = MatrixQ::from_i64(2, 2, &[5, 6, 7, 8]);
    let c = a.mul(&b).unwrap();

    assert_eq!(c.get(0, 0), Q(19, 1));
    assert_eq!(c.get(0, 1), Q(22, 1));
    assert_eq!(c.get(1, 0), Q(43, 1));
    assert_eq!(c.get(1, 1), Q(50, 1));
}

#[test]
fn test_matrix_transpose_correctness() {
    let a = MatrixQ::from_i64(2, 3, &[1, 2, 3, 4, 5, 6]);
    let t = a.transpose();

    assert_eq!(t.rows, 3);
    assert_eq!(t.cols, 2);
    assert_eq!(t.get(0, 0), Q(1, 1));
    assert_eq!(t.get(0, 1), Q(4, 1));
    assert_eq!(t.get(1, 0), Q(2, 1));
    assert_eq!(t.get(1, 1), Q(5, 1));
    assert_eq!(t.get(2, 0), Q(3, 1));
    assert_eq!(t.get(2, 1), Q(6, 1));
}

#[test]
fn test_matrix_scalar_mul_correctness() {
    let a = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
    let s = a.scalar_mul(Q(3, 1));

    assert_eq!(s.get(0, 0), Q(3, 1));
    assert_eq!(s.get(0, 1), Q(6, 1));
    assert_eq!(s.get(1, 0), Q(9, 1));
    assert_eq!(s.get(1, 1), Q(12, 1));
}

#[test]
fn test_matrix_trace_correctness() {
    let a = MatrixQ::from_i64(3, 3, &[1, 2, 3, 4, 5, 6, 7, 8, 9]);
    let trace = a.trace().unwrap();

    // trace = 1 + 5 + 9 = 15
    assert_eq!(trace, Q(15, 1));
}

// ========== Determinant Tests ==========

#[test]
fn test_det_bareiss_2x2() {
    let m = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
    let det = m.det_bareiss().unwrap();

    // det = 1*4 - 2*3 = -2
    assert_eq!(det, Q(-2, 1));
}

#[test]
fn test_det_bareiss_3x3() {
    let m = MatrixQ::from_i64(3, 3, &[2, 0, 1, 1, 1, 0, 0, 3, 1]);
    let det = m.det_bareiss().unwrap();

    assert_eq!(det, Q(5, 1));
}

#[test]
fn test_det_identity() {
    let m = MatrixQ::identity(5);
    let det = m.det_bareiss().unwrap();

    assert_eq!(det, Q(1, 1));
}

#[test]
fn test_det_singular() {
    // Second row is 2 * first row
    let m = MatrixQ::from_i64(2, 2, &[1, 2, 2, 4]);
    let det = m.det_bareiss().unwrap();

    assert_eq!(det, Q(0, 1));
}

// ========== Solving Tests ==========

#[test]
fn test_solve_bareiss_2x2() {
    let m = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
    let b = vec![Q(5, 1), Q(11, 1)];
    let x = m.solve_bareiss(&b).unwrap().expect("has solution");

    assert_eq!(x.len(), 2);
    assert_eq!(x[0], Q(1, 1));
    assert_eq!(x[1], Q(2, 1));
}

#[test]
fn test_solve_bareiss_3x3() {
    let m = MatrixQ::from_i64(3, 3, &[2, 1, 0, 1, 3, 1, 0, 2, 1]);
    let b = vec![Q(5, 1), Q(10, 1), Q(7, 1)];
    let x = m.solve_bareiss(&b).unwrap().expect("has solution");

    assert_eq!(x.len(), 3);
}

#[test]
fn test_solve_lu_2x2() {
    let m = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
    let b = vec![Q(5, 1), Q(11, 1)];
    let x = m.solve_lu(&b).unwrap().expect("has solution");

    assert_eq!(x.len(), 2);
    assert_eq!(x[0], Q(1, 1));
    assert_eq!(x[1], Q(2, 1));
}

#[test]
fn test_solve_lu_identity() {
    let m = MatrixQ::identity(3);
    let b = vec![Q(1, 1), Q(2, 1), Q(3, 1)];
    let x = m.solve_lu(&b).unwrap().expect("has solution");

    assert_eq!(x, b);
}

#[test]
fn test_solve_singular_returns_none() {
    let m = MatrixQ::from_i64(2, 2, &[1, 2, 2, 4]);
    let b = vec![Q(1, 1), Q(2, 1)];

    assert!(m.solve_bareiss(&b).unwrap().is_none());
    assert!(m.solve_lu(&b).unwrap().is_none());
}

// ========== Decomposition Tests ==========

#[test]
fn test_lu_decompose_correctness() {
    let m = MatrixQ::from_i64(3, 3, &[2, 1, 0, 1, 3, 1, 0, 2, 1]);
    let (l, u, perm) = m.lu_decompose().unwrap();

    // L should be lower triangular with 1's on diagonal
    assert_eq!(l.get(0, 0), Q(1, 1));
    assert_eq!(l.get(1, 1), Q(1, 1));
    assert_eq!(l.get(2, 2), Q(1, 1));

    // Verify dimensions
    assert_eq!(l.rows, 3);
    assert_eq!(l.cols, 3);
    assert_eq!(u.rows, 3);
    assert_eq!(u.cols, 3);
    assert_eq!(perm.len(), 3);
}

#[test]
fn test_lu_decompose_identity() {
    let m = MatrixQ::identity(4);
    let (l, u, _) = m.lu_decompose().unwrap();

    // For identity, L = I and U = I
    assert_eq!(l, m);
    assert_eq!(u, m);
}

#[test]
fn test_inverse_2x2() {
    let m = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
    let inv = m.inverse().unwrap().expect("invertible");

    // Verify m * inv = I
    let product = m.mul(&inv).unwrap();
    let identity = MatrixQ::identity(2);

    for i in 0..2 {
        for j in 0..2 {
            assert_eq!(product.get(i, j), identity.get(i, j));
        }
    }
}

#[test]
fn test_inverse_3x3() {
    let m = MatrixQ::from_i64(3, 3, &[2, 1, 0, 1, 3, 1, 0, 2, 1]);
    let inv = m.inverse().unwrap().expect("invertible");

    // Verify m * inv = I
    let product = m.mul(&inv).unwrap();
    let identity = MatrixQ::identity(3);

    for i in 0..3 {
        for j in 0..3 {
            assert_eq!(product.get(i, j), identity.get(i, j));
        }
    }
}

#[test]
fn test_inverse_singular_returns_none() {
    let m = MatrixQ::from_i64(2, 2, &[1, 2, 2, 4]);
    assert!(m.inverse().unwrap().is_none());
}

// ========== Subspace Tests ==========

#[test]
fn test_rank_full_rank() {
    let m = MatrixQ::identity(5);
    assert_eq!(m.rank(), 5);
}

#[test]
fn test_rank_rank_deficient() {
    // Matrix with rank 2 (third row = first + second)
    let m = MatrixQ::from_i64(3, 3, &[1, 2, 3, 4, 5, 6, 5, 7, 9]);
    assert_eq!(m.rank(), 2);
}

#[test]
fn test_rank_zero_matrix() {
    let m = MatrixQ::from_i64(3, 3, &[0, 0, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(m.rank(), 0);
}

#[test]
fn test_nullspace_full_rank() {
    // Full rank matrix has trivial nullspace
    let m = MatrixQ::identity(3);
    let null = m.nullspace();

    assert_eq!(null.len(), 0);
}

#[test]
fn test_nullspace_rank_deficient() {
    // Matrix with nullspace dimension 1
    let m = MatrixQ::from_i64(2, 3, &[1, 2, 3, 2, 4, 6]);
    let null = m.nullspace();

    // Should have non-trivial nullspace
    assert!(!null.is_empty());
}

#[test]
fn test_columnspace_full_rank() {
    let m = MatrixQ::identity(3);
    let colspace = m.columnspace();

    // Full rank -> column space dimension = rank
    assert_eq!(colspace.len(), 3);
}

#[test]
fn test_columnspace_rank_deficient() {
    // Rank 2 matrix
    let m = MatrixQ::from_i64(3, 3, &[1, 2, 3, 4, 5, 6, 5, 7, 9]);
    let colspace = m.columnspace();

    assert_eq!(colspace.len(), 2);
}

// ========== Combined Operations Tests ==========

#[test]
fn test_matrix_vector_product() {
    let m = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
    let v = MatrixQ::new(2, 1, vec![Q(5, 1), Q(6, 1)]);
    let result = m.mul(&v).unwrap();

    // [1,2] * [5]   [17]
    // [3,4]   [6] = [39]
    assert_eq!(result.get(0, 0), Q(17, 1));
    assert_eq!(result.get(1, 0), Q(39, 1));
}

#[test]
fn test_matrix_power() {
    let m = MatrixQ::from_i64(2, 2, &[1, 1, 0, 1]);

    // Compute m^2
    let m2 = m.mul(&m).unwrap();
    assert_eq!(m2.get(0, 0), Q(1, 1));
    assert_eq!(m2.get(0, 1), Q(2, 1));
    assert_eq!(m2.get(1, 0), Q(0, 1));
    assert_eq!(m2.get(1, 1), Q(1, 1));

    // Compute m^3 = m^2 * m
    let m3 = m2.mul(&m).unwrap();
    assert_eq!(m3.get(0, 0), Q(1, 1));
    assert_eq!(m3.get(0, 1), Q(3, 1));
    assert_eq!(m3.get(1, 0), Q(0, 1));
    assert_eq!(m3.get(1, 1), Q(1, 1));
}

#[test]
fn test_solve_multiple_rhs() {
    let m = MatrixQ::from_i64(2, 2, &[2, 1, 1, 2]);

    let b1 = vec![Q(3, 1), Q(3, 1)];
    let b2 = vec![Q(5, 1), Q(4, 1)];

    let x1 = m.solve_lu(&b1).unwrap().expect("has solution");
    let x2 = m.solve_lu(&b2).unwrap().expect("has solution");

    assert_eq!(x1.len(), 2);
    assert_eq!(x2.len(), 2);
}

// ========== Edge Cases ==========

#[test]
fn test_empty_matrix_operations() {
    let m = MatrixQ::new(0, 0, vec![]);

    assert_eq!(m.det_bareiss().unwrap(), Q(1, 1));
    assert_eq!(m.rank(), 0);
    assert_eq!(m.nullspace().len(), 0);
    assert_eq!(m.columnspace().len(), 0);
}

#[test]
fn test_large_matrix_add() {
    let size = 20usize;
    let data1: Vec<i64> = (1..=(size * size) as i64).collect();
    let data2: Vec<i64> = ((size * size) as i64 + 1..=(2 * size * size) as i64).collect();

    let m1 = MatrixQ::from_i64(size, size, &data1);
    let m2 = MatrixQ::from_i64(size, size, &data2);

    let result = m1.add(&m2).unwrap();
    assert_eq!(result.rows, size);
    assert_eq!(result.cols, size);
}

#[test]
fn test_rectangular_matrix_mul() {
    // (2x3) * (3x2) = (2x2)
    let a = MatrixQ::from_i64(2, 3, &[1, 2, 3, 4, 5, 6]);
    let b = MatrixQ::from_i64(3, 2, &[1, 2, 3, 4, 5, 6]);

    let c = a.mul(&b).unwrap();
    assert_eq!(c.rows, 2);
    assert_eq!(c.cols, 2);
}

#[test]
fn test_rational_entries() {
    let a = MatrixQ::new(2, 2, vec![Q(1, 2), Q(1, 3), Q(1, 4), Q(1, 5)]);
    let b = MatrixQ::new(2, 2, vec![Q(1, 2), Q(2, 3), Q(3, 4), Q(4, 5)]);

    let c = a.add(&b).unwrap();
    assert_eq!(c.get(0, 0), Q(1, 1)); // 1/2 + 1/2 = 1
    assert_eq!(c.get(0, 1), Q(1, 1)); // 1/3 + 2/3 = 1
}

#[test]
fn test_transpose_idempotent() {
    let m = MatrixQ::from_i64(3, 3, &[1, 2, 3, 4, 5, 6, 7, 8, 9]);
    let t = m.transpose();
    let tt = t.transpose();

    // Transposing twice should give original
    assert_eq!(m, tt);
}
