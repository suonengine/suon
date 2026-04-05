use bevy::prelude::*;

use crate::server::connection::{Connection, outgoing::OutgoingConnections};

/// Processes and removes any finished connections.
pub(crate) fn cleanup_finished_connections(
    mut commands: Commands,
    outgoing_connections: Res<OutgoingConnections>,
    query: Query<&Connection>,
) {
    for (client, addr) in outgoing_connections.read() {
        if let Ok(connection) = query.get(client) {
            // Remove the connection if the address matches.
            if connection.addr() == addr {
                commands.entity(client).remove::<Connection>();

                info!("Removed outgoing connection for {addr} (client {client}).");
            } else {
                warn!(
                    "Address mismatch for client {client}: expected {addr}, found {}. Skipping \
                     removal.",
                    connection.addr()
                );
            }
        } else {
            debug!("No active connection found for {addr} (client {client}). Skipping cleanup.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::{connection::Connection, settings::PacketPolicy};
    use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

    fn build_connection(addr: SocketAddr) -> Connection {
        let (outgoing_sender, _outgoing_receiver) = crossbeam_channel::unbounded();
        let (_incoming_sender, incoming_receiver) = crossbeam_channel::unbounded();
        let (xtea_sender, _xtea_receiver) = tokio::sync::watch::channel(None);

        Connection::new(
            outgoing_sender,
            incoming_receiver,
            addr,
            xtea_sender,
            PacketPolicy::default(),
        )
    }

    #[test]
    fn should_remove_connections_that_match_the_finished_outgoing_address() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<OutgoingConnections>();
        app.add_systems(Update, cleanup_finished_connections);

        let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 7172));
        let client = app.world_mut().spawn(build_connection(addr)).id();

        app.world()
            .resource::<OutgoingConnections>()
            .send((client, addr))
            .expect("The finished connection queue should accept the test event");

        app.update();

        assert!(
            !app.world().entity(client).contains::<Connection>(),
            "cleanup_finished_connections should remove matching connection components"
        );
    }

    #[test]
    fn should_keep_connections_when_the_finished_address_does_not_match() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<OutgoingConnections>();
        app.add_systems(Update, cleanup_finished_connections);

        let actual_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 7172));
        let queued_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 7173));
        let client = app.world_mut().spawn(build_connection(actual_addr)).id();

        app.world()
            .resource::<OutgoingConnections>()
            .send((client, queued_addr))
            .expect("The finished connection queue should accept the test event");

        app.update();

        assert!(
            app.world().entity(client).contains::<Connection>(),
            "cleanup_finished_connections should ignore queued addresses that do not match"
        );
    }

    #[test]
    fn should_ignore_finished_connections_for_entities_that_no_longer_exist() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<OutgoingConnections>();
        app.add_systems(Update, cleanup_finished_connections);

        let missing_client = Entity::from_bits(99);
        let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 7172));

        app.world()
            .resource::<OutgoingConnections>()
            .send((missing_client, addr))
            .expect("The finished connection queue should accept events for missing entities");

        app.update();

        assert!(
            app.world().get_entity(missing_client).is_err(),
            "cleanup_finished_connections should leave missing entities untouched"
        );
    }
}
