//! Benchmarks for matrix operations (Phase L)
//!
//! Tests performance of:
//! - Matrix arithmetic (add, sub, mul, transpose, scalar_mul)
//! - Determinant computation (Bareiss algorithm)
//! - Linear system solving (Bareiss and LU methods)
//! - Matrix decompositions (LU, inverse)
//! - Subspace computations (rank, nullspace, columnspace)

use arith::Q;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use matrix::MatrixQ;

// ========== Matrix Arithmetic ==========

pub fn bench_matrix_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("matrix_add");
    for &size in &[5usize, 10, 20, 50] {
        group.throughput(Throughput::Elements((size * size) as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &n| {
            // Create two nxn matrices with integer entries
            let data1: Vec<i64> = (1..=(n * n) as i64).collect();
            let data2: Vec<i64> = ((n * n) as i64..=(2 * n * n) as i64).collect();
            let m1 = MatrixQ::from_i64(n, n, &data1);
            let m2 = MatrixQ::from_i64(n, n, &data2);

            b.iter(|| {
                let _result = black_box(&m1).add(black_box(&m2)).unwrap();
            });
        });
    }
    group.finish();
}

pub fn bench_matrix_sub(c: &mut Criterion) {
    let mut group = c.benchmark_group("matrix_sub");
    for &size in &[5usize, 10, 20, 50] {
        group.throughput(Throughput::Elements((size * size) as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &n| {
            let data1: Vec<i64> = ((n * n) as i64..=(2 * n * n) as i64).collect();
            let data2: Vec<i64> = (1..=(n * n) as i64).collect();
            let m1 = MatrixQ::from_i64(n, n, &data1);
            let m2 = MatrixQ::from_i64(n, n, &data2);

            b.iter(|| {
                let _result = black_box(&m1).sub(black_box(&m2)).unwrap();
            });
        });
    }
    group.finish();
}

pub fn bench_matrix_mul(c: &mut Criterion) {
    let mut group = c.benchmark_group("matrix_mul");
    for &size in &[5usize, 10, 20, 30] {
        group.throughput(Throughput::Elements((size * size * size) as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &n| {
            let data1: Vec<i64> = (1..=(n * n) as i64).collect();
            let data2: Vec<i64> = ((n * n) as i64 + 1..=(2 * n * n) as i64).collect();
            let m1 = MatrixQ::from_i64(n, n, &data1);
            let m2 = MatrixQ::from_i64(n, n, &data2);

            b.iter(|| {
                let _result = black_box(&m1).mul(black_box(&m2)).unwrap();
            });
        });
    }
    group.finish();
}

pub fn bench_matrix_transpose(c: &mut Criterion) {
    let mut group = c.benchmark_group("matrix_transpose");
    for &size in &[10usize, 20, 50, 100] {
        group.throughput(Throughput::Elements((size * size) as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &n| {
            let data: Vec<i64> = (1..=(n * n) as i64).collect();
            let m = MatrixQ::from_i64(n, n, &data);

            b.iter(|| {
                let _result = black_box(&m).transpose();
            });
        });
    }
    group.finish();
}

pub fn bench_matrix_scalar_mul(c: &mut Criterion) {
    let mut group = c.benchmark_group("matrix_scalar_mul");
    for &size in &[10usize, 20, 50, 100] {
        group.throughput(Throughput::Elements((size * size) as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &n| {
            let data: Vec<i64> = (1..=(n * n) as i64).collect();
            let m = MatrixQ::from_i64(n, n, &data);
            let scalar = Q(3, 2);

            b.iter(|| {
                let _result = black_box(&m).scalar_mul(black_box(scalar));
            });
        });
    }
    group.finish();
}

pub fn bench_matrix_trace(c: &mut Criterion) {
    let mut group = c.benchmark_group("matrix_trace");
    for &size in &[10usize, 50, 100, 200] {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &n| {
            let data: Vec<i64> = (1..=(n * n) as i64).collect();
            let m = MatrixQ::from_i64(n, n, &data);

            b.iter(|| {
                let _result = black_box(&m).trace().unwrap();
            });
        });
    }
    group.finish();
}

// ========== Determinant Computation ==========

pub fn bench_det_bareiss(c: &mut Criterion) {
    let mut group = c.benchmark_group("det_bareiss");
    for &size in &[3usize, 5, 7, 10, 15] {
        group.throughput(Throughput::Elements((size * size) as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &n| {
            // Create a well-conditioned matrix
            let mut data = vec![0i64; n * n];
            for i in 0..n {
                for j in 0..n {
                    data[i * n + j] = (i + j + 1) as i64;
                }
            }
            let m = MatrixQ::from_i64(n, n, &data);

            b.iter(|| {
                let _det = black_box(&m).det_bareiss().unwrap();
            });
        });
    }
    group.finish();
}

pub fn bench_det_identity(c: &mut Criterion) {
    c.bench_function("det_identity_20x20", |b| {
        let m = MatrixQ::identity(20);
        b.iter(|| {
            let _det = black_box(&m).det_bareiss().unwrap();
        });
    });
}

// ========== Linear System Solving ==========

pub fn bench_solve_bareiss(c: &mut Criterion) {
    let mut group = c.benchmark_group("solve_bareiss");
    for &size in &[3usize, 5, 7, 10] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &n| {
            // Create a well-conditioned system
            let mut data = vec![0i64; n * n];
            for i in 0..n {
                for j in 0..n {
                    data[i * n + j] = if i == j { 2 } else { 1 };
                }
            }
            let m = MatrixQ::from_i64(n, n, &data);
            let rhs: Vec<Q> = (1..=n).map(|i| Q(i as i64, 1)).collect();

            b.iter(|| {
                let _x = black_box(&m).solve_bareiss(black_box(&rhs)).unwrap();
            });
        });
    }
    group.finish();
}

pub fn bench_solve_lu(c: &mut Criterion) {
    let mut group = c.benchmark_group("solve_lu");
    for &size in &[3usize, 5, 7, 10, 15] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &n| {
            let mut data = vec![0i64; n * n];
            for i in 0..n {
                for j in 0..n {
                    data[i * n + j] = if i == j { 2 } else { 1 };
                }
            }
            let m = MatrixQ::from_i64(n, n, &data);
            let rhs: Vec<Q> = (1..=n).map(|i| Q(i as i64, 1)).collect();

            b.iter(|| {
                let _x = black_box(&m).solve_lu(black_box(&rhs)).unwrap();
            });
        });
    }
    group.finish();
}

