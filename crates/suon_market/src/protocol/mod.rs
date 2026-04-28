mod packet;

use bevy::prelude::*;

pub(crate) struct MarketProtocolPlugin;

impl Plugin for MarketProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(packet::on_market_packet);
    }
}
