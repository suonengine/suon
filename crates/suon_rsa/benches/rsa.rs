//! Benchmarks for the suon_rsa crate.
//!
//! Measures RSA key loading, encryption, decryption, and roundtrip
//! throughput for a 1024-bit key.

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use suon_rsa::{decrypt, encrypt, load_pem};

/// 1024-bit RSA private key in PEM format.
const RSA_PRIVATE_KEY_PEM: &str = concat!(
    "-----BEGIN RSA PRIVATE KEY-----\n",
    "MIICXAIBAAKBgQCbZGkDtFsHrJVlaNhzU71xZROd15QHA7A+bdB5OZZhtKg3qmBWHXzLlFL6AIBZ\n",
    "SQmIKrW8pYoaGzX4sQWbcrEhJhHGFSrT27PPvuetwUKnXT11lxUJwyHFwkpb1R/UYPAbThW+sN4Z\n",
    "MFKKXT8VwePL9cQB1nd+EKyqsz2+jVt/9QIDAQABAoGAQovTtTRtr3GnYRBvcaQxAvjIV9ZUnFRm\n",
    "C7Y3i1KwJhOZ3ozmSLrEEOLqTgoc7R+sJ1YzEiDKbbete11EC3gohlhW56ptj0WDf+7ptKOgqiEy\n",
    "Kh4qt1sYJeeGz4GiiooJoeKFGdtk/5uvMR6FDCv6H7ewigVswzf330Q3Ya7+jYECQQERBxsga6+5\n",
    "x6IofXyNF6QuMqvuiN/pUgaStUOdlnWBf/T4yUpKvNS1+I4iDzqGWOOSR6RsaYPYVhj9iRABoKyx\n",
    "AkEAkbNzB6vhLAWht4dUdGzaREF3p4SwNcu5bJRa/9wCLSHaS9JaTq4lljgVPp1zyXyJCSCWpFnl\n",
    "0WvK3Qf6nVBIhQJBANS7rK8+ONWQbxENdZaZ7Rrx8HUTwSOS/fwhsGWBbl1Qzhdq/6/sIfEHkfeH\n",
    "1hoH+IlpuPuf21MdAqvJt+cMwoECQF1LyBOYduYGcSgg6u5mKVldhm3pJCA+ZGxnjuGZEnet3qeA\n",
    "eb05++112fyvO85ABUun524z9lokKNFh45NKLjUCQGshzV43P+RioiBhtEpB/QFzijiS4L2HKNu1\n",
    "tdhudnUjWkaf6jJmQS/ppln0hhRMHlk9Vus/bPx7LtuDuo6VQDo=\n",
    "-----END RSA PRIVATE KEY-----\n",
);

/// Byte value used to fill benchmark buffers.  0x78 is less than the
/// first byte of the modulus (0x9b), guaranteeing the plaintext is
/// numerically smaller than `n`.
const PLAINTEXT_PATTERN: u8 = 0x78;

/// Key size in bytes for a 1024-bit RSA key.
const KEY_SIZE: usize = 128;

fn load_pem_benchmark(criterion: &mut Criterion) {
    criterion.bench_function("rsa/load_pem", |bencher| {
        bencher.iter(|| load_pem(black_box(RSA_PRIVATE_KEY_PEM)));
    });
}

fn encrypt_benchmark(criterion: &mut Criterion) {
    let key = load_pem(RSA_PRIVATE_KEY_PEM).expect("test PEM must be valid");

    let mut group = criterion.benchmark_group("rsa/encrypt");

    group.throughput(Throughput::Bytes(KEY_SIZE as u64));
    group.bench_function("1024_bit", |bencher| {
        let mut buffer = vec![PLAINTEXT_PATTERN; KEY_SIZE];
        bencher.iter(|| {
            encrypt(black_box(&key), black_box(&mut buffer))
                .expect("encrypt of fixed-size buffer must succeed");
        });
    });

    group.finish();
}

fn decrypt_benchmark(criterion: &mut Criterion) {
    let key = load_pem(RSA_PRIVATE_KEY_PEM).expect("test PEM must be valid");

    // Pre-encrypt a buffer so the decryption benchmark has valid ciphertext.
    let mut buffer = vec![PLAINTEXT_PATTERN; KEY_SIZE];
    encrypt(&key, &mut buffer).expect("buffer encryption for decrypt benchmark setup must succeed");

    let mut group = criterion.benchmark_group("rsa/decrypt");

    group.throughput(Throughput::Bytes(KEY_SIZE as u64));
    group.bench_function("1024_bit", |bencher| {
        bencher.iter(|| {
            decrypt(black_box(&key), black_box(buffer.as_mut_slice()))
                .expect("decrypt of fixed-size buffer must succeed");
        });
    });

    group.finish();
}

fn roundtrip_benchmark(criterion: &mut Criterion) {
    let key = load_pem(RSA_PRIVATE_KEY_PEM).expect("test PEM must be valid");

    let mut group = criterion.benchmark_group("rsa/roundtrip");

    group.throughput(Throughput::Bytes(KEY_SIZE as u64));
    group.bench_function("1024_bit", |bencher| {
        let mut buffer = vec![PLAINTEXT_PATTERN; KEY_SIZE];
        bencher.iter(|| {
            encrypt(black_box(&key), black_box(&mut buffer))
                .expect("encrypt in roundtrip must succeed");
            decrypt(black_box(&key), black_box(&mut buffer))
                .expect("decrypt in roundtrip must succeed");
        });
    });

    group.finish();
}

criterion_group!(
    name = rsa_benchmarks;
    config = Criterion::default();
    targets =
        load_pem_benchmark,
        encrypt_benchmark,
        decrypt_benchmark,
        roundtrip_benchmark,
);
criterion_main!(rsa_benchmarks);
