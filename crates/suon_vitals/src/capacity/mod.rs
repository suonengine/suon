//! Carrying capacity components and flows.

pub(crate) mod add_capacity_modifier;
pub(crate) mod capacity_available;
pub(crate) mod capacity_full;
pub(crate) mod consume_capacity;
pub(crate) mod remove_capacity_modifier;
pub(crate) mod restore_capacity;
pub(crate) mod sync_capacity;

use bevy::{app::PluginGroupBuilder, prelude::*};
use suon_uuid::{Uuid, UuidGenerator};

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Deref, DerefMut)]
#[require(CapacityModifiers = CapacityModifiers::empty(), MaxCapacity(0), FreeCapacity(0))]
pub struct Capacity(pub u32);

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Deref)]
pub struct CapacityModifierId(pub Uuid);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CapacityModifier {
    id: CapacityModifierId,
    pub value: i32,
}

impl CapacityModifier {
    pub fn new(value: i32) -> Self {
        Self {
            id: CapacityModifierId(UuidGenerator::generate_uuid()),
            value,
        }
    }

    pub fn generate(generator: &UuidGenerator, value: i32) -> Self {
        Self {
            id: CapacityModifierId(generator.generate()),
            value,
        }
    }

    pub fn id(&self) -> CapacityModifierId {
        self.id
    }
}

#[derive(Component, Clone, PartialEq, Eq, Debug)]
#[component(immutable)]
pub struct CapacityModifiers(Vec<CapacityModifier>);

impl CapacityModifiers {
    pub(crate) fn empty() -> Self {
        Self(Vec::new())
    }

    #[cfg(test)]
    pub(crate) fn new(modifiers: impl IntoIterator<Item = CapacityModifier>) -> Self {
        Self(modifiers.into_iter().collect())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &CapacityModifier> {
        self.0.iter()
    }

    pub fn total(&self) -> i32 {
        self.iter().map(|modifier| modifier.value).sum()
    }

    pub(crate) fn with_added(&self, modifier: CapacityModifier) -> Self {
        let mut modifiers = self.0.clone();

        match modifiers
            .iter_mut()
            .find(|current| current.id() == modifier.id())
        {
            Some(current) => *current = modifier,
            None => modifiers.push(modifier),
        }

        Self(modifiers)
    }

    pub(crate) fn without(&self, id: CapacityModifierId) -> Option<Self> {
        let mut modifiers = self.0.clone();
        let index = modifiers.iter().position(|modifier| modifier.id() == id)?;
        modifiers.remove(index);
        Some(Self(modifiers))
    }
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Deref, DerefMut)]
pub struct MaxCapacity(pub u32);

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Deref, DerefMut)]
pub struct FreeCapacity(u32);

impl FreeCapacity {
    pub fn is_zero(&self) -> bool {
        **self == 0
    }

    pub fn is_at_maximum(&self, maximum: &MaxCapacity) -> bool {
        **self == **maximum
    }
}

pub(crate) struct CapacityPlugins;

impl PluginGroup for CapacityPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(restore_capacity::RestoreCapacityPlugin)
            .add(consume_capacity::ConsumeCapacityPlugin)
            .add(add_capacity_modifier::AddCapacityModifierPlugin)
            .add(remove_capacity_modifier::RemoveCapacityModifierPlugin)
            .add(capacity_full::CapacityFullPlugin)
            .add(capacity_available::CapacityAvailablePlugin)
            .add(sync_capacity::SyncCapacityPlugin)
    }
}

