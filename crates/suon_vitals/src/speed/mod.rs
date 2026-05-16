//! Movement speed components and flows.

pub(crate) mod add_speed_modifier;
pub(crate) mod remove_speed_modifier;
pub(crate) mod set_base_speed;
pub(crate) mod speed_changed;
pub(crate) mod sync_speed;

use bevy::{app::PluginGroupBuilder, prelude::*};
use suon_uuid::{Uuid, UuidGenerator};

/// Base movement speed before modifiers.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Deref, DerefMut)]
#[require(SpeedModifiers = SpeedModifiers::empty(), Speed(0))]
pub struct BaseSpeed(pub u32);

/// Stable identifier for a speed modifier.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Deref)]
pub struct SpeedModifierId(pub Uuid);

/// Additive movement speed modifier.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SpeedModifier {
    id: SpeedModifierId,
    pub value: i32,
}

impl SpeedModifier {
    pub fn new(value: i32) -> Self {
        Self {
            id: SpeedModifierId(UuidGenerator::generate_uuid()),
            value,
        }
    }

    pub fn generate(generator: &UuidGenerator, value: i32) -> Self {
        Self {
            id: SpeedModifierId(generator.generate()),
            value,
        }
    }

    pub fn id(&self) -> SpeedModifierId {
        self.id
    }
}

/// Active additive movement speed modifiers.
#[derive(Component, Clone, PartialEq, Eq, Debug)]
pub struct SpeedModifiers(Vec<SpeedModifier>);

impl SpeedModifiers {
    pub(crate) fn empty() -> Self {
        Self(Vec::new())
    }

    #[cfg(test)]
    pub(crate) fn new(modifiers: impl IntoIterator<Item = SpeedModifier>) -> Self {
        Self(modifiers.into_iter().collect())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &SpeedModifier> {
        self.0.iter()
    }

    pub fn total(&self) -> i32 {
        self.iter().map(|modifier| modifier.value).sum()
    }

    pub(crate) fn with_added(&self, modifier: SpeedModifier) -> Self {
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

    pub(crate) fn without(&self, id: SpeedModifierId) -> Option<Self> {
        let mut modifiers = self.0.clone();
        let index = modifiers.iter().position(|modifier| modifier.id() == id)?;
        modifiers.remove(index);
        Some(Self(modifiers))
    }
}

/// Effective movement speed after modifiers.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Deref, DerefMut)]
pub struct Speed(pub(crate) i32);

pub(crate) struct SpeedPlugins;

impl PluginGroup for SpeedPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(set_base_speed::SetBaseSpeedPlugin)
            .add(add_speed_modifier::AddSpeedModifierPlugin)
            .add(remove_speed_modifier::RemoveSpeedModifierPlugin)
            .add(sync_speed::SyncSpeedPlugin)
    }
}

pub(crate) mod prelude {
    pub use super::{
        BaseSpeed, Speed, SpeedModifier, SpeedModifierId, SpeedModifiers,
        add_speed_modifier::{
            AddSpeedModifierIntent, AddSpeedModifierIntentError, AddSpeedModifierRejected,
        },
        remove_speed_modifier::{
            RemoveSpeedModifierIntent, RemoveSpeedModifierIntentError, RemoveSpeedModifierRejected,
        },
        set_base_speed::{SetBaseSpeedIntent, SetBaseSpeedIntentError, SetBaseSpeedRejected},
        speed_changed::SpeedChanged,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_store_speed_values() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SpeedPlugins));

        let entity = app
            .world_mut()
            .spawn((
                BaseSpeed(220),
                SpeedModifiers::new([SpeedModifier::new(-30)]),
                Speed(190),
            ))
            .id();

        assert_eq!(
            **app
                .world()
                .get::<BaseSpeed>(entity)
                .expect("BaseSpeed should exist"),
            220
        );

        assert_eq!(
            app.world()
                .get::<SpeedModifiers>(entity)
                .expect("SpeedModifiers should exist")
                .iter()
                .next()
                .expect("Speed modifier should exist")
                .value,
            -30
        );

        assert_eq!(
            **app
                .world()
                .get::<Speed>(entity)
                .expect("Speed should exist"),
            190
        );
    }

    #[test]
    fn should_insert_required_speed_components_with_base_speed() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SpeedPlugins));

        let entity = app.world_mut().spawn(BaseSpeed(220)).id();

        app.update();

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
    fn should_compute_total_of_speed_modifiers() {
        let modifiers = SpeedModifiers::empty()
            .with_added(SpeedModifier::new(50))
            .with_added(SpeedModifier::new(-20));

        assert_eq!(modifiers.total(), 30);
    }

    #[test]
    fn should_replace_speed_modifier_with_same_id() {
        let modifier = SpeedModifier::new(50);
        let replacement = SpeedModifier {
            id: modifier.id,
            value: 100,
        };
        let modifiers = SpeedModifiers::new([modifier]);
        let result = modifiers.with_added(replacement);

        assert_eq!(result.iter().count(), 1);
        assert_eq!(
            result.iter().next().expect("modifier should exist").value,
            100
        );
    }

    #[test]
    fn should_remove_speed_modifier_by_id() {
        let modifier = SpeedModifier::new(50);
        let modifiers = SpeedModifiers::new([modifier]);
        let result = modifiers.without(modifier.id).expect("should remove");

        assert!(result.is_empty());
    }

    #[test]
    fn should_return_none_when_removing_unknown_speed_modifier_id() {
        let modifiers = SpeedModifiers::new([SpeedModifier::new(50)]);
        let unknown = SpeedModifierId(suon_uuid::UuidGenerator::generate_uuid());

        assert!(modifiers.without(unknown).is_none());
    }
}
