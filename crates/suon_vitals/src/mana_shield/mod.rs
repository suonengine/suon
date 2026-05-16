//! Current mana shield points components and flows.

pub(crate) mod absorb_mana_shield;
pub(crate) mod clamp_mana_shield;
pub(crate) mod mana_shield_activated;
pub(crate) mod mana_shield_broken;
pub(crate) mod restore_mana_shield;

use bevy::{app::PluginGroupBuilder, prelude::*};

/// Current mana shield points.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Deref, DerefMut)]
pub struct ManaShield(pub(crate) u32);

impl ManaShield {
    pub fn is_zero(&self) -> bool {
        **self == 0
    }

    pub fn is_at_maximum(&self, maximum: &MaxManaShield) -> bool {
        **self == **maximum
    }
}

/// Maximum mana shield points.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Deref, DerefMut)]
#[require(ManaShield(0))]
pub struct MaxManaShield(pub u32);

pub(crate) struct ManaShieldPlugins;

impl PluginGroup for ManaShieldPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(clamp_mana_shield::ClampManaShieldPlugin)
            .add(restore_mana_shield::RestoreManaShieldPlugin)
            .add(absorb_mana_shield::AbsorbManaShieldPlugin)
            .add(mana_shield_activated::ManaShieldActivatedPlugin)
            .add(mana_shield_broken::ManaShieldBrokenPlugin)
    }
}

pub(crate) mod prelude {
    pub use super::{
        ManaShield, MaxManaShield,
        absorb_mana_shield::{
            AbsorbManaShield, AbsorbManaShieldIntent, AbsorbManaShieldIntentError,
            AbsorbManaShieldRejected,
        },
        mana_shield_activated::ManaShieldActivated,
        mana_shield_broken::ManaShieldBroken,
        restore_mana_shield::{
            RestoreManaShield, RestoreManaShieldIntent, RestoreManaShieldIntentError,
            RestoreManaShieldRejected,
        },
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_store_values() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ManaShieldPlugins));

        let entity = app
            .world_mut()
            .spawn((ManaShield(75), MaxManaShield(150)))
            .id();

        assert_eq!(
            **app
                .world()
                .get::<ManaShield>(entity)
                .expect("ManaShield should exist"),
            75
        );

        assert_eq!(
            **app
                .world()
                .get::<MaxManaShield>(entity)
                .expect("MaxManaShield should exist"),
            150
        );
    }

    #[test]
    fn should_insert_required_mana_shield_with_max_mana_shield() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ManaShieldPlugins));

        let entity = app.world_mut().spawn(MaxManaShield(150)).id();

        assert_eq!(
            **app
                .world()
                .get::<ManaShield>(entity)
                .expect("ManaShield should exist"),
            0
        );
    }

    #[test]
    fn should_build_plugin() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ManaShieldPlugins));

        app.update();

        assert_eq!(std::mem::size_of::<ManaShieldPlugins>(), 0);
    }

    #[test]
    fn should_report_zero_when_mana_shield_is_zero() {
        assert!(ManaShield(0).is_zero());
    }

    #[test]
    fn should_not_report_zero_when_mana_shield_is_nonzero() {
        assert!(!ManaShield(1).is_zero());
    }

    #[test]
    fn should_report_at_maximum_when_mana_shield_equals_max() {
        assert!(ManaShield(100).is_at_maximum(&MaxManaShield(100)));
    }

    #[test]
    fn should_not_report_at_maximum_when_mana_shield_is_below_max() {
        assert!(!ManaShield(50).is_at_maximum(&MaxManaShield(100)));
    }
}
