mod event;
mod intent;
mod system;

use bevy::prelude::*;

pub use self::{
    event::{BrowseMarket, BrowseMarketRejected},
    intent::BrowseMarketIntent,
};

pub(crate) struct MarketBrowsePlugin;

impl Plugin for MarketBrowsePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(system::on_browse_market_intent);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use suon_chunk::prelude::{Chunk, ChunkPlugins, Chunks};
    use suon_movement::prelude::{MovementPlugins, StepIntent, TeleportIntent};
    use suon_position::prelude::{Floor, Position};
    use suon_protocol_client::prelude::MarketBrowseKind;

    use crate::session::{MarketSession, MarketSessionPlugin};

    #[test]
    fn should_close_market_session_when_step_event_is_received() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ChunkPlugins);
        app.add_plugins(MovementPlugins);
        app.add_plugins(MarketSessionPlugin);

        const START: Position = Position { x: 10, y: 10 };
        const TARGET: Position = Position { x: 11, y: 10 };
        let chunk = app.world_mut().spawn(Chunk).id();
        app.insert_resource(Chunks::from_iter([(START, chunk), (TARGET, chunk)]));

        let client = app
            .world_mut()
            .spawn((
                MarketSession::new(Some(MarketBrowseKind::OwnOffers)),
                START,
                Floor { z: 0 },
            ))
            .id();

        app.world_mut().trigger(StepIntent {
            to: suon_position::prelude::Direction::East,
            entity: client,
        });
        app.update();

        assert!(
            app.world().get::<MarketSession>(client).is_none(),
            "A successful step should close the open market session"
        );
    }

    #[test]
    fn should_close_market_session_when_teleport_event_is_received() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ChunkPlugins);
        app.add_plugins(MovementPlugins);
        app.add_plugins(MarketSessionPlugin);

        const START: Position = Position { x: 20, y: 20 };
        const TARGET: Position = Position { x: 25, y: 20 };
        let start_chunk = app.world_mut().spawn(Chunk).id();
        let target_chunk = app.world_mut().spawn(Chunk).id();
        app.insert_resource(Chunks::from_iter([
            (START, start_chunk),
            (TARGET, target_chunk),
        ]));

        let client = app
            .world_mut()
            .spawn((
                MarketSession::new(Some(MarketBrowseKind::OwnHistory)),
                START,
            ))
            .id();

        app.world_mut().trigger(TeleportIntent {
            to: TARGET,
            floor: None,
            entity: client,
        });
        app.update();
        app.update();

        assert!(
            app.world().get::<MarketSession>(client).is_none(),
            "A successful teleport should close the open market session"
        );
    }
}
