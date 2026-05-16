//! Flee health threshold components and flows.

pub(crate) mod clamp_flee_health;
pub(crate) mod enter_flee_health;
pub(crate) mod exit_flee_health;

use bevy::{app::PluginGroupBuilder, prelude::*};

/// Health threshold used by monster AI to flee.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Deref, DerefMut)]
pub struct FleeHealth(pub u32);

pub(crate) struct FleeHealthPlugins;

impl PluginGroup for FleeHealthPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(enter_flee_health::EnterFleeHealthPlugin)
            .add(exit_flee_health::ExitFleeHealthPlugin)
            .add(clamp_flee_health::ClampFleeHealthPlugin)
    }
}

pub(crate) mod prelude {
    pub use super::{
        FleeHealth, enter_flee_health::EnterFleeHealth, exit_flee_health::ExitFleeHealth,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::health::*;

    #[test]
    fn should_store_flee_health() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, FleeHealthPlugins));

        let entity = app.world_mut().spawn((FleeHealth(25), MaxHealth(100))).id();

        assert_eq!(
            **app
                .world()
                .get::<FleeHealth>(entity)
                .expect("FleeHealth should exist"),
            25
        );
    }
}
