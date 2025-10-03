use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use expr_core::Store;
use simplify::simplify;

pub fn bench_build_atoms(c: &mut Criterion) {
    let mut group = c.benchmark_group("build_atoms");
    for &n in &[10_000usize, 50_000, 100_000] {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                let mut st = Store::new();
                for i in 0..n {
                    let _ = st.sym(format!("x{i}"));
                }
            });
        });
    }
    group.finish();
}

pub fn bench_add_chain(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_chain");
    for &n in &[1_000usize, 5_000, 10_000] {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                let mut st = Store::new();
                let mut terms = Vec::with_capacity(n);
                for i in 0..n as i64 {
                    terms.push(st.int(i));
                }
                let _sum = st.add(terms);
            });
        });
    }
    group.finish();
}

pub fn bench_simplify_collect(c: &mut Criterion) {
    c.bench_function("simplify_collect_small", |b| {
        b.iter(|| {
            let mut st = Store::new();
            let x = st.sym("x");
            let two = st.int(2);
            let two_x = st.mul(vec![two, x]);
            let three = st.int(3);
            let three_x = st.mul(vec![three, x]);
            let half = st.rat(1, 2);
            let half_x = st.mul(vec![half, x]);
            let expr = st.add(vec![two_x, three_x, half_x, half]);
            let _s = simplify(&mut st, expr);
        })
    });
}

criterion_group!(benches, bench_build_atoms, bench_add_chain, bench_simplify_collect);
criterion_main!(benches);
