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

impl NetworkPlugin {
    fn register_lua_bindings(app: &mut App) {
        let lua_vm = app.get_resource::<LuaVm>();
        let connections = app.get_resource::<Connections>();

        lua_vm.execute(move |lua: &Lua| {
            if let Err(error) = Self::inject_bindings(lua, connections.clone()) {
                error!(target: "App", "Failed to register Lua network bindings: {error}");
            }
        });
    }

    fn inject_bindings(lua: &Lua, connections: Connections) -> Result<(), Error> {
        let globals = lua.globals();

        let connection = globals
            .get::<Table>("Connection")
            .map_err(|_| Error::external("Connection global not found"))?;

        connection.set("send", {
            let connections_send = connections.clone();
            lua.create_function(move |_, (table, data): (Table, String)| {
                let id: u64 = table.raw_get("_id")?;
                let bytes = data.as_bytes().to_vec();
                connections_send
                    .send(id, bytes)
                    .map_err(|error| Error::external(format!("Connection:send failed: {error}")))
            })?
        })?;

        connection.set("close", {
            let connections_close = connections.clone();
            lua.create_function(move |_, table: Table| {
                let identifier: u64 = table.raw_get("_id")?;
                connections_close
                    .close(identifier)
                    .map_err(|error| Error::external(format!("Connection:close failed: {error}")))
            })?
        })?;

        Ok(())
    }
}

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        let settings = NetworkSettings::load();

        let connection_manager = Arc::new(ConnectionManager::new(0));
        let connections = Connections {
            manager: connection_manager.clone(),
        };
        app.add_resource(connections);

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

        Self::register_lua_bindings(app);
    }
}
