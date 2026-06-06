use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use suon_xtea::{Key, decrypt, encrypt, expand};

const KEY_128: Key = [0x0123_4567, 0x89AB_CDEF, 0xFEDC_BA98, 0x7654_3210];

fn xtea_expand_key(c: &mut Criterion) {
    c.bench_function("xtea_expand_key", |b| {
        b.iter(|| expand(black_box(&KEY_128)))
    });
}

fn xtea_encrypt_single_block(c: &mut Criterion) {
    let round_keys = expand(&KEY_128);
    let mut buffer = vec![0xABu8; 8];
    c.bench_function("xtea_encrypt_8_bytes", |b| {
        b.iter(|| encrypt(black_box(&mut buffer), &round_keys).unwrap())
    });
}

fn xtea_decrypt_single_block(c: &mut Criterion) {
    let round_keys = expand(&KEY_128);
    let mut buffer = vec![0xABu8; 8];
    encrypt(&mut buffer, &round_keys).unwrap();
    c.bench_function("xtea_decrypt_8_bytes", |b| {
        b.iter(|| decrypt(black_box(&mut buffer), &round_keys).unwrap())
    });
}

fn xtea_encrypt_1_kibibyte(c: &mut Criterion) {
    let round_keys = expand(&KEY_128);
    let mut buffer = vec![0xABu8; 1024];
    c.bench_function("xtea_encrypt_1024_bytes", |b| {
        b.iter(|| encrypt(black_box(&mut buffer), &round_keys).unwrap())
    });
}

fn xtea_decrypt_1_kibibyte(c: &mut Criterion) {
    let round_keys = expand(&KEY_128);
    let mut buffer = vec![0xABu8; 1024];
    encrypt(&mut buffer, &round_keys).unwrap();
    c.bench_function("xtea_decrypt_1024_bytes", |b| {
        b.iter(|| decrypt(black_box(&mut buffer), &round_keys).unwrap())
    });
}

fn xtea_encrypt_1_mebibyte(c: &mut Criterion) {
    let round_keys = expand(&KEY_128);
    let mut buffer = vec![0xABu8; 1024 * 1024];
    c.bench_function("xtea_encrypt_1_mebibyte", |b| {
        b.iter(|| encrypt(black_box(&mut buffer), &round_keys).unwrap())
    });
}

fn xtea_decrypt_1_mebibyte(c: &mut Criterion) {
    let round_keys = expand(&KEY_128);
    let mut buffer = vec![0xABu8; 1024 * 1024];
    encrypt(&mut buffer, &round_keys).unwrap();
    c.bench_function("xtea_decrypt_1_mebibyte", |b| {
        b.iter(|| decrypt(black_box(&mut buffer), &round_keys).unwrap())
    });
}

fn xtea_roundtrip_1_mebibyte(c: &mut Criterion) {
    let round_keys = expand(&KEY_128);
    let mut buffer = vec![0xABu8; 1024 * 1024];
    c.bench_function("xtea_roundtrip_1_mebibyte", |b| {
        b.iter(|| {
            encrypt(black_box(&mut buffer), &round_keys).unwrap();
            decrypt(black_box(&mut buffer), &round_keys).unwrap();
        })
    });
}

criterion_group!(
    benches,
    xtea_expand_key,
    xtea_encrypt_single_block,
    xtea_decrypt_single_block,
    xtea_encrypt_1_kibibyte,
    xtea_decrypt_1_kibibyte,
    xtea_encrypt_1_mebibyte,
    xtea_decrypt_1_mebibyte,
    xtea_roundtrip_1_mebibyte,
);
criterion_main!(benches);
