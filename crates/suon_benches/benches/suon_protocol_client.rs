use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use suon_protocol::prelude::*;
use suon_protocol_client::prelude::*;

fn benchmark_protocol_client(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol_client");

    for text in ["bench", "suon-protocol", "decode-benchmark-payload"] {
        let payload = Encoder::new()
            .put_bool(true)
            .put_u32(42)
            .put_str(text)
            .finalize();

        group.bench_with_input(
            BenchmarkId::new("decode_sequence", text.len()),
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

    group.bench_function("keep_alive_decode", |b| {
        b.iter(|| {
            let mut payload: &[u8] = black_box(&[]);
            KeepAlivePacket::decode(&mut payload)
                .expect("Client keep-alive packets should decode without payload bytes")
        })
    });

    black_box(PacketKind::KeepAlive);

    group.finish();
}

criterion_group!(benches, benchmark_protocol_client);
criterion_main!(benches);
