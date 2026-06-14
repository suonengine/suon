use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use suon_network::{
    connection::{ConnectionHandle, ConnectionId, ConnectionManager},
    protocol::Command as TcpCommand,
    server::tcp::ProtocolSettings,
};

const PACKET_SIZES: &[usize] = &[
    0, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536,
];

fn connection_send(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("connection");

    for &size in PACKET_SIZES {
        let (tx, rx) = crossbeam_channel::bounded::<TcpCommand>(1024);
        let handle = ConnectionHandle::new(
            ConnectionId::new(0, 1),
            "127.0.0.1:7000"
                .parse()
                .expect("benchmark address should parse"),
            tx,
        );

        group.bench_function(format!("send/{}", format_size(size)), |bencher| {
            let data = vec![0xABu8; size];
            bencher.iter(|| {
                handle
                    .send(black_box(data.clone()))
                    .expect("bench send should not fail with bounded channel");
                drop(rx.try_recv());
            });
        });
    }

    group.finish();
}

fn manager_register(criterion: &mut Criterion) {
    let manager = ConnectionManager::new(0);
    let protocol = ProtocolSettings {
        header_size: 6,
        has_checksum: true,
        uses_xtea: true,
        uses_rsa: true,
    };

    let peer = "127.0.0.1:7000"
        .parse()
        .expect("benchmark address should parse");

    criterion.bench_function("manager/register", |bencher| {
        let (sender, _) = crossbeam_channel::bounded(16);
        bencher.iter(|| {
            let id = manager.register(black_box(peer), black_box(protocol), sender.clone());
            black_box(id);
        });
    });
}

fn manager_register_unregister(criterion: &mut Criterion) {
    let manager = ConnectionManager::new(0);
    let protocol = ProtocolSettings {
        header_size: 6,
        has_checksum: true,
        uses_xtea: true,
        uses_rsa: true,
    };

    let peer = "127.0.0.1:7000"
        .parse()
        .expect("benchmark address should parse");

    criterion.bench_function("manager/register_unregister", |bencher| {
        bencher.iter(|| {
            let (sender, _) = crossbeam_channel::bounded(16);
            let id = manager.register(peer, protocol, sender);
            manager.unregister(id);
        });
    });
}

fn manager_active_connections(criterion: &mut Criterion) {
    let manager = ConnectionManager::new(0);
    let protocol = ProtocolSettings {
        header_size: 6,
        has_checksum: true,
        uses_xtea: true,
        uses_rsa: true,
    };

    let peer = "127.0.0.1:7000"
        .parse()
        .expect("benchmark address should parse");

    let mut ids = Vec::new();
    for _ in 0..100 {
        let (sender, _) = crossbeam_channel::bounded(16);
        ids.push(manager.register(peer, protocol, sender));
    }

    criterion.bench_function("manager/active_connections_100", |bencher| {
        bencher.iter(|| {
            let list = manager.active_connections();
            black_box(list.len());
        });
    });

    for id in ids {
        manager.unregister(id);
    }
}

fn manager_concurrent_register(criterion: &mut Criterion) {
    use std::{sync::Arc, thread};

    let manager = Arc::new(ConnectionManager::new(0));
    let protocol = ProtocolSettings {
        header_size: 6,
        has_checksum: true,
        uses_xtea: true,
        uses_rsa: true,
    };

    let peer = "127.0.0.1:7000"
        .parse()
        .expect("benchmark address should parse");

    criterion.bench_function("manager/concurrent_register", |bencher| {
        bencher.iter(|| {
            let mgr = manager.clone();
            let mut threads = Vec::new();
            for _ in 0..4 {
                let m = mgr.clone();
                let p = peer;
                threads.push(thread::spawn(move || {
                    for _ in 0..25 {
                        let (sender, _) = crossbeam_channel::bounded(16);
                        let id = m.register(p, protocol, sender);
                        black_box(id);
                    }
                }));
            }
            for t in threads {
                t.join().expect("benchmark thread should join successfully");
            }
        });
    });
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
    name = connection;
    config = Criterion::default();
    targets =
        connection_send,
        manager_register,
        manager_register_unregister,
        manager_active_connections,
        manager_concurrent_register,
);
criterion_main!(connection);
