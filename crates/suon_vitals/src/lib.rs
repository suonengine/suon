//! Shared vital components and synchronization plugins.
//!
//! This crate provides small Bevy components for common gameplay values such as
//! health, mana, soul, stamina, capacity, progression, regeneration and costs.
//!
//! # Examples
//! ```rust
//! use bevy::prelude::*;
//! use suon_vitals::prelude::*;
//!
//! let mut app = App::new();
//! app.add_plugins((MinimalPlugins, VitalsPlugins));
//!
//! let entity = app
//!     .world_mut()
//!     .spawn((MaxHealth(150), MaxMana(40)))
//!     .id();
//!
//! app.world_mut().trigger(HealIntent { entity, amount: 150 });
//!
//! app.update();
//!
//! assert_eq!(
//!     **app
//!         .world()
//!         .get::<Health>(entity)
//!         .expect("Health should exist"),
//!     150
//! );
//! ```

use bevy::{app::PluginGroupBuilder, prelude::*};

mod capacity;
mod cost;
mod death;
mod flee_health;
mod health;
mod mana;
mod mana_shield;
mod progression;
mod regeneration;
mod soul;
mod speed;
mod stamina;

pub mod prelude {
    pub use crate::{
        VitalsPlugins, capacity::prelude::*, cost::prelude::*, death::prelude::*,
        flee_health::prelude::*, health::prelude::*, mana::prelude::*, mana_shield::prelude::*,
        progression::prelude::*, regeneration::prelude::*, soul::prelude::*, speed::prelude::*,
        stamina::prelude::*,
    };
}

/// Plugin group for vital component synchronization.
pub struct VitalsPlugins;

impl PluginGroup for VitalsPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add_group(capacity::CapacityPlugins)
            .add_group(health::HealthPlugins)
            .add_group(death::DeathPlugins)
            .add_group(flee_health::FleeHealthPlugins)
            .add_group(mana::ManaPlugins)
            .add_group(mana_shield::ManaShieldPlugins)
            .add_group(speed::SpeedPlugins)
            .add_group(soul::SoulPlugins)
            .add_group(stamina::StaminaPlugins)
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use std::time::Duration;

    use crate::prelude::*;

    #[test]
    fn should_expose_vitals_through_prelude() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, VitalsPlugins));

        let entity = app
            .world_mut()
            .spawn((
                Health(150),
                MaxHealth(150),
                Mana(40),
                MaxMana(40),
                ManaSpent(12),
                ManaShield(5),
                MaxManaShield(10),
                Soul(100),
                MaxSoul(100),
                Stamina::from_minutes(2520),
                MaxStamina::from_minutes(2520),
                Capacity(400),
            ))
            .id();

        app.world_mut().trigger(AddCapacityModifierIntent {
            entity,
            modifier: CapacityModifier::new(25),
        });

        app.world_mut().entity_mut(entity).insert((
            BaseSpeed(220),
            SpeedModifiers::new([SpeedModifier::new(10)]),
            Speed(230),
            Level(8),
            Experience(4200),
            MagicLevel(1),
            HealthGain(5),
            HealthGainTicks(6000),
            ManaGain(5),
            ManaGainTicks(6000),
            SoulGain(1),
            SoulGainTicks(120000),
            ManaCost(20),
            ManaPercentCost(10),
            SoulCost(1),
        ));

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<Health>(entity)
                .expect("Health should exist"),
            150
        );

        assert_eq!(
            **app
                .world()
                .get::<ManaSpent>(entity)
                .expect("ManaSpent should exist"),
            12
        );

        assert_eq!(
            **app
                .world()
                .get::<Capacity>(entity)
                .expect("Capacity should exist"),
            400
        );

        assert_eq!(
            **app
                .world()
                .get::<FreeCapacity>(entity)
                .expect("FreeCapacity should exist"),
            400
        );

        assert_eq!(
            **app
                .world()
                .get::<MaxCapacity>(entity)
                .expect("MaxCapacity should exist"),
            425
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
                .get::<Speed>(entity)
                .expect("Speed should exist"),
            230
        );
    }

    #[test]
    fn should_expose_cost_health_components_through_prelude() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, VitalsPlugins));

        let entity = app
            .world_mut()
            .spawn((HealthCost(10), HealthPercentCost(5), FleeHealth(30)))
            .id();

        app.update();

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
                .get::<FleeHealth>(entity)
                .expect("FleeHealth should exist"),
            30
        );
    }

    #[test]
    fn should_insert_required_vital_components() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, VitalsPlugins));

        let entity = app
            .world_mut()
            .spawn((
                MaxHealth(150),
                MaxMana(40),
                MaxManaShield(10),
                MaxSoul(100),
                MaxStamina::from_minutes(2520),
                Capacity(400),
                BaseSpeed(220),
            ))
            .id();

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<Health>(entity)
                .expect("Health should exist"),
            0
        );

        assert_eq!(
            **app.world().get::<Mana>(entity).expect("Mana should exist"),
            0
        );

        assert_eq!(
            **app
                .world()
                .get::<ManaShield>(entity)
                .expect("ManaShield should exist"),
            0
        );

        assert_eq!(
            **app.world().get::<Soul>(entity).expect("Soul should exist"),
            0
        );

        assert_eq!(
            **app
                .world()
                .get::<Stamina>(entity)
                .expect("Stamina should exist"),
            Duration::ZERO
        );

        assert_eq!(
            **app
                .world()
                .get::<FreeCapacity>(entity)
                .expect("FreeCapacity should exist"),
            400
        );

        assert_eq!(
            **app
                .world()
                .get::<MaxCapacity>(entity)
                .expect("MaxCapacity should exist"),
            400
        );

        assert!(
            app.world()
                .get::<SpeedModifiers>(entity)
                .expect("SpeedModifiers should exist")
                .is_empty()
        );

        assert_eq!(
            **app
                .world()
                .get::<Speed>(entity)
                .expect("Speed should exist"),
            220
        );
    }

    #[test]
    fn should_build_vitals_plugin_group() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, VitalsPlugins));

        app.update();

        assert_eq!(std::mem::size_of::<VitalsPlugins>(), 0);
    }
}
