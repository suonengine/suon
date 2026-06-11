use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use suon_network::{protocol::PacketWriter, server::tcp::ProtocolSettings};

const PACKET_SIZES: &[usize] = &[
    0, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072,
    262144,
];

fn checksum(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("checksum");

    for &size in PACKET_SIZES {
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format_size(size), |bencher| {
            let mut writer = PacketWriter::new(ProtocolSettings {
                header_size: 2,
                has_checksum: true,
                uses_xtea: false,
                uses_rsa: false,
            });

            let data = vec![0xABu8; size];
            bencher.iter(|| {
                writer.send(black_box(&data));
                drop(writer.take_buffer());
            });
        });
    }

    group.finish();
}

fn xtea(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("xtea");

    for &size in PACKET_SIZES {
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format_size(size), |bencher| {
            let mut writer = PacketWriter::new(ProtocolSettings {
                header_size: 6,
                has_checksum: true,
                uses_xtea: true,
                uses_rsa: true,
            });
            writer.set_xtea_key([0x0123_4567, 0x89AB_CDEF, 0xFEDC_BA98, 0x7654_3210]);

            let data = vec![0xABu8; size];
            bencher.iter(|| {
                writer.send(black_box(&data));
                drop(writer.take_buffer());
            });
        });
    }

    group.finish();
}

fn plain(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("plain");

    for &size in PACKET_SIZES {
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format_size(size), |bencher| {
            let mut writer = PacketWriter::new(ProtocolSettings {
                header_size: 2,
                has_checksum: true,
                uses_xtea: false,
                uses_rsa: false,
            });
            writer.set_xtea_enabled(false);

            let data = vec![0xABu8; size];
            bencher.iter(|| {
                writer.send(black_box(&data));
                drop(writer.take_buffer());
            });
        });
    }
    group.finish();
}

fn login_fallback(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("login_fallback");

    for &size in PACKET_SIZES {
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format_size(size), |bencher| {
            let mut writer = PacketWriter::new(ProtocolSettings {
                header_size: 6,
                has_checksum: true,
                uses_xtea: true,
                uses_rsa: false,
            });

            let data = vec![0xABu8; size];
            bencher.iter(|| {
                writer.send(black_box(&data));
                drop(writer.take_buffer());
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
    name = writer;
    config = Criterion::default();
    targets = checksum, xtea, plain, login_fallback
);
criterion_main!(writer);
