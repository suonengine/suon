use ::benches::bench;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use suon_xtea::{XTEAKey, decrypt, encrypt, expand_key};

const KEY: XTEAKey = [0xA56BABCD, 0x00000000, 0xFFFFFFFF, 0x12345678];

fn packet_with_payload_len(payload_len: usize) -> Vec<u8> {
    let mut packet = (payload_len as u16).to_le_bytes().to_vec();
    packet.extend((0..payload_len).map(|index| (index % 251) as u8));
    packet
}

fn benchmark_xtea(c: &mut Criterion) {
    let mut group = c.benchmark_group("xtea");

    group.bench_function(bench!("expand_key"), |b| {
        b.iter(|| expand_key(black_box(&KEY)))
    });

    for payload_len in [0usize, 5, 14, 64, 256] {
        let packet = packet_with_payload_len(payload_len);
        group.bench_with_input(
            BenchmarkId::new(bench!("encrypt"), payload_len),
            &packet,
            |b, packet| b.iter(|| encrypt(black_box(packet), black_box(&KEY))),
        );
    }

    for payload_len in [0usize, 5, 14, 64, 256] {
        let packet = packet_with_payload_len(payload_len);
        let ciphertext = encrypt(&packet, &KEY);
        group.bench_with_input(
            BenchmarkId::new(bench!("decrypt"), payload_len),
            &ciphertext,
            |b, ciphertext| b.iter(|| decrypt(black_box(ciphertext), black_box(&KEY))),
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_xtea);
criterion_main!(benches);
