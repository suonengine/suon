use ::benches::bench;
use bevy::prelude::*;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use suon_chunk::prelude::*;
use suon_position::prelude::*;

fn benchmark_chunk_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("chunk");

    for positions in [256usize, 4_096usize, 16_384usize] {
        let chunks = Chunks::from_iter((0..positions).map(|index| {
            (
                Position {
                    x: index as u16,
                    y: index as u16,
                },
                Entity::from_bits((index + 1) as u64),
            )
        }));

        group.bench_with_input(
            BenchmarkId::new(bench!("get"), positions),
            &positions,
            |b, _| {
                let mut index = 0usize;
                b.iter(|| {
                    let position = Position {
                        x: (index % positions) as u16,
                        y: (index % positions) as u16,
                    };
                    index = index.wrapping_add(1);
                    chunks.get(black_box(&position))
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new(bench!("contains"), positions),
            &positions,
            |b, _| {
                let mut index = 0usize;
                b.iter(|| {
                    let position = Position {
                        x: (index % positions) as u16,
                        y: (index % positions) as u16,
                    };
                    index = index.wrapping_add(1);
                    chunks.contains(black_box(&position))
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new(bench!("from_iter"), positions),
            &positions,
            |b, &size| {
                b.iter(|| {
                    Chunks::from_iter((0..size).map(|index| {
                        (
                            Position {
                                x: index as u16,
                                y: index as u16,
                            },
                            Entity::from_bits((index + 1) as u64),
                        )
                    }))
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_chunk_lookup);
criterion_main!(benches);
