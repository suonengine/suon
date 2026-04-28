use bevy::prelude::*;

/// Intent requesting that an open market session be closed.
#[derive(Debug, Clone, Copy, EntityEvent, PartialEq, Eq)]
pub struct CloseMarketSessionIntent {
    #[event_target]
    /// Entity whose market session should close.
    pub entity: Entity,
}
