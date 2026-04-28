use bevy::prelude::*;
use suon_movement::prelude::{Step, Teleport};

use crate::session::{CloseMarketSessionIntent, MarketSession};

/// Removes the open market session when a close intent is received.
pub(super) fn on_close_market_session_intent(
    event: On<CloseMarketSessionIntent>,
    mut commands: Commands,
    sessions: Query<(), With<MarketSession>>,
) {
    close_market_session(event.entity, &mut commands, &sessions);
}

/// Removes the open market session after a successful step.
pub(super) fn on_step_close_market_session(
    event: On<Step>,
    mut commands: Commands,
    sessions: Query<(), With<MarketSession>>,
) {
    close_market_session(event.event_target(), &mut commands, &sessions);
}

/// Removes the open market session after a successful teleport.
pub(super) fn on_teleport_close_market_session(
    event: On<Teleport>,
    mut commands: Commands,
    sessions: Query<(), With<MarketSession>>,
) {
    close_market_session(event.event_target(), &mut commands, &sessions);
}

fn close_market_session(
    entity: Entity,
    commands: &mut Commands,
    sessions: &Query<(), With<MarketSession>>,
) {
    if sessions.contains(entity) {
        commands.entity(entity).remove::<MarketSession>();
    }
}
