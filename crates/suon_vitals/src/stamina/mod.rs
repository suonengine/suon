//! Current stamina points components and flows.

pub(crate) mod clamp_stamina;
pub(crate) mod consume_stamina;
pub(crate) mod restore_stamina;
pub(crate) mod stamina_exhausted;
pub(crate) mod stamina_recovered;

use bevy::{app::PluginGroupBuilder, prelude::*};
use std::time::Duration;

/// Current stamina duration.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Deref, DerefMut)]
pub struct Stamina(pub(crate) Duration);

impl Stamina {
    pub const fn from_minutes(minutes: u64) -> Self {
        Self(Duration::from_secs(minutes * 60))
    }

    pub fn is_zero(&self) -> bool {
        **self == Duration::ZERO
    }

    pub fn is_at_maximum(&self, maximum: &MaxStamina) -> bool {
        **self == **maximum
    }
}

/// Maximum stamina duration.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Deref, DerefMut)]
#[require(Stamina(Duration::ZERO))]
pub struct MaxStamina(pub Duration);

impl MaxStamina {
    pub const fn from_minutes(minutes: u64) -> Self {
        Self(Duration::from_secs(minutes * 60))
    }
}

pub(crate) struct StaminaPlugins;

impl PluginGroup for StaminaPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(clamp_stamina::ClampStaminaPlugin)
            .add(restore_stamina::RestoreStaminaPlugin)
            .add(consume_stamina::ConsumeStaminaPlugin)
            .add(stamina_recovered::StaminaRecoveredPlugin)
            .add(stamina_exhausted::StaminaExhaustedPlugin)
    }
}

pub(crate) mod prelude {
    pub use super::{
        MaxStamina, Stamina,
        consume_stamina::{
            ConsumeStamina, ConsumeStaminaIntent, ConsumeStaminaIntentError, ConsumeStaminaRejected,
        },
        restore_stamina::{
            RestoreStamina, RestoreStaminaIntent, RestoreStaminaIntentError, RestoreStaminaRejected,
        },
        stamina_exhausted::StaminaExhausted,
        stamina_recovered::StaminaRecovered,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_store_values() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StaminaPlugins));

        let entity = app
            .world_mut()
            .spawn((Stamina::from_minutes(75), MaxStamina::from_minutes(150)))
            .id();

        assert_eq!(
            **app
                .world()
                .get::<Stamina>(entity)
                .expect("Stamina should exist"),
            Duration::from_secs(75 * 60)
        );

        assert_eq!(
            **app
                .world()
                .get::<MaxStamina>(entity)
                .expect("MaxStamina should exist"),
            Duration::from_secs(150 * 60)
        );
    }

    #[test]
    fn should_insert_required_stamina_with_max_stamina() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StaminaPlugins));

        let entity = app.world_mut().spawn(MaxStamina::from_minutes(150)).id();

        assert_eq!(
            **app
                .world()
                .get::<Stamina>(entity)
                .expect("Stamina should exist"),
            Duration::ZERO
        );
    }

    #[test]
    fn should_build_plugin() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StaminaPlugins));

        app.update();

        assert_eq!(std::mem::size_of::<StaminaPlugins>(), 0);
    }

    #[test]
    fn should_report_zero_when_stamina_is_zero() {
        assert!(Stamina(Duration::ZERO).is_zero());
    }

    #[test]
    fn should_not_report_zero_when_stamina_is_nonzero() {
        assert!(!Stamina(Duration::from_secs(1)).is_zero());
    }

    #[test]
    fn should_store_sub_minute_precision() {
        let stamina = Stamina(Duration::from_millis(1500));
        assert_eq!(*stamina, Duration::from_millis(1500));
    }

    #[test]
    fn should_report_at_maximum_when_stamina_equals_max() {
        assert!(Stamina::from_minutes(2520).is_at_maximum(&MaxStamina::from_minutes(2520)));
    }

    #[test]
    fn should_not_report_at_maximum_when_stamina_is_below_max() {
        assert!(!Stamina::from_minutes(1000).is_at_maximum(&MaxStamina::from_minutes(2520)));
    }
}
