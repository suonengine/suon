use bevy::prelude::*;

use crate::{
    browse::{BrowseMarket, BrowseMarketIntent},
    session::MarketSession,
};

/// Opens or refreshes the market session for a successful browse intent.
pub(super) fn on_browse_market_intent(event: On<BrowseMarketIntent>, mut commands: Commands) {
    commands
        .entity(event.entity)
        .insert(MarketSession::new(Some(event.scope)));

    commands.trigger(BrowseMarket {
        entity: event.entity,
        scope: event.scope,
    });
}
