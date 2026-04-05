use bevy::prelude::*;
use criterion::{Criterion, criterion_group, criterion_main};
use suon_network::NetworkPlugins;

fn benchmark_network_plugin_setup(c: &mut Criterion) {
    c.bench_function("network/plugin_setup", |b| {
        b.iter(|| {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins);
            app.add_plugins(NetworkPlugins);
            app
        })
    });
}

criterion_group!(benches, benchmark_network_plugin_setup);
criterion_main!(benches);
