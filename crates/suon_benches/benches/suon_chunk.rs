use bevy::prelude::Entity;
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use suon_chunk::chunks::Chunks;
use suon_position::position::Position;

fn benchmark_chunk_lookup(c: &mut Criterion) {
    const POSITIONS: usize = 256;

    let chunks = Chunks::from_iter((0..POSITIONS).map(|index| {
        (
            Position {
                x: index as u16,
                y: index as u16,
            },
            Entity::from_bits((index + 1) as u64),
        )
    }));

    c.bench_function("chunk/get_lookup", |b| {
        let mut index = 0usize;
        b.iter(|| {
            let position = Position {
                x: (index % POSITIONS) as u16,
                y: (index % POSITIONS) as u16,
            };
            index = index.wrapping_add(1);
            chunks.get(black_box(&position))
        })
    });
}

criterion_group!(benches, benchmark_chunk_lookup);
criterion_main!(benches);
