use bevy::prelude::*;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use suon_database::{AppTablesExt, Database, DatabaseMut, Table, Tables};

#[derive(Default)]
struct BenchTable {
    value: usize,
}

impl Table for BenchTable {}

fn benchmark_database(c: &mut Criterion) {
    let mut group = c.benchmark_group("database");

    group.bench_function("init_table", |b| {
        b.iter(|| {
            let mut app = App::new();
            app.init_database_table::<BenchTable>();
            black_box(
                app.world()
                    .get_resource::<Tables<BenchTable>>()
                    .expect("Table should exist after initialization")
                    .value,
            )
        })
    });

    group.bench_function("insert_table", |b| {
        b.iter(|| {
            let mut app = App::new();
            app.insert_database_table(BenchTable {
                value: black_box(42),
            });

            black_box(
                app.world()
                    .get_resource::<Tables<BenchTable>>()
                    .expect("Table should exist after insertion")
                    .value,
            )
        })
    });

    group.bench_function("system_param_read_write", |b| {
        b.iter(|| {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins);
            app.init_database_table::<BenchTable>();
            app.add_systems(Update, |mut table: DatabaseMut<BenchTable>| {
                table.value = black_box(64);
            });
            app.add_systems(PostUpdate, |table: Database<BenchTable>| {
                let _ = black_box(table.value);
            });
            app.update();
        })
    });

    for value in [1usize, 64usize, 4_096usize] {
        group.bench_with_input(
            BenchmarkId::new("overwrite_existing", value),
            &value,
            |b, &value| {
                b.iter(|| {
                    let mut app = App::new();
                    app.insert_database_table(BenchTable { value: 0 });
                    app.insert_database_table(BenchTable {
                        value: black_box(value),
                    });

                    black_box(
                        app.world()
                            .get_resource::<Tables<BenchTable>>()
                            .expect("Table should exist after overwrite")
                            .value,
                    )
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_database);
criterion_main!(benches);
