use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use suon_macros::Task;

use suon_channel::{Channel, TaskHandler};
use suon_resource::Resources;

#[derive(Task)]
struct NoOp;

impl TaskHandler for NoOp {
    fn run(self: Box<Self>, _: &mut Resources) {}
}

fn bench_send(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("channel");

    group.bench_function("send", |bencher| {
        bencher.iter_batched(
            Channel::default,
            |channel| {
                channel.send(NoOp);
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn bench_drain_into(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("channel");

    for &count in &[1, 8, 64, 512, 4096] {
        group.bench_function(format!("drain_{count}"), |bencher| {
            bencher.iter_batched(
                || {
                    let channel = Channel::default();
                    for _ in 0..count {
                        channel.send(NoOp);
                    }
                    (channel, Vec::with_capacity(count))
                },
                |(channel, mut buffer)| {
                    channel.drain_into(&mut buffer);
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_send_and_drain(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("channel");

    group.bench_function("send_and_drain", |bencher| {
        bencher.iter_batched(
            || (Channel::default(), Vec::new()),
            |(channel, mut buffer)| {
                channel.send(NoOp);
                channel.drain_into(&mut buffer);
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(
    name = channel_benchmarks;
    config = Criterion::default();
    targets =
        bench_send,
        bench_drain_into,
        bench_send_and_drain,
);
criterion_main!(channel_benchmarks);
