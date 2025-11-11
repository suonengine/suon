use bevy::prelude::*;

use crate::server::{
    connection::{limiter::Limiter, throttle::Throttle},
    settings::Settings,
};

/// Loads the server settings and initializes related resources.
pub(crate) fn initialize_settings(mut commands: Commands) {
    let settings = Settings::load_or_default().expect("Failed to load network server settings.");

    commands.insert_resource(Throttle::new(settings));
    commands.insert_resource(Limiter::new(settings));
    commands.insert_resource(settings);

    info!("Server settings initialized successfully.");
}
