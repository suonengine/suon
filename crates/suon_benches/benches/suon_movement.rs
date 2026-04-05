use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use suon_movement::prelude::{StepDirection, StepPath};
use suon_position::position::Position;

fn benchmark_direction_math(c: &mut Criterion) {
    let mut group = c.benchmark_group("movement/direction_math");

    for direction in [
        StepDirection::North,
        StepDirection::NorthEast,
        StepDirection::East,
        StepDirection::SouthWest,
    ] {
        group.bench_with_input(
            BenchmarkId::new("add_then_sub", format!("{direction:?}")),
            &direction,
            |b, direction| {
                b.iter(|| {
                    let position = Position { x: 100, y: 100 };
                    let position = black_box(position) + *direction;
                    position - *direction
                })
            },
        );
    }

    group.finish();
}

fn benchmark_path_push_pop(c: &mut Criterion) {
    let mut group = c.benchmark_group("movement/path");

    for queued_steps in [2usize, 16usize, 128usize] {
        group.bench_with_input(
            BenchmarkId::new("push_pop", queued_steps),
            &queued_steps,
            |b, &queued_steps| {
                b.iter(|| {
                    let mut path = StepPath::default();
                    for index in 0..queued_steps {
                        let direction = if index % 2 == 0 {
                            StepDirection::North
                        } else {
                            StepDirection::East
                        };
                        path.push(direction);
                    }

                    while path.pop().is_some() {}

                    black_box(path.is_empty())
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("clear", queued_steps),
            &queued_steps,
            |b, &queued_steps| {
                b.iter(|| {
                    let mut path = StepPath::default();
                    for index in 0..queued_steps {
                        let direction = if index % 2 == 0 {
                            StepDirection::South
                        } else {
                            StepDirection::West
                        };
                        path.push(direction);
                    }

                    path.clear();
                    black_box(path.len())
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_direction_math, benchmark_path_push_pop);
criterion_main!(benches);
