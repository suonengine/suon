use criterion::{Criterion, criterion_group, criterion_main};
use std::{collections::BTreeSet, hint::black_box};
use suon_position::position::Position;

fn benchmark_position_btree_insert(c: &mut Criterion) {
    c.bench_function("position/btree_insert", |b| {
        b.iter(|| {
            let mut set = BTreeSet::new();

            for index in 0..128usize {
                set.insert(Position {
                    x: black_box((index % 1024) as u16),
                    y: black_box(((index * 3) % 1024) as u16),
                });
            }

            set.len()
        })
    });
}

criterion_group!(benches, benchmark_position_btree_insert);
criterion_main!(benches);
