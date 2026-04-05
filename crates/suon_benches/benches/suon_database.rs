use bevy::prelude::*;
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use suon_database::{AppTablesExt, Table, Tables};

#[derive(Default)]
struct BenchTable {
    value: usize,
}

impl Table for BenchTable {}

fn benchmark_database(c: &mut Criterion) {
    c.bench_function("database/init_insert_get", |b| {
        b.iter(|| {
            let mut app = App::new();
            app.init_database_table::<BenchTable>();
            app.insert_database_table(BenchTable {
                value: black_box(42),
            });

            app.world()
                .get_resource::<Tables<BenchTable>>()
                .expect("Table should exist")
                .value
        })
    });
}

criterion_group!(benches, benchmark_database);
criterion_main!(benches);