// ========== Matrix Decompositions ==========

pub fn bench_lu_decompose(c: &mut Criterion) {
    let mut group = c.benchmark_group("lu_decompose");
    for &size in &[5usize, 10, 15, 20] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &n| {
            let mut data = vec![0i64; n * n];
            for i in 0..n {
                for j in 0..n {
                    data[i * n + j] = ((i + 1) * (j + 1)) as i64;
                }
            }
            let m = MatrixQ::from_i64(n, n, &data);

            b.iter(|| {
                let _decomp = black_box(&m).lu_decompose().unwrap();
            });
        });
    }
    group.finish();
}

pub fn bench_inverse(c: &mut Criterion) {
    let mut group = c.benchmark_group("inverse");
    for &size in &[3usize, 5, 7, 10] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &n| {
            // Create invertible matrix
            let mut data = vec![0i64; n * n];
            for i in 0..n {
                for j in 0..n {
                    data[i * n + j] = if i == j { n as i64 } else { 1 };
                }
            }
            let m = MatrixQ::from_i64(n, n, &data);

            b.iter(|| {
                let _inv = black_box(&m).inverse().unwrap();
            });
        });
    }
    group.finish();
}

// ========== Subspace Computations ==========

pub fn bench_rank(c: &mut Criterion) {
    let mut group = c.benchmark_group("rank");
    for &size in &[5usize, 10, 15, 20] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &n| {
            let data: Vec<i64> = (1..=(n * n) as i64).collect();
            let m = MatrixQ::from_i64(n, n, &data);

            b.iter(|| {
                let _r = black_box(&m).rank();
            });
        });
    }
    group.finish();
}

