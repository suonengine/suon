use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use suon_network::server::tcp::xtea_pad;
use suon_xtea::{Key, decrypt, encrypt, expand};

const XTEA_KEY: Key = [0x0123_4567, 0x89AB_CDEF, 0xFEDC_BA98, 0x7654_3210];

const EXPANDED_KEY: suon_xtea::ExpandedKey = expand(&XTEA_KEY);

const PACKET_SIZES: &[usize] = &[
    0, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072,
    262144, 524288,
];

fn bench_encrypt(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("encrypt");

    let expanded = &EXPANDED_KEY;

    for &size in PACKET_SIZES {
        let padded = xtea_pad(&vec![0xABu8; size]);

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format_size(size), |bencher| {
            let mut buffer = padded.clone();
            bencher.iter(|| {
                encrypt(black_box(&mut buffer), expanded)
                    .expect("bench XTEA encryption should succeed");
                black_box(&buffer);
            });
        });
    }

    group.finish();
}

fn bench_decrypt(criterion: &mut Criterion) {
    let expanded = &EXPANDED_KEY;
    let mut group = criterion.benchmark_group("decrypt");

    for &size in PACKET_SIZES {
        let padded = xtea_pad(&vec![0xABu8; size]);
        let mut encrypted = padded.clone();
        encrypt(&mut encrypted, expanded).expect("bench XTEA encryption should succeed");

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format_size(size), |bencher| {
            let mut buffer = encrypted.clone();
            bencher.iter(|| {
                decrypt(black_box(&mut buffer), expanded)
                    .expect("bench XTEA decryption should succeed");
                black_box(&buffer);
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
    name = xtea;
    config = Criterion::default();
    targets = bench_encrypt, bench_decrypt
);
criterion_main!(xtea);
