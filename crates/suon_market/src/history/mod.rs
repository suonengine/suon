mod action;
mod entry;

use bevy::prelude::*;

pub use self::{
    action::{MarketHistoryAction, ParseMarketHistoryActionError},
    entry::{MarketHistoryEntry, MarketHistoryTable},
};

pub(crate) struct MarketHistoryPlugin;

pub(crate) use self::entry::MarketHistorySequence;

impl Plugin for MarketHistoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MarketHistorySequence>();
    }
}