pub(crate) mod prelude {
    pub use super::{
        Capacity, CapacityModifier, CapacityModifierId, CapacityModifiers, FreeCapacity,
        MaxCapacity,
        add_capacity_modifier::{
            AddCapacityModifierIntent, AddCapacityModifierIntentError, AddCapacityModifierRejected,
        },
        capacity_available::CapacityAvailable,
        capacity_full::CapacityFull,
        consume_capacity::{
            ConsumeCapacity, ConsumeCapacityIntent, ConsumeCapacityIntentError,
            ConsumeCapacityRejected,
        },
        remove_capacity_modifier::{
            RemoveCapacityModifierIntent, RemoveCapacityModifierIntentError,
            RemoveCapacityModifierRejected,
        },
        restore_capacity::{
            RestoreCapacity, RestoreCapacityIntent, RestoreCapacityIntentError,
            RestoreCapacityRejected,
        },
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_store_capacity_values() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, CapacityPlugins));

        let entity = app
            .world_mut()
            .spawn((
                Capacity(400),
                CapacityModifiers::new([CapacityModifier::new(50)]),
                MaxCapacity(450),
                FreeCapacity(250),
            ))
            .id();

        assert_eq!(
            **app
                .world()
                .get::<Capacity>(entity)
                .expect("Capacity should exist"),
            400
        );

        assert_eq!(
            app.world()
                .get::<CapacityModifiers>(entity)
                .expect("CapacityModifiers should exist")
                .iter()
                .next()
                .expect("Capacity modifier should exist")
                .value,
            50
        );

        assert_eq!(
            **app
                .world()
                .get::<MaxCapacity>(entity)
                .expect("MaxCapacity should exist"),
            450
        );

        assert_eq!(
            **app
                .world()
                .get::<FreeCapacity>(entity)
                .expect("FreeCapacity should exist"),
            250
        );
    }

    #[test]
    fn should_insert_required_capacity_components_with_capacity() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, CapacityPlugins));

        let entity = app.world_mut().spawn(Capacity(400)).id();

        assert!(
            app.world()
                .get::<CapacityModifiers>(entity)
                .expect("CapacityModifiers should exist")
                .is_empty()
        );

        assert_eq!(
            **app
                .world()
                .get::<MaxCapacity>(entity)
                .expect("MaxCapacity should exist"),
            400
        );

        assert_eq!(
            **app
                .world()
                .get::<FreeCapacity>(entity)
                .expect("FreeCapacity should exist"),
            400
        );
    }

    #[test]
    fn should_build_capacity_plugin() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, CapacityPlugins));

        app.update();

        assert_eq!(std::mem::size_of::<CapacityPlugins>(), 0);
    }

    #[test]
    fn should_report_free_capacity_as_zero() {
        assert!(FreeCapacity(0).is_zero());
    }

    #[test]
    fn should_not_report_free_capacity_as_zero_when_nonzero() {
        assert!(!FreeCapacity(1).is_zero());
    }

    #[test]
    fn should_report_free_capacity_at_maximum() {
        assert!(FreeCapacity(100).is_at_maximum(&MaxCapacity(100)));
    }

    #[test]
    fn should_not_report_free_capacity_at_maximum_when_below() {
        assert!(!FreeCapacity(50).is_at_maximum(&MaxCapacity(100)));
    }

    #[test]
    fn should_compute_total_of_modifiers() {
        let modifiers = CapacityModifiers::empty()
            .with_added(CapacityModifier::new(100))
            .with_added(CapacityModifier::new(-30));

        assert_eq!(modifiers.total(), 70);
    }

    #[test]
    fn should_replace_existing_modifier_with_same_id() {
        let modifier = CapacityModifier::new(50);
        let updated = CapacityModifier {
            id: modifier.id(),
            value: 200,
        };
        let modifiers = CapacityModifiers::new([modifier]);
        let result = modifiers.with_added(updated);

        assert_eq!(result.iter().count(), 1);
        assert_eq!(
            result.iter().next().expect("modifier should exist").value,
            200
        );
    }

    #[test]
    fn should_remove_modifier_by_id() {
        let modifier = CapacityModifier::new(50);
        let modifiers = CapacityModifiers::new([modifier]);
        let result = modifiers.without(modifier.id()).expect("should remove");

        assert!(result.is_empty());
    }

    #[test]
    fn should_return_none_when_removing_unknown_id() {
        let modifiers = CapacityModifiers::new([CapacityModifier::new(50)]);
        let unknown = CapacityModifierId(suon_uuid::UuidGenerator::generate_uuid());

        assert!(modifiers.without(unknown).is_none());
    }
}
