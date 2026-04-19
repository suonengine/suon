use ::benches::bench;
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use suon_checksum::Adler32Checksum;

fn benchmark_checksum(c: &mut Criterion) {
    let mut group = c.benchmark_group("checksum");
    let tiny = vec![0xAB; 16];
    let small = vec![0xCD; 256];
    let medium = vec![0xEF; 4_096];
    let large = vec![0x42; 65_536];

    for (name, payload) in [
        ("empty", &[][..]),
        ("tiny", tiny.as_slice()),
        ("small", small.as_slice()),
        ("medium", medium.as_slice()),
        ("large", large.as_slice()),
    ] {
        group.bench_function(format!("{}/{}", bench!("calculate"), name), |b| {
            b.iter(|| Adler32Checksum::calculate(black_box(payload)))
        });

        group.bench_function(format!("{}/{}", bench!("from-slice"), name), |b| {
            b.iter(|| Adler32Checksum::from(black_box(payload)))
        });
    }

    group.bench_function(bench!("from-vec/small"), |b| {
        b.iter(|| Adler32Checksum::from(black_box(small.clone())))
    });

    let display_checksum = Adler32Checksum::calculate(&large);
    group.bench_function(bench!("display/large"), |b| {
        b.iter(|| black_box(display_checksum).to_string())
    });

    group.finish();
}

criterion_group!(benches, benchmark_checksum);
criterion_main!(benches);
