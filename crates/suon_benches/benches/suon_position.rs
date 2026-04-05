use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::{
    collections::{BTreeSet, HashSet},
    hint::black_box,
};
use suon_position::{
    floor::Floor, position::Position, previous_floor::PreviousFloor,
    previous_position::PreviousPosition,
};

fn benchmark_position_btree_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("position");

    for item_count in [128usize, 1_024usize, 4_096usize] {
        group.bench_with_input(
            BenchmarkId::new("btree_insert", item_count),
            &item_count,
            |b, &item_count| {
                b.iter(|| {
                    let mut set = BTreeSet::new();

                    for index in 0..item_count {
                        set.insert(Position {
                            x: black_box((index % 1024) as u16),
                            y: black_box(((index * 3) % 1024) as u16),
                        });
                    }

                    set.len()
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("hash_insert_previous_position", item_count),
            &item_count,
            |b, &item_count| {
                b.iter(|| {
                    let mut set = HashSet::new();

                    for index in 0..item_count {
                        set.insert(PreviousPosition {
                            x: black_box((index % 1024) as u16),
                            y: black_box(((index * 5) % 1024) as u16),
                        });
                    }

                    set.len()
                })
            },
        );
    }

    group.bench_function("floor_ord_sort", |b| {
        b.iter(|| {
            let mut floors = vec![
                Floor { z: 7 },
                Floor { z: 2 },
                Floor { z: 9 },
                Floor { z: 1 },
                Floor { z: 4 },
            ];
            floors.sort();
            black_box(floors)
        })
    });

    group.bench_function("previous_floor_hash", |b| {
        b.iter(|| {
            let mut set = HashSet::new();
            for z in 0..32u8 {
                set.insert(PreviousFloor { z: black_box(z) });
            }
            black_box(set.len())
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_position_btree_insert);
criterion_main!(benches);
