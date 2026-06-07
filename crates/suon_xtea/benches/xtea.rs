//! Benchmarks for the suon_xtea crate.
//!
//! Measures encryption and decryption throughput across realistic packet
//! sizes under both hot cache and cold cache scenarios.  Packet sizes
//! reflect typical protocol payloads.

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use suon_xtea::{ExpandedKey, Key, decrypt, encrypt, expand};

/// 128-bit XTEA key shared by all benchmarks.
const XTEA_KEY_128_BIT: Key = [0x0123_4567, 0x89AB_CDEF, 0xFEDC_BA98, 0x7654_3210];

/// Byte value used to fill every benchmark buffer.
const PLAINTEXT_PATTERN: u8 = 0xAB;

/// Expanded round keys for [`XTEA_KEY_128_BIT`].
///
/// Evaluated at compile time so no runtime setup cost is paid by any
/// individual benchmark.
const EXPANDED_KEYS: ExpandedKey = expand(&XTEA_KEY_128_BIT);

/// Memory pool size for cold cache benchmarks.
///
/// Large enough to exceed the CPU cache (64 mebibytes) so the rotating
/// stride evicts previously accessed regions.
///
/// A fresh [`pool.fill()`](Vec::fill) call between sizes guarantees that
/// each benchmark starts from the same clean state, regardless of
/// mutations from the previous size.
const COLD_CACHE_POOL_SIZE: usize = 64 * 1024 * 1024;

/// Packet sizes benchmarked under the cold cache scenario.
///
/// Limited to the most common realistic sizes because allocating and
/// striding through a 64-mebibyte pool is expensive.
const COLD_CACHE_SIZES: &[usize] = &[64, 256, 1024];

/// Size used for the roundtrip benchmark (encrypt + decrypt).
///
/// 1024 bytes is the most realistic packet size for this scenario.
const ROUNDTRIP_SIZE: usize = 1024;

/// Packet sizes that represent realistic protocol payloads.
///
/// Includes a single-block minimum (8 bytes), an odd-block count
/// (24 bytes = 3 blocks), common small-to-large packets, and a
/// throughput measurement size (1 mebibyte).
const PACKET_SIZES: &[usize] = &[8, 24, 64, 128, 256, 512, 1024, 1024 * 1024];

fn expand_key(criterion: &mut Criterion) {
    criterion.bench_function("xtea/expand_key", |bencher| {
        bencher.iter(|| expand(black_box(&XTEA_KEY_128_BIT)));
    });
}

fn encrypt_hot_cache(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("xtea/encrypt");

    for &packet_size in PACKET_SIZES {
        group.throughput(Throughput::Bytes(packet_size as u64));
        group.bench_function(format_size(packet_size), |bencher| {
            let mut buffer = vec![PLAINTEXT_PATTERN; packet_size];
            bencher.iter(|| {
                encrypt(black_box(&mut buffer), &EXPANDED_KEYS)
                    .expect("encryption of fixed-size buffer must succeed");
            });
        });
    }

    group.finish();
}

fn decrypt_hot_cache(criterion: &mut Criterion) {
    // Pre-encrypts one buffer per size so the decryption benchmark has
    // valid ciphertext to operate on.
    let mut buffers: Vec<Vec<u8>> = PACKET_SIZES
        .iter()
        .map(|&packet_size| {
            let mut buffer = vec![PLAINTEXT_PATTERN; packet_size];
            encrypt(&mut buffer, &EXPANDED_KEYS)
                .expect("buffer encryption for decryption benchmark setup must succeed");
            buffer
        })
        .collect();

    let mut group = criterion.benchmark_group("xtea/decrypt");

    for (index, &packet_size) in PACKET_SIZES.iter().enumerate() {
        group.throughput(Throughput::Bytes(packet_size as u64));
        group.bench_function(format_size(packet_size), |bencher| {
            let buffer = &mut buffers[index];
            bencher.iter(|| {
                decrypt(black_box(buffer.as_mut_slice()), &EXPANDED_KEYS)
                    .expect("decryption of fixed-size buffer must succeed");
            });
        });
    }

    group.finish();
}

