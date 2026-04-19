use ::benches::bench;
use bevy::prelude::*;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use suon_network::prelude::*;

fn benchmark_network(c: &mut Criterion) {
    let mut group = c.benchmark_group("network");

    group.bench_function(bench!("plugin_setup"), |b| {
        b.iter(|| {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins);
            app.add_plugins(NetworkPlugins);
            app
        })
    });

    group.bench_function(bench!("runtime/limiter_acquire_release"), |b| {
        b.iter(|| {
            let mut limiter = Limiter::from(SessionQuota {
                max_total: 64,
                max_per_address: 4,
            });
            let addr = "127.0.0.1:7172"
                .parse()
                .expect("The benchmark socket address should parse");

            limiter
                .try_acquire(addr)
                .expect("The limiter should accept the first acquisition");
            limiter.release(addr);
            limiter.total_active_sessions()
        })
    });

    for payload_size in [1usize, 32usize, 256usize] {
        group.bench_with_input(
            BenchmarkId::new(bench!("runtime/checksum_mode_display"), payload_size),
            &payload_size,
            |b, &payload_size| {
                b.iter(|| {
                    if payload_size % 2 == 0 {
                        ChecksumMode::Adler32.to_string()
                    } else {
                        ChecksumMode::Sequence(payload_size).to_string()
                    }
                })
            },
        );
    }

    group.bench_function(bench!("runtime/packet_policy_default"), |b| {
        b.iter(|| {
            let policy = PacketPolicy::default();
            (
                policy.incoming.server_name_max_length,
                policy.outgoing.max_length,
            )
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_network);
criterion_main!(benches);
