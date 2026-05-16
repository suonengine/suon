//! Current soul points components and flows.

pub(crate) mod clamp_soul;
pub(crate) mod consume_soul;
pub(crate) mod restore_soul;
pub(crate) mod soul_depleted;
pub(crate) mod soul_recovered;

use bevy::{app::PluginGroupBuilder, prelude::*};

/// Current soul points.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Deref, DerefMut)]
pub struct Soul(pub(crate) u32);

impl Soul {
    pub fn is_zero(&self) -> bool {
        **self == 0
    }

    pub fn is_at_maximum(&self, maximum: &MaxSoul) -> bool {
        **self == **maximum
    }
}

/// Maximum soul points.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Deref, DerefMut)]
#[require(Soul(0))]
pub struct MaxSoul(pub u32);

pub(crate) struct SoulPlugins;

impl PluginGroup for SoulPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(clamp_soul::ClampSoulPlugin)
            .add(restore_soul::RestoreSoulPlugin)
            .add(consume_soul::ConsumeSoulPlugin)
            .add(soul_recovered::SoulRecoveredPlugin)
            .add(soul_depleted::SoulDepletedPlugin)
    }
}

pub(crate) mod prelude {
    pub use super::{
        MaxSoul, Soul,
        consume_soul::{
            ConsumeSoul, ConsumeSoulIntent, ConsumeSoulIntentError, ConsumeSoulRejected,
        },
        restore_soul::{
            RestoreSoul, RestoreSoulIntent, RestoreSoulIntentError, RestoreSoulRejected,
        },
        soul_depleted::SoulDepleted,
        soul_recovered::SoulRecovered,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_store_values() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SoulPlugins));

        let entity = app.world_mut().spawn((Soul(75), MaxSoul(150))).id();

        assert_eq!(
            **app.world().get::<Soul>(entity).expect("Soul should exist"),
            75
        );

        assert_eq!(
            **app
                .world()
                .get::<MaxSoul>(entity)
                .expect("MaxSoul should exist"),
            150
        );
    }

    #[test]
    fn should_insert_required_soul_with_max_soul() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SoulPlugins));

        let entity = app.world_mut().spawn(MaxSoul(150)).id();

        assert_eq!(
            **app.world().get::<Soul>(entity).expect("Soul should exist"),
            0
        );
    }

    #[test]
    fn should_build_plugin() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SoulPlugins));

        app.update();

        assert_eq!(std::mem::size_of::<SoulPlugins>(), 0);
    }

    #[test]
    fn should_report_zero_when_soul_is_zero() {
        assert!(Soul(0).is_zero());
    }

    #[test]
    fn should_not_report_zero_when_soul_is_nonzero() {
        assert!(!Soul(1).is_zero());
    }

    #[test]
    fn should_report_at_maximum_when_soul_equals_max() {
        assert!(Soul(100).is_at_maximum(&MaxSoul(100)));
    }

    #[test]
    fn should_not_report_at_maximum_when_soul_is_below_max() {
        assert!(!Soul(50).is_at_maximum(&MaxSoul(100)));
    }
}
