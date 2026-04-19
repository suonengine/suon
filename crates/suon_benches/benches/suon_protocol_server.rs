use bytes::Bytes;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::{
    hint::black_box,
    time::{Duration, UNIX_EPOCH},
};
use suon_protocol::prelude::*;
use suon_protocol_server::prelude::*;

struct BenchPacket;

impl Encodable for BenchPacket {
    const KIND: PacketKind = PacketKind::PingLatency;

    fn encode(self) -> Option<Bytes> {
        Some(Encoder::new().put_u16(0xCAFE).put_str("suon").finalize())
    }
}

fn benchmark_protocol_server(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol_server");

    group.bench_function("encode_with_kind", |b| {
        b.iter(|| BenchPacket.encode_with_kind())
    });

    group.bench_function("keep_alive_encode", |b| {
        b.iter(|| KeepAlivePacket.encode_with_kind())
    });

    let challenge = ChallengePacket {
        timestamp: UNIX_EPOCH + Duration::from_secs(1_234_567),
        random_number: 42,
    };
    group.bench_function("challenge_encode", |b| {
        b.iter(|| {
            black_box(ChallengePacket {
                timestamp: challenge.timestamp,
                random_number: challenge.random_number,
            })
            .encode_with_kind()
        })
    });

    for payload_size in [8usize, 64usize, 512usize] {
        group.bench_with_input(
            BenchmarkId::new("encoder_roundtrip", payload_size),
            &payload_size,
            |b, &payload_size| {
                let payload = vec![0xAB; payload_size];

                b.iter(|| {
                    let bytes = Encoder::new()
                        .put_u16(payload_size as u16)
                        .put_bytes(Bytes::from(payload.clone()))
                        .finalize();
                    let mut slice = bytes.as_ref();
                    let mut decoder = &mut slice;
                    let len = decoder.get_u16().expect("length should decode");
                    let remaining = decoder.take_remaining();
                    (len, remaining.len())
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_protocol_server);
criterion_main!(benches);
