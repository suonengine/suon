use bevy::prelude::*;
use criterion::{Criterion, criterion_group, criterion_main};
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

criterion_group!(benches, benchmark_add_background_task_systems);
criterion_main!(benches);
