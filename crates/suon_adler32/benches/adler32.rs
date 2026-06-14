//! Benchmarks for the suon_adler32 crate.
//!
//! Measures throughput across realistic packet sizes.  Packet sizes
//! reflect typical protocol payloads seen in the Suon MMORPG server.

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

/// Packet sizes benchmarked, in bytes.
///
/// These cover a range from tiny status responses up to 64 KiB,
/// including sizes that cross the 5552-byte lazy-modulo boundary
/// to exercise multi-chunk code paths.
const PACKET_SIZES: &[usize] = &[
    0, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072,
    262144, 524288, 1048576,
];

/// Byte value used to fill every benchmark buffer.
const PATTERN: u8 = 0xAB;

fn bench_adler32(c: &mut Criterion) {
    let mut group = c.benchmark_group("adler32");

    for &size in PACKET_SIZES {
        let data = vec![PATTERN; size];
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format_size(size), |b| {
            b.iter(|| {
                let result = suon_adler32::generate(black_box(&data));
                black_box(result);
            });
        });
    }

    group.finish();
}

fn format_size(bytes: usize) -> String {
    if bytes >= 1024 * 1024 {
        format!("{}_mb", bytes / (1024 * 1024))
    } else if bytes >= 1024 {
        format!("{}_kb", bytes / 1024)
    } else {
        format!("{bytes}_bytes")
    }
}

criterion_group!(
    name = adler32;
    config = Criterion::default();
    targets = bench_adler32
);
criterion_main!(adler32);
