//! Spell and weapon vital cost components.

use bevy::prelude::*;

/// Flat health cost.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Deref, DerefMut)]
pub struct HealthCost(pub u32);

/// Health cost as a percentage of maximum health.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Deref, DerefMut)]
pub struct HealthPercentCost(pub u32);

/// Flat mana cost.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Deref, DerefMut)]
pub struct ManaCost(pub u32);

/// Mana cost as a percentage of maximum mana.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Deref, DerefMut)]
pub struct ManaPercentCost(pub u32);

/// Soul point cost.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Deref, DerefMut)]
pub struct SoulCost(pub u32);

pub(crate) mod prelude {
    pub use super::{HealthCost, HealthPercentCost, ManaCost, ManaPercentCost, SoulCost};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_store_cost_values() {
        let mut app = App::new();

        let entity = app
            .world_mut()
            .spawn((
                HealthCost(10),
                HealthPercentCost(5),
                ManaCost(20),
                ManaPercentCost(15),
                SoulCost(1),
            ))
            .id();

        assert_eq!(
            **app
                .world()
                .get::<HealthCost>(entity)
                .expect("HealthCost should exist"),
            10
        );

        assert_eq!(
            **app
                .world()
                .get::<HealthPercentCost>(entity)
                .expect("HealthPercentCost should exist"),
            5
        );

        assert_eq!(
            **app
                .world()
                .get::<ManaCost>(entity)
                .expect("ManaCost should exist"),
            20
        );

        assert_eq!(
            **app
                .world()
                .get::<ManaPercentCost>(entity)
                .expect("ManaPercentCost should exist"),
            15
        );

        assert_eq!(
            **app
                .world()
                .get::<SoulCost>(entity)
                .expect("SoulCost should exist"),
            1
        );
    }
}
