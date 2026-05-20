use bevy::prelude::*;
use suon_network::prelude::*;
use suon_protocol_client::prelude::LogoutPacket;

use crate::component::Session;

pub(super) fn on_logout(
    event: On<Packet<LogoutPacket>>,
    mut commands: Commands,
    sessions: Query<&Session>,
) {
    let connection = event.entity();
    let Ok(session) = sessions.get(connection) else {
        return;
    };

    commands.entity(session.player).despawn();
    commands.entity(connection).remove::<Session>();
}
