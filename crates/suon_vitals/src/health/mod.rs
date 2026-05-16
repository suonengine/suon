//! Current hit points components and flows.

pub(crate) mod clamp_health;
pub(crate) mod damage;
pub(crate) mod heal;

use bevy::{app::PluginGroupBuilder, prelude::*};

/// Current hit points.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Deref, DerefMut)]
pub struct Health(pub(crate) u32);

impl Health {
    pub fn is_zero(&self) -> bool {
        **self == 0
    }

    pub fn is_at_maximum(&self, maximum: &MaxHealth) -> bool {
        **self == **maximum
    }
}

/// Maximum hit points.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Deref, DerefMut)]
#[require(Health(0))]
pub struct MaxHealth(pub u32);

pub(crate) struct HealthPlugins;

impl PluginGroup for HealthPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(clamp_health::ClampHealthPlugin)
            .add(heal::HealPlugin)
            .add(damage::DamagePlugin)
    }
}

pub(crate) mod prelude {
    pub use super::{
        Health, MaxHealth,
        damage::{Damage, DamageIntent, DamageIntentError, DamageRejected},
        heal::{Heal, HealIntent, HealIntentError, HealRejected},
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_store_values() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, HealthPlugins));

        let entity = app.world_mut().spawn((Health(75), MaxHealth(150))).id();

        assert_eq!(
            **app
                .world()
                .get::<Health>(entity)
                .expect("Health should exist"),
            75
        );

        assert_eq!(
            **app
                .world()
                .get::<MaxHealth>(entity)
                .expect("MaxHealth should exist"),
            150
        );
    }

    #[test]
    fn should_insert_required_health_with_max_health() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, HealthPlugins));

        let entity = app.world_mut().spawn(MaxHealth(150)).id();

        assert_eq!(
            **app
                .world()
                .get::<Health>(entity)
                .expect("Health should exist"),
            0
        );
    }

    #[test]
    fn should_build_plugin() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, HealthPlugins));

        app.update();

        assert_eq!(std::mem::size_of::<HealthPlugins>(), 0);
    }

    #[test]
    fn should_report_zero_when_health_is_zero() {
        assert!(Health(0).is_zero());
    }

    #[test]
    fn should_not_report_zero_when_health_is_nonzero() {
        assert!(!Health(1).is_zero());
    }

    #[test]
    fn should_report_at_maximum_when_health_equals_max() {
        assert!(Health(100).is_at_maximum(&MaxHealth(100)));
    }

    #[test]
    fn should_not_report_at_maximum_when_health_is_below_max() {
        assert!(!Health(50).is_at_maximum(&MaxHealth(100)));
    }
}
