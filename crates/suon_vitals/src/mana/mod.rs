//! Current mana points components and flows.

pub(crate) mod clamp_mana;
pub(crate) mod consume_mana;
pub(crate) mod mana_depleted;
pub(crate) mod mana_recovered;
pub(crate) mod restore_mana;

use bevy::{app::PluginGroupBuilder, prelude::*};

/// Current mana points.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Deref, DerefMut)]
pub struct Mana(pub(crate) u32);

impl Mana {
    pub fn is_zero(&self) -> bool {
        **self == 0
    }

    pub fn is_at_maximum(&self, maximum: &MaxMana) -> bool {
        **self == **maximum
    }
}

/// Maximum mana points.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Deref, DerefMut)]
#[require(Mana(0))]
pub struct MaxMana(pub u32);

pub(crate) struct ManaPlugins;

impl PluginGroup for ManaPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(clamp_mana::ClampManaPlugin)
            .add(restore_mana::RestoreManaPlugin)
            .add(consume_mana::ConsumeManaPlugin)
            .add(mana_recovered::ManaRecoveredPlugin)
            .add(mana_depleted::ManaDepletedPlugin)
    }
}

pub(crate) mod prelude {
    pub use super::{
        Mana, MaxMana,
        consume_mana::{
            ConsumeMana, ConsumeManaIntent, ConsumeManaIntentError, ConsumeManaRejected,
        },
        mana_depleted::ManaDepleted,
        mana_recovered::ManaRecovered,
        restore_mana::{
            RestoreMana, RestoreManaIntent, RestoreManaIntentError, RestoreManaRejected,
        },
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_store_values() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ManaPlugins));

        let entity = app.world_mut().spawn((Mana(75), MaxMana(150))).id();

        assert_eq!(
            **app.world().get::<Mana>(entity).expect("Mana should exist"),
            75
        );

        assert_eq!(
            **app
                .world()
                .get::<MaxMana>(entity)
                .expect("MaxMana should exist"),
            150
        );
    }

    #[test]
    fn should_insert_required_mana_with_max_mana() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ManaPlugins));

        let entity = app.world_mut().spawn(MaxMana(150)).id();

        assert_eq!(
            **app.world().get::<Mana>(entity).expect("Mana should exist"),
            0
        );
    }

    #[test]
    fn should_build_plugin() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ManaPlugins));

        app.update();

        assert_eq!(std::mem::size_of::<ManaPlugins>(), 0);
    }

    #[test]
    fn should_report_zero_when_mana_is_zero() {
        assert!(Mana(0).is_zero());
    }

    #[test]
    fn should_not_report_zero_when_mana_is_nonzero() {
        assert!(!Mana(1).is_zero());
    }

    #[test]
    fn should_report_at_maximum_when_mana_equals_max() {
        assert!(Mana(40).is_at_maximum(&MaxMana(40)));
    }

    #[test]
    fn should_not_report_at_maximum_when_mana_is_below_max() {
        assert!(!Mana(20).is_at_maximum(&MaxMana(40)));
    }
}
