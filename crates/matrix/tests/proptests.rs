//! Property-based tests for matrix

use arith::{div_q, mul_q, Q};
use matrix::MatrixQ;
use proptest::prelude::*;

fn small_q() -> impl Strategy<Value = Q> {
    (-5i64..=5).prop_map(|n| Q::new(n, 1))
}

fn diag_matrix(diag: &[Q]) -> MatrixQ {
    let n = diag.len();
    let mut data = vec![Q::zero(); n * n];
    for i in 0..n {
        data[i * n + i] = diag[i];
    }
    MatrixQ::new(n, n, data)
}

proptest! {
    #[test]
    fn prop_det_of_diagonal_equals_product(diag in prop::collection::vec(small_q(), 1..=6)) {
        let m = diag_matrix(&diag);
        let det = m.det_bareiss().expect("square");
        let prod = diag.iter().copied().fold(Q::one(), mul_q);
        prop_assert_eq!(det, prod);
    }

    #[test]
    fn prop_solve_diagonal(n in 1usize..=5) {
        let diag: Vec<Q> = (0..n).map(|i| Q::new((i as i64 % 5) + 1, 1)).collect();
        let b: Vec<Q> = (0..n).map(|i| Q::new(i as i64 - 2, 1)).collect();
        let m = diag_matrix(&diag);
        let sol = m.solve_bareiss(&b).expect("ok").expect("unique");
        for i in 0..n {
            let expected = div_q(b[i], diag[i]);
            prop_assert_eq!(sol[i], expected);
        }
    }
}
