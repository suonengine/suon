use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use suon_macros::Task;

use suon_channel::{Channel, TaskHandler};
use suon_resource::Resources;

const DRAIN_SIZES: &[usize] = &[
    1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536,
];

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
                black_box(());
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn bench_wait_and_drain(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("channel");

    for &count in DRAIN_SIZES {
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
                    channel.wait_and_drain(&mut buffer);
                    black_box(());
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
                channel.wait_and_drain(&mut buffer);
                black_box(());
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
        bench_wait_and_drain,
        bench_send_and_drain,
);
criterion_main!(channel_benchmarks);
