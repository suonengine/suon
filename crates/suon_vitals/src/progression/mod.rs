//! Player progression components tied to vital calculations.

use bevy::prelude::*;

/// Player level.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Deref)]
pub struct Level(pub u32);

/// Player experience points.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Deref)]
pub struct Experience(pub u64);

/// Player magic level.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Deref)]
pub struct MagicLevel(pub u32);

/// Total spent mana used for magic progression.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Deref, DerefMut)]
pub struct ManaSpent(pub u64);

pub(crate) mod prelude {
    pub use super::{Experience, Level, MagicLevel, ManaSpent};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_store_progression_values() {
        let mut app = App::new();
        let entity = app
            .world_mut()
            .spawn((Level(8), Experience(4200), MagicLevel(1), ManaSpent(1000)))
            .id();

        assert_eq!(
            **app
                .world()
                .get::<Level>(entity)
                .expect("Level should exist"),
            8
        );

        assert_eq!(
            **app
                .world()
                .get::<Experience>(entity)
                .expect("Experience should exist"),
            4200
        );

        assert_eq!(
            **app
                .world()
                .get::<MagicLevel>(entity)
                .expect("MagicLevel should exist"),
            1
        );

        assert_eq!(
            **app
                .world()
                .get::<ManaSpent>(entity)
                .expect("ManaSpent should exist"),
            1000
        );
    }
}
