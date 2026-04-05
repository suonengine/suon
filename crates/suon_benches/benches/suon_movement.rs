use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use suon_movement::prelude::{StepDirection, StepPath};
use suon_position::position::Position;

fn benchmark_direction_math(c: &mut Criterion) {
    c.bench_function("movement/direction_math", |b| {
        b.iter(|| {
            let position = Position { x: 100, y: 100 };
            let position = black_box(position) + StepDirection::NorthEast;
            position - StepDirection::SouthWest
        })
    });
}

fn benchmark_path_push_pop(c: &mut Criterion) {
    c.bench_function("movement/path_push_pop", |b| {
        b.iter(|| {
            let mut path = StepPath::default();
            path.push(StepDirection::North);
            path.push(StepDirection::East);
            let _ = path.pop();
            path.pop()
        })
    });
}

criterion_group!(benches, benchmark_direction_math, benchmark_path_push_pop);
criterion_main!(benches);
