use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use suon_network::{
    protocol::{PacketReader, ProcessError},
    server::tcp::{ProtocolSettings, RSA_KEY_SIZE, XTEA_KEY_BYTES, xtea_pad},
};
use suon_xtea::{Key, encrypt, expand};

const XTEA_KEY: Key = [0x0123_4567, 0x89AB_CDEF, 0xFEDC_BA98, 0x7654_3210];
const EXPANDED_KEY: suon_xtea::ExpandedKey = expand(&XTEA_KEY);

/// 1024-bit RSA key used for RSA handshake benchmarks.
const TEST_RSA_PEM: &str = "\
-----BEGIN RSA PRIVATE KEY-----\nMIICXgIBAAKBgQDSpz7WAGdJgdvbIGy4leEbqptY/LyxWY4eyJ5Fn/IC0cMWw830\\
     nRexg0F78yHeA4Rcu0V6r5oatCtoKTgbO5g9UJtY9BHXANiK4K4q+RVjSeEDx0StW\n+EqhRGptc0c39T0B/\
                            dSbw9Y8lKmkaOk/2OEPCGtPW6qbwt5ahBoJINwYEQIDAQAB\\
     nAoGAIk15vf9y0lWDJ7uv+J7veUHe6i69y2N58SlaHJxfHHZr/lkEQLLiOyGzVhaO\n3z3IOKd/\
                            cx6m76bEusjZ8vcjp5Sry1xZQuMWBx2iCB0e9+nxGuaSTSoOJrpscJLH\\
     nngqqdjGJY6brU6QpEV0w8UjWnXe9pVWIORQIpa/fdME4/8ECQQD1iTLmuQPG0Y+p\\
     nHnCRdgeKUUNUXMDjO9cQtSMHH2ke1ZMbpayAKrhPFuBc6qf1kur0J8WH9xNAvf+c\nkLzZgbL7AkEA26F5/\
                            idz+5OyNXEnudthKyEToPO53SJYY5uyEcOdRYEgrsNYCVsM\\
     nJvKV1vlBriZ1GiNeYWKVQ4Y3AYMLyF3TYwJBAICl7DebRPFNJ7pyqoRslTLRtTdk\\
     nieQFnH+yiLHYsVloifV4btOQjpVR5SiKAorW+agHlqXQvRO0+VLtOyWzoTUCQQC/\njPfex/\
                            4J3mjA322sVT9L5E9AQxFJYhkA1tvZTmguJE6i3VA86KGSnmQ816uG/ZeI\\
                            nMmywNtDD0ZzLvsVZ/SrNAkEA7i/nj9I9vYmjroqD+1r6D5zfj5rFmqxAhW8wMDzh\\
                            ntO6vywVbLiOFudajEttnKgRV7AWJENyfTbhcuW1AXJvlEA==\\
     n-----END RSA PRIVATE KEY-----";

const PACKET_SIZES: &[usize] = &[
    0, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072,
    262144,
];

fn xtea_decrypt(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("xtea_decrypt");

    let expanded = &EXPANDED_KEY;

    for &size in PACKET_SIZES {
        let plaintext = vec![0x42u8; size];
        let padded = xtea_pad(&plaintext);
        let mut encrypted = padded.clone();
        encrypt(&mut encrypted, expanded).expect("benchmark XTEA encryption should succeed");

        let mut body = Vec::with_capacity(4 + encrypted.len());
        body.extend_from_slice(&0u32.to_le_bytes());
        body.extend_from_slice(&encrypted);

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format_size(size), |bencher| {
            let mut reader = PacketReader::new(ProtocolSettings {
                header_size: 6,
                has_checksum: true,
                uses_xtea: true,
                uses_rsa: true,
            });
            reader.set_xtea_key(XTEA_KEY);
            reader.set_rsa_done(true);

            bencher.iter(|| {
                let result = reader.process(black_box(&body)).ok().flatten();
                black_box(result);
            });
        });
    }

    group.finish();
}

fn rsa_handshake(criterion: &mut Criterion) {
    let rsa = suon_rsa::load_pem(TEST_RSA_PEM).expect("test RSA key must be valid");
    let mut rsa_buf = vec![0u8; RSA_KEY_SIZE];
    rsa_buf[0] = 0;
    let xtea_key_bytes: [u8; XTEA_KEY_BYTES] = [
        0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0xFE, 0xDC, 0xBA, 0x98, 0x76, 0x54, 0x32,
        0x10,
    ];
    rsa_buf[1..=XTEA_KEY_BYTES].copy_from_slice(&xtea_key_bytes);
    suon_rsa::encrypt(&rsa, &mut rsa_buf).expect("RSA encrypt must succeed");

    let mut group = criterion.benchmark_group("xtea_decrypt_rsa_done");

    group.bench_function("rsa_handshake", |bencher| {
        bencher.iter(|| {
            let rsa = suon_rsa::load_pem(TEST_RSA_PEM).expect("test RSA key must be valid");
            let mut reader = PacketReader::new(ProtocolSettings {
                header_size: 6,
                has_checksum: true,
                uses_xtea: true,
                uses_rsa: true,
            });
            reader.set_rsa_key(rsa);
            let result = reader.process(black_box(&rsa_buf)).ok().flatten();
            black_box(result);
        });
    });

    group.finish();
}

fn checksum_only(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("checksum_only");

    for &size in PACKET_SIZES {
        let data = vec![0xABu8; size];
        let checksum = suon_adler32::generate(&data);
        let mut body = Vec::with_capacity(4 + size);
        body.extend_from_slice(&checksum.to_le_bytes());
        body.extend_from_slice(&data);

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format_size(size), |bencher| {
            let mut reader = PacketReader::new(ProtocolSettings {
                header_size: 2,
                has_checksum: true,
                uses_xtea: false,
                uses_rsa: false,
            });

            bencher.iter(|| {
                let result = reader.process(black_box(&body)).ok().flatten();
                black_box(result);
            });
        });
    }

    group.finish();
}

fn plaintext(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("plaintext");

    for &size in PACKET_SIZES {
        let data = vec![0xABu8; size];

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format_size(size), |bencher| {
            let mut reader = PacketReader::new(ProtocolSettings {
                header_size: 2,
                has_checksum: true,
                uses_xtea: false,
                uses_rsa: false,
            });

            bencher.iter(|| {
                let result = reader.process(black_box(&data)).ok().flatten();
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
    name = reader;
    config = Criterion::default();
    targets =
        xtea_decrypt,
        rsa_handshake,
        checksum_only,
        plaintext
);
criterion_main!(reader);
