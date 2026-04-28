use bevy::prelude::*;
use suon_protocol_client::prelude::MarketBrowseKind;

/// Intent requesting a market browse operation for the target actor.
#[derive(Debug, Clone, EntityEvent, PartialEq, Eq)]
pub struct BrowseMarketIntent {
    #[event_target]
    /// Entity that requested the browse action.
    pub entity: Entity,

    /// Resolved browse scope requested by the caller.
    pub scope: MarketBrowseKind,
}
