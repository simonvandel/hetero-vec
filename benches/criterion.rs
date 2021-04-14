use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hetero_vec::*;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("hvec 20", |b| {
        b.iter(|| {
            let mut hvec = HVec::new();
            hvec.push(black_box(20));
            hvec.push(black_box(20f64))
        })
    });

    c.bench_function("vec 20", |b| {
        b.iter(|| {
            let mut vec = Vec::new();
            vec.push(black_box(20));
            let mut vec = Vec::new();
            vec.push(black_box(20f64))
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
