use std::sync::Arc;

use mlua::{Error, Lua, Table};
use suon_app::{App, plugin::Plugin};
use suon_channel::BufferPool;
use suon_lua::LuaVm;
use tracing::error;

use crate::{
    connection::manager::ConnectionManager, connections::Connections, manager::NetworkManager,
    pool::NetworkBufferPool, settings::NetworkSettings,
};

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        let settings = NetworkSettings::load();

        let connection_manager = Arc::new(ConnectionManager::new(0));
        let connections = Connections {
            manager: connection_manager.clone(),
        };
        app.add_resource(connections.clone());

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
            if let Err(error) =
                manager.spawn_server(server_settings.clone(), connection_manager.clone())
            {
                error!(target: "App", "Failed to spawn {kind} server on port {port}: {error}");
            }
        }

        app.add_resource(manager);

        Self::register_connection_bindings(app, connections);
    }
}

impl NetworkPlugin {
    fn register_connection_bindings(app: &mut App, connections: Connections) {
        let vm = app.get_resource::<LuaVm>();

        vm.execute(move |lua: &Lua| {
            let globals = lua.globals();

            let Ok(connection) = globals.get::<Table>("Connection") else {
                error!(target: "App", "Connection global not found.");
                return;
            };

            let send_fn = {
                let connection_send = connections.clone();
                match lua.create_function(move |_, (table, data): (Table, String)| {
                    let id: u64 = table.raw_get("_id")?;
                    let bytes = data.as_bytes().to_vec();
                    connection_send
                        .send(id, bytes)
                        .map_err(|e| Error::external(format!("Connection:send failed: {e}")))
                }) {
                    Ok(func) => func,
                    Err(err) => {
                        error!(target: "App", "Failed to create Connection:send function: {err}");
                        return;
                    }
                }
            };

            if let Err(err) = connection.set("send", send_fn) {
                error!(target: "App", "Failed to register Connection:send: {err}");
            }

            let close_fn = {
                let connection_close = connections.clone();
                match lua.create_function(move |_, table: Table| {
                    let id: u64 = table.raw_get("_id")?;
                    connection_close
                        .close(id)
                        .map_err(|e| Error::external(format!("Connection:close failed: {e}")))
                }) {
                    Ok(func) => func,
                    Err(err) => {
                        error!(target: "App", "Failed to create Connection:close function: {err}");
                        return;
                    }
                }
            };

            if let Err(err) = connection.set("close", close_fn) {
                error!(target: "App", "Failed to register Connection:close: {err}");
            }

            let send_raw_fn = {
                let connection_send_raw = connections;
                match lua.create_function(move |_, (table, data): (Table, String)| {
                    let id: u64 = table.raw_get("_id")?;
                    let bytes = data.as_bytes().to_vec();
                    connection_send_raw
                        .send_raw(id, bytes)
                        .map_err(|e| Error::external(format!("Connection:sendRaw failed: {e}")))
                }) {
                    Ok(func) => func,
                    Err(err) => {
                        error!(target: "App", "Failed to create Connection:sendRaw function: {err}");
                        return;
                    }
                }
            };

            if let Err(err) = connection.set("sendRaw", send_raw_fn) {
                error!(target: "App", "Failed to register Connection:sendRaw: {err}");
            }
        });
    }
}
