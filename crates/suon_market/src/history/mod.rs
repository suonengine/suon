mod action;
mod entry;
mod system;

use bevy::prelude::*;

pub use self::{
    action::{MarketHistoryAction, ParseMarketHistoryActionError},
    entry::MarketHistoryEntry,
};

pub(crate) struct MarketHistoryPlugin;

impl Plugin for MarketHistoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(system::on_market_offer_created)
            .add_observer(system::on_market_offer_cancelled)
            .add_observer(system::on_market_offer_accepted);
    }
}
