mod actor;
mod component;
mod intent;
mod system;

use bevy::prelude::*;

pub use self::{actor::MarketActorRef, component::MarketSession, intent::CloseMarketSessionIntent};

pub(crate) struct MarketSessionPlugin;

impl Plugin for MarketSessionPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(system::on_close_market_session_intent)
            .add_observer(system::on_step_close_market_session)
            .add_observer(system::on_teleport_close_market_session);
    }
}
