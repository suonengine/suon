use bevy::prelude::*;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use suon_task::background::{AppWithBackgroundTasks, BackgroundTask};

struct BenchTask;

impl BackgroundTask for BenchTask {
    type Output = ();

    async fn run(self) -> Self::Output {}
}

fn benchmark_add_background_task_systems(c: &mut Criterion) {
    c.bench_function("task/add_background_systems", |b| {
        b.iter(|| {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins);
            app.add_background_task_systems::<Update, BenchTask>();
            app
        })
    });
}

fn benchmark_add_background_task_systems_multiple_schedules(c: &mut Criterion) {
    let mut group = c.benchmark_group("task/add_background_systems");

    for schedule_name in ["update", "post_update"] {
        group.bench_with_input(
            BenchmarkId::new("schedule", schedule_name),
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

criterion_group!(
    benches,
    benchmark_add_background_task_systems,
    benchmark_add_background_task_systems_multiple_schedules
);
criterion_main!(benches);
