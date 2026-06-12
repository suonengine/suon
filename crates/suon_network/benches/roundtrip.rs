use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use suon_network::{
    protocol::{PacketReader, PacketWriter},
    server::tcp::ProtocolSettings,
};
use suon_xtea::Key;

const XTEA_KEY: Key = [0x0123_4567, 0x89AB_CDEF, 0xFEDC_BA98, 0x7654_3210];

const PACKET_SIZES: &[usize] = &[
    0, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536,
];

fn bench_roundtrip(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("roundtrip");

    for &size in PACKET_SIZES {
        let plaintext = vec![0xABu8; size];

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format_size(size), |bencher| {
            bencher.iter(|| {
                let mut writer = PacketWriter::new(
                    ProtocolSettings {
                        header_size: 6,
                        has_checksum: true,
                        uses_xtea: true,
                        uses_rsa: true,
                    },
                    4096,
                );
                writer.set_xtea_key(XTEA_KEY);
                writer.send(&plaintext);

                let framed = writer.take_buffer();

                let mut reader = PacketReader::new(ProtocolSettings {
                    header_size: 6,
                    has_checksum: true,
                    uses_xtea: true,
                    uses_rsa: true,
                });
                reader.set_xtea_key(XTEA_KEY);
                reader.set_rsa_done(true);

                let body = &framed[2..];
                let result = reader.process(black_box(body)).ok().flatten();
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
    name = roundtrip;
    config = Criterion::default();
    targets = bench_roundtrip
);
criterion_main!(roundtrip);
