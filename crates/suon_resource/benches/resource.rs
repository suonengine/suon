//! Benchmarks for the suon_resource crate.
//!
//! Measures resource container insert, read, and write throughput.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use suon_resource::{Resource, Resources};

struct Data(i32);
impl Resource for Data {}

#[derive(Default)]
struct Marker;
impl Resource for Marker {}

fn bench_init(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("resource");

    group.bench_function("init", |bencher| {
        bencher.iter(|| {
            let mut resources = Resources::default();
            black_box(resources.init::<Marker>());
        });
    });

    group.finish();
}

fn bench_insert(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("resource");

    group.bench_function("insert", |bencher| {
        bencher.iter(|| {
            let mut resources = Resources::default();
            black_box(resources.insert(Data(1)));
        });
    });

    group.finish();
}

fn bench_get(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("resource");

    let mut resources = Resources::default();
    resources.insert(Data(42));

    group.bench_function("get", |bencher| {
        bencher.iter(|| {
            black_box(resources.get::<Data>());
        });
    });

    group.finish();
}

fn bench_get_mut(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("resource");

    let mut resources = Resources::default();
    resources.insert(Data(0));

    group.bench_function("get_mut", |bencher| {
        bencher.iter(|| {
            let resource = resources.get_mut::<Data>();
            black_box(&mut resource.0);
        });
    });

    group.finish();
}

criterion_group!(
    name = resource_benchmarks;
    config = Criterion::default();
    targets =
        bench_init,
        bench_insert,
        bench_get,
        bench_get_mut,
);
criterion_main!(resource_benchmarks);
