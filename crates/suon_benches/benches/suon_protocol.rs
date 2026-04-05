use bytes::Bytes;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::{
    hint::black_box,
    time::{Duration, UNIX_EPOCH},
};
use suon_protocol::packets::{
    client::{Decodable, prelude::KeepAlivePacket as ClientKeepAlivePacket},
    decoder::Decoder,
    encoder::Encoder,
    server::{
        Encodable, PacketKind,
        prelude::{ChallengePacket, KeepAlivePacket as ServerKeepAlivePacket},
    },
};

struct BenchPacket;

impl Encodable for BenchPacket {
    const KIND: PacketKind = PacketKind::PingLatency;

    fn encode(self) -> Option<Bytes> {
        Some(Encoder::new().put_u16(0xCAFE).put_str("suon").finalize())
    }
}

fn benchmark_encode_with_kind(c: &mut Criterion) {
    c.bench_function("protocol/encode_with_kind", |b| {
        b.iter(|| BenchPacket.encode_with_kind())
    });
}

fn benchmark_decode_sequence(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol/decode_sequence");

    for text in ["bench", "suon-protocol", "decode-benchmark-payload"] {
        let payload = Encoder::new()
            .put_bool(true)
            .put_u32(42)
            .put_str(text)
            .finalize();

        group.bench_with_input(
            BenchmarkId::new("mixed_fields", text.len()),
            &payload,
            |b, payload| {
                b.iter(|| {
                    let mut slice = payload.as_ref();
                    let flag = (&mut slice).get_bool().expect("bool should decode");
                    let value = (&mut slice).get_u32().expect("u32 should decode");
                    let text = (&mut slice).get_string().expect("string should decode");
                    (flag, value, text)
                })
            },
        );
    }

    group.finish();
}

fn benchmark_server_keep_alive_encode(c: &mut Criterion) {
    c.bench_function("protocol/server_keep_alive_encode", |b| {
        b.iter(|| ServerKeepAlivePacket.encode_with_kind())
    });
}

fn benchmark_client_keep_alive_decode(c: &mut Criterion) {
    c.bench_function("protocol/client_keep_alive_decode", |b| {
        b.iter(|| {
            let mut payload: &[u8] = black_box(&[]);
            ClientKeepAlivePacket::decode(&mut payload)
                .expect("Client keep-alive packets should decode without payload bytes")
        })
    });
}

fn benchmark_challenge_encode(c: &mut Criterion) {
    let packet = ChallengePacket {
        timestamp: UNIX_EPOCH + Duration::from_secs(1_234_567),
        random_number: 42,
    };

    c.bench_function("protocol/challenge_encode", |b| {
        b.iter(|| {
            black_box(ChallengePacket {
                timestamp: packet.timestamp,
                random_number: packet.random_number,
            })
            .encode_with_kind()
        })
    });
}

fn benchmark_encoder_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol/encoder_roundtrip");

    for payload_size in [8usize, 64usize, 512usize] {
        group.bench_with_input(
            BenchmarkId::new("put_bytes_then_take_remaining", payload_size),
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

criterion_group!(
    benches,
    benchmark_encode_with_kind,
    benchmark_decode_sequence,
    benchmark_server_keep_alive_encode,
    benchmark_client_keep_alive_decode,
    benchmark_challenge_encode,
    benchmark_encoder_roundtrip
);
criterion_main!(benches);
