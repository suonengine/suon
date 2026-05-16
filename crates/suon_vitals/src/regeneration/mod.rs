//! Vital regeneration components.

use bevy::prelude::*;

/// Health points gained per regeneration tick.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Deref, DerefMut)]
pub struct HealthGain(pub u32);

/// Health regeneration interval in milliseconds.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Deref, DerefMut)]
pub struct HealthGainTicks(pub u32);

/// Mana points gained per regeneration tick.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Deref, DerefMut)]
pub struct ManaGain(pub u32);

/// Mana regeneration interval in milliseconds.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Deref, DerefMut)]
pub struct ManaGainTicks(pub u32);

/// Soul points gained per regeneration tick.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Deref, DerefMut)]
pub struct SoulGain(pub u32);

/// Soul regeneration interval in milliseconds.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Deref, DerefMut)]
pub struct SoulGainTicks(pub u32);

pub(crate) mod prelude {
    pub use super::{
        HealthGain, HealthGainTicks, ManaGain, ManaGainTicks, SoulGain, SoulGainTicks,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_store_regeneration_values() {
        let mut app = App::new();
        let entity = app
            .world_mut()
            .spawn((
                HealthGain(5),
                HealthGainTicks(6000),
                ManaGain(5),
                ManaGainTicks(6000),
                SoulGain(1),
                SoulGainTicks(120000),
            ))
            .id();

        assert_eq!(
            **app
                .world()
                .get::<HealthGain>(entity)
                .expect("HealthGain should exist"),
            5
        );

        assert_eq!(
            **app
                .world()
                .get::<HealthGainTicks>(entity)
                .expect("HealthGainTicks should exist"),
            6000
        );

        assert_eq!(
            **app
                .world()
                .get::<ManaGain>(entity)
                .expect("ManaGain should exist"),
            5
        );

        assert_eq!(
            **app
                .world()
                .get::<ManaGainTicks>(entity)
                .expect("ManaGainTicks should exist"),
            6000
        );

        assert_eq!(
            **app
                .world()
                .get::<SoulGain>(entity)
                .expect("SoulGain should exist"),
            1
        );

        assert_eq!(
            **app
                .world()
                .get::<SoulGainTicks>(entity)
                .expect("SoulGainTicks should exist"),
            120000
        );
    }
}
