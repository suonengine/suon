//! Benchmarks for the suon_app crate.
//!
//! Measures application startup, startup system execution, and task
//! dispatch throughput.

use criterion::{Criterion, criterion_group, criterion_main};
use suon_app::{App, shutdown::Shutdown};
use suon_channel::{Channel, TaskHandler};
use suon_macros::Task;
use suon_resource::Resources;

#[derive(Task)]
struct NoOp;

impl TaskHandler for NoOp {
    fn run(self: Box<Self>, _: &mut Resources) {}
}

fn bench_empty_shutdown(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("app");

    group.bench_function("empty_shutdown", |bencher| {
        bencher.iter(|| {
            let mut app = App::new();
            app.channel().send(Shutdown);
            app.run();
        });
    });

    group.finish();
}

fn bench_startup_system(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("app");

    group.bench_function("startup_system", |bencher| {
        bencher.iter(|| {
            let mut app = App::new();
            app.add_startup_system(|resources: &mut Resources| {
                resources.get::<Channel>().send(Shutdown);
            });
            app.run();
        });
    });

    group.finish();
}

fn bench_task_dispatch(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("app");

    group.bench_function("task_dispatch_100", |bencher| {
        bencher.iter(|| {
            let mut app = App::new();
            app.add_startup_system(|resources: &mut Resources| {
                let channel = resources.get::<Channel>();
                for _ in 0..100 {
                    channel.send(NoOp);
                }
                channel.send(Shutdown);
            });
            app.run();
        });
    });

    group.finish();
}

criterion_group!(
    name = app_benchmarks;
    config = Criterion::default();
    targets =
        bench_empty_shutdown,
        bench_startup_system,
        bench_task_dispatch,
);
criterion_main!(app_benchmarks);
