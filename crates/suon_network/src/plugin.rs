use std::sync::Arc;
use suon_app::{App, plugin::Plugin};
use suon_channel::buffer_pool::BufferPool;
use tracing::error;

use crate::{manager::NetworkManager, pool::NetworkBufferPool, settings::NetworkSettings};

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        let settings = NetworkSettings::load();

        let runtime = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(settings.worker_threads)
                .enable_io()
                .enable_time()
                .build()
                .expect("failed to build network tokio runtime"),
        );

        let buffer_pool = Arc::new(BufferPool::new(
            settings.buffer_pool.buffer_size,
            settings.buffer_pool.prealloc,
        ));
        let mut manager = NetworkManager::new(runtime, app.channel(), buffer_pool.clone());
        app.add_resource(NetworkBufferPool(buffer_pool));

        for server_settings in settings.server {
            let port = server_settings.port;
            let kind = server_settings.kind.as_str();
            if let Err(e) = manager.spawn_server(server_settings.clone()) {
                error!(target: "App", "Failed to spawn {kind} server on port {port}: {e}");
            }
        }

        app.add_resource(manager);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_plugin_build_does_not_panic() {
        let mut app = App::new();
        let plugin = NetworkPlugin;
        plugin.build(&mut app);
    }
}
