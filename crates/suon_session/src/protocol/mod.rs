mod handshake;
mod keep_alive;
mod logout;
mod ping;

use bevy::prelude::*;

pub(super) struct SessionProtocolPlugin;

impl Plugin for SessionProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handshake::on_server_name)
            .add_observer(keep_alive::on_keep_alive)
            .add_observer(ping::on_ping)
            .add_observer(logout::on_logout);
    }
}
