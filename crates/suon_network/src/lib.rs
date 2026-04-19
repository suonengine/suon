//! Networking plugins and server infrastructure for Suon.
//!
//! # Examples
//! ```no_run
//! use bevy::prelude::*;
//! use suon_network::NetworkPlugins;
//!
//! let mut app = App::new();
//! app.add_plugins(MinimalPlugins);
//! app.add_plugins(NetworkPlugins);
//!
//! assert_eq!(std::mem::size_of::<NetworkPlugins>(), 0);
//! ```

use bevy::{app::PluginGroupBuilder, prelude::*};

use crate::server::NetworkServerPlugin;

mod server;

pub mod prelude {
    pub use crate::{
        NetworkPlugins,
        server::{
            connection::{
                Connection, WriteError,
                checksum_mode::ChecksumMode,
                limiter::{AcquireError, Limiter},
            },
            packet::{DecodeError, Packet},
            settings::{
                IncomingPacketPolicy, OutgoingPacketPolicy, PacketPolicy, PacketPolicyPenalty,
                SessionQuota, ThrottlePolicy,
            },
        },
    };
}

/// Plugin group that installs the Suon networking server runtime.
pub struct NetworkPlugins;

impl PluginGroup for NetworkPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(NetworkServerPlugin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_add_network_plugins_to_app() {
        let mut app = App::new();

        app.add_plugins(MinimalPlugins);
        app.add_plugins(NetworkPlugins);

        assert!(
            app.world()
                .contains_resource::<crate::server::connection::incoming::IncomingConnections>(),
            "Adding the network plugin group should initialize incoming connection state"
        );
        assert!(
            app.world()
                .contains_resource::<crate::server::connection::outgoing::OutgoingConnections>(),
            "Adding the network plugin group should initialize outgoing connection state"
        );
    }
}
