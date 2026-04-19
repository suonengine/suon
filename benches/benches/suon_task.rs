use ::benches::bench;
use bevy::prelude::*;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use suon_task::prelude::*;

struct BenchTask;

impl BackgroundTask for BenchTask {
    type Output = ();

    async fn run(self) -> Self::Output {}
}

fn benchmark_task(c: &mut Criterion) {
    let mut group = c.benchmark_group("task");

    group.bench_function(bench!("add_background_systems/update"), |b| {
        b.iter(|| {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins);
            app.add_background_task_systems::<Update, BenchTask>();
            app
        })
    });

    for schedule_name in ["update", "post_update"] {
        group.bench_with_input(
            BenchmarkId::new(bench!("add_background_systems/schedule"), schedule_name),
            &schedule_name,
            |b, schedule_name| {
                b.iter(|| {
                    let mut app = App::new();
                    app.add_plugins(MinimalPlugins);

                    match *schedule_name {
                        "update" => {
                            app.add_background_task_systems::<Update, BenchTask>();
                        }
                        "post_update" => {
                            app.add_background_task_systems::<PostUpdate, BenchTask>();
                        }
                        _ => unreachable!("Unexpected schedule benchmark input"),
                    }

                    app
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_task);
criterion_main!(benches);
