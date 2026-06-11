use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use suon_network::server::tcp::{xtea_pad, xtea_unpad};

const PACKET_SIZES: &[usize] = &[
    0, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072,
    262144, 524288, 1048576,
];

fn adler32_bench(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("adler32");

    for &size in PACKET_SIZES {
        let data = vec![0xABu8; size];
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format_size(size), |bencher| {
            bencher.iter(|| {
                let result = suon_adler32::generate(black_box(&data));
                black_box(result);
            });
        });
    }

    group.finish();
}

fn xtea_pad_bench(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("xtea_pad");

    for &size in PACKET_SIZES {
        let data = vec![0xABu8; size];
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format_size(size), |bencher| {
            bencher.iter(|| {
                let result = xtea_pad(black_box(&data));
                black_box(result);
            });
        });
    }

    group.finish();
}

fn xtea_unpad_bench(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("xtea_unpad");

    for &size in PACKET_SIZES {
        let padded = xtea_pad(&vec![0xABu8; size]);
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format_size(size), |bencher| {
            bencher.iter(|| {
                let result = xtea_unpad(black_box(&padded));
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
    name = primitives;
    config = Criterion::default();
    targets = adler32_bench, xtea_pad_bench, xtea_unpad_bench
);
criterion_main!(primitives);
