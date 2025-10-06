use criterion::{criterion_group, criterion_main, Criterion};
use expr_core::Store;
use summation::{pochhammer, sum_arithmetic, sum_geometric, sum_power};

fn bench_arithmetic_sum(c: &mut Criterion) {
    c.bench_function("arithmetic_sum_1_to_n", |b| {
        b.iter(|| {
            let mut st = Store::new();
            let k = st.sym("k");
            let one = st.int(1);
            let n = st.sym("n");
            let zero = st.int(0);
            sum_arithmetic(&mut st, k, one, n, zero, one)
        })
    });
}

fn bench_geometric_sum(c: &mut Criterion) {
    c.bench_function("geometric_sum_powers_of_2", |b| {
        b.iter(|| {
            let mut st = Store::new();
            let k = st.sym("k");
            let zero = st.int(0);
            let n = st.sym("n");
            let two = st.int(2);
            let term = st.pow(two, k);
            sum_geometric(&mut st, term, "k", zero, n, two)
        })
    });
}

fn bench_power_sum(c: &mut Criterion) {
    c.bench_function("power_sum_squares", |b| {
        b.iter(|| {
            let mut st = Store::new();
            let one = st.int(1);
            let n = st.sym("n");
            let two = st.int(2);
            sum_power(&mut st, "k", one, n, two)
        })
    });
}

fn bench_pochhammer(c: &mut Criterion) {
    c.bench_function("pochhammer_rising_factorial", |b| {
        b.iter(|| {
            let mut st = Store::new();
            let x = st.sym("x");
            let five = st.int(5);
            pochhammer(&mut st, x, five)
        })
    });
}

fn bench_pochhammer_large(c: &mut Criterion) {
    c.bench_function("pochhammer_10", |b| {
        b.iter(|| {
            let mut st = Store::new();
            let one = st.int(1);
            let ten = st.int(10);
            pochhammer(&mut st, one, ten)
        })
    });
}

criterion_group!(
    benches,
    bench_arithmetic_sum,
    bench_geometric_sum,
    bench_power_sum,
    bench_pochhammer,
    bench_pochhammer_large
);
criterion_main!(benches);
