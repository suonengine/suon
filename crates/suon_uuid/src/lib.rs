//! UUID helpers used across the Suon workspace.

use bevy::prelude::*;

pub use uuid::Uuid;

/// Bevy plugin that exposes UUID generation as an ECS resource.
pub struct UuidPlugin;

impl Plugin for UuidPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UuidGenerator>();
    }
}

/// UUID generator resource.
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct UuidGenerator;

impl UuidGenerator {
    pub fn generate(&self) -> Uuid {
        Self::generate_uuid()
    }

    pub fn generate_uuid() -> Uuid {
        Uuid::now_v7()
    }
}

pub mod prelude {
    pub use crate::{Uuid, UuidGenerator, UuidPlugin};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_generate_unique_uuids() {
        let generator = UuidGenerator;
        let first = generator.generate();
        let second = generator.generate();
        assert_ne!(first, second);
    }

    #[test]
    fn should_register_uuid_generator_resource() {
        let mut app = App::new();
        app.add_plugins(UuidPlugin);

        assert!(app.world().get_resource::<UuidGenerator>().is_some());
    }
}
