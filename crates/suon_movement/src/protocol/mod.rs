use bevy::prelude::*;
use suon_network::prelude::*;
use suon_protocol_client::prelude::{CancelStepsPacket, FacePacket, StepPacket, StepsPacket};

use crate::orientation::FaceIntent;
use crate::prelude::{StepIntent, StepPath};

/// Maps a connection entity to its player entity.
///
/// Added to the connection entity by the session layer (suon_session)
/// when a player is fully authenticated.
#[derive(Component, Clone, Copy, Debug)]
pub struct PlayerLink(pub Entity);

pub(super) struct MovementProtocolPlugin;

impl Plugin for MovementProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_cancel_steps_packet)
            .add_observer(on_single_step)
            .add_observer(on_multi_step)
            .add_observer(on_face);
    }
}

fn on_cancel_steps_packet(
    event: On<Packet<CancelStepsPacket>>,
    mut commands: Commands,
    connections: Query<&PlayerLink>,
) {
    let connection = event.entity();
    let Ok(link) = connections.get(connection) else { return };
    commands.entity(link.0).remove::<StepPath>();
}

fn on_single_step(
    event: On<Packet<StepPacket>>,
    mut commands: Commands,
    connections: Query<&PlayerLink>,
) {
    let connection = event.entity();
    let Ok(link) = connections.get(connection) else { return };
    commands.trigger(StepIntent {
        entity: link.0,
        to: event.packet().direction,
    });
}

fn on_multi_step(
    event: On<Packet<StepsPacket>>,
    mut commands: Commands,
    connections: Query<&PlayerLink>,
) {
    let connection = event.entity();
    let Ok(link) = connections.get(connection) else { return };

    let path = &event.packet().path;
    let mut step_path = StepPath::default();
    for dir in path {
        step_path.push(*dir);
    }

    commands.entity(link.0).insert(step_path);
}

fn on_face(
    event: On<Packet<FacePacket>>,
    mut commands: Commands,
    connections: Query<&PlayerLink>,
) {
    let connection = event.entity();
    let Ok(link) = connections.get(connection) else { return };
    commands.trigger(FaceIntent {
        entity: link.0,
        to: event.packet().direction,
    });
}