fn encrypt_cold_cache(criterion: &mut Criterion) {
    let mut pool: Vec<u8> = vec![PLAINTEXT_PATTERN; COLD_CACHE_POOL_SIZE];

    let mut group = criterion.benchmark_group("xtea/encrypt/cold_cache");

    for &packet_size in COLD_CACHE_SIZES {
        // Resets the pool to a clean state so every size starts with
        // identical data, regardless of mutations from previous runs.
        pool.fill(PLAINTEXT_PATTERN);

        group.throughput(Throughput::Bytes(packet_size as u64));
        group.bench_function(format_size(packet_size), |bencher| {
            let mut offset: usize = 0;
            bencher.iter(|| {
                let start: usize = offset;
                offset = (start + packet_size) % (COLD_CACHE_POOL_SIZE - packet_size);

                encrypt(
                    black_box(&mut pool[start..start + packet_size]),
                    &EXPANDED_KEYS,
                )
                .expect("encryption of cold cache pool slice must succeed");
            });
        });
    }

    group.finish();
}

fn decrypt_cold_cache(criterion: &mut Criterion) {
    let mut pool: Vec<u8> = vec![PLAINTEXT_PATTERN; COLD_CACHE_POOL_SIZE];

    let mut group = criterion.benchmark_group("xtea/decrypt/cold_cache");

    for &packet_size in COLD_CACHE_SIZES {
        // Resets the pool to a clean state so every size starts with
        // identical data, regardless of mutations from previous runs.
        pool.fill(PLAINTEXT_PATTERN);

        group.throughput(Throughput::Bytes(packet_size as u64));
        group.bench_function(format_size(packet_size), |bencher| {
            // XTEA uses only arithmetic operations (wrapping_add,
            // wrapping_sub, xor, shifts) with no data-dependent branches
            // or table lookups.  The computational cost of decrypting
            // plaintext is therefore identical to decrypting valid
            // ciphertext, which lets us skip a pre-encryption pass that
            // would warm the pool and defeat the cold cache measurement.
            let mut offset: usize = 0;
            bencher.iter(|| {
                let start: usize = offset;
                offset = (start + packet_size) % (COLD_CACHE_POOL_SIZE - packet_size);
                decrypt(
                    black_box(&mut pool[start..start + packet_size]),
                    &EXPANDED_KEYS,
                )
                .expect("decryption of cold cache pool slice must succeed");
            });
        });
    }

    group.finish();
}

fn roundtrip(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("xtea/roundtrip");

    group.throughput(Throughput::Bytes(ROUNDTRIP_SIZE as u64));
    group.bench_function(format_size(ROUNDTRIP_SIZE), |bencher| {
        let mut buffer = vec![PLAINTEXT_PATTERN; ROUNDTRIP_SIZE];
        bencher.iter(|| {
            encrypt(black_box(&mut buffer), &EXPANDED_KEYS)
                .expect("encryption in roundtrip benchmark must succeed");
            decrypt(black_box(&mut buffer), &EXPANDED_KEYS)
                .expect("decryption in roundtrip benchmark must succeed");
        });
    });

    group.finish();
}

/// Formats a byte count into a human-readable benchmark label.
///
/// Sizes below one mebibyte use a plain number suffixed with `_bytes`.
/// Sizes of one mebibyte or larger use a mebibyte count so the label
/// stays readable.
fn format_size(bytes: usize) -> String {
    if bytes >= 1024 * 1024 {
        let mebibytes = bytes / (1024 * 1024);
        if mebibytes == 1 {
            "1_mebibyte".to_string()
        } else {
            format!("{mebibytes}_mebibytes")
        }
    } else {
        format!("{}_bytes", bytes)
    }
}

criterion_group!(
    name = xtea_benchmarks;
    config = Criterion::default();
    targets =
        expand_key,
        encrypt_hot_cache,
        decrypt_hot_cache,
        encrypt_cold_cache,
        decrypt_cold_cache,
        roundtrip,
);
criterion_main!(xtea_benchmarks);
