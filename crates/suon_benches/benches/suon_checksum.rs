use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use suon_checksum::Adler32Checksum;

fn benchmark_checksum(c: &mut Criterion) {
    const PAYLOAD: &[u8] = b"benchmark-payload-for-adler32";

    c.bench_function("checksum/calculate", |b| {
        b.iter(|| Adler32Checksum::calculate(black_box(PAYLOAD)))
    });
}

criterion_group!(benches, benchmark_checksum);
criterion_main!(benches);
