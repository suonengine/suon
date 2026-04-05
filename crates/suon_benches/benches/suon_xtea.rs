use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use suon_xtea::{XTEAKey, decrypt, encrypt, expand_key};

const KEY: XTEAKey = [0xA56BABCD, 0x00000000, 0xFFFFFFFF, 0x12345678];
const MESSAGE: &[u8] = b"\x0E\0Sample Message";

fn benchmark_expand_key(c: &mut Criterion) {
    c.bench_function("xtea/expand_key", |b| {
        b.iter(|| expand_key(black_box(&KEY)))
    });
}

fn benchmark_encrypt(c: &mut Criterion) {
    c.bench_function("xtea/encrypt", |b| {
        b.iter(|| encrypt(black_box(MESSAGE), black_box(&KEY)))
    });
}

fn benchmark_decrypt(c: &mut Criterion) {
    let ciphertext = encrypt(MESSAGE, &KEY);

    c.bench_function("xtea/decrypt", |b| {
        b.iter(|| decrypt(black_box(&ciphertext), black_box(&KEY)))
    });
}

criterion_group!(
    benches,
    benchmark_expand_key,
    benchmark_encrypt,
    benchmark_decrypt
);
criterion_main!(benches);