pub fn bench_nullspace(c: &mut Criterion) {
    let mut group = c.benchmark_group("nullspace");

    // Test with matrices that have nontrivial nullspace
    for &size in &[5usize, 8, 10] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &n| {
            // Create a matrix with rank n-1 (has 1D nullspace)
            let mut data = vec![0i64; n * n];
            for i in 0..n {
                for j in 0..n {
                    if i < n - 1 {
                        data[i * n + j] = (i + j + 1) as i64;
                    } else {
                        // Last row is zero
                        data[i * n + j] = 0;
                    }
                }
            }
            let m = MatrixQ::from_i64(n, n, &data);

            b.iter(|| {
                let _null = black_box(&m).nullspace();
            });
        });
    }
    group.finish();
}

pub fn bench_columnspace(c: &mut Criterion) {
    let mut group = c.benchmark_group("columnspace");
    for &size in &[5usize, 8, 10, 15] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &n| {
            let data: Vec<i64> = (1..=(n * n) as i64).collect();
            let m = MatrixQ::from_i64(n, n, &data);

            b.iter(|| {
                let _colspace = black_box(&m).columnspace();
            });
        });
    }
    group.finish();
}

// ========== Combined Operations ==========

pub fn bench_matrix_vector_product(c: &mut Criterion) {
    c.bench_function("matrix_vector_20x20", |b| {
        let data: Vec<i64> = (1..=400).collect();
        let m = MatrixQ::from_i64(20, 20, &data);
        let v_data: Vec<Q> = (1..=20).map(|i| Q(i, 1)).collect();
        let v = MatrixQ::new(20, 1, v_data);

        b.iter(|| {
            let _result = black_box(&m).mul(black_box(&v)).unwrap();
        });
    });
}

pub fn bench_matrix_power(c: &mut Criterion) {
    c.bench_function("matrix_power_10x10_cubed", |b| {
        let data: Vec<i64> = (1..=100).collect();
        let m = MatrixQ::from_i64(10, 10, &data);

        b.iter(|| {
            // Compute m^3
            let m2 = black_box(&m).mul(black_box(&m)).unwrap();
            let _m3 = m2.mul(black_box(&m)).unwrap();
        });
    });
}

pub fn bench_solve_multiple_rhs(c: &mut Criterion) {
    c.bench_function("solve_5x5_three_rhs", |b| {
        let mut data = vec![0i64; 25];
        for i in 0..5 {
            for j in 0..5 {
                data[i * 5 + j] = if i == j { 3 } else { 1 };
            }
        }
        let m = MatrixQ::from_i64(5, 5, &data);

        let rhs1: Vec<Q> = vec![Q(1, 1), Q(2, 1), Q(3, 1), Q(4, 1), Q(5, 1)];
        let rhs2: Vec<Q> = vec![Q(5, 1), Q(4, 1), Q(3, 1), Q(2, 1), Q(1, 1)];
        let rhs3: Vec<Q> = vec![Q(1, 1), Q(1, 1), Q(1, 1), Q(1, 1), Q(1, 1)];

        b.iter(|| {
            let _x1 = black_box(&m).solve_lu(black_box(&rhs1)).unwrap();
            let _x2 = black_box(&m).solve_lu(black_box(&rhs2)).unwrap();
            let _x3 = black_box(&m).solve_lu(black_box(&rhs3)).unwrap();
        });
    });
}

// ========== Criterion Configuration ==========

criterion_group!(
    arithmetic,
    bench_matrix_add,
    bench_matrix_sub,
    bench_matrix_mul,
    bench_matrix_transpose,
    bench_matrix_scalar_mul,
    bench_matrix_trace
);

criterion_group!(determinant, bench_det_bareiss, bench_det_identity);

criterion_group!(solving, bench_solve_bareiss, bench_solve_lu);

criterion_group!(decomposition, bench_lu_decompose, bench_inverse);

criterion_group!(subspace, bench_rank, bench_nullspace, bench_columnspace);

criterion_group!(
    combined,
    bench_matrix_vector_product,
    bench_matrix_power,
    bench_solve_multiple_rhs
);

criterion_main!(arithmetic, determinant, solving, decomposition, subspace, combined);
