use bevy::prelude::*;
use suon_position::prelude::*;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Facing(pub Direction);

#[derive(Debug, Clone, Copy, EntityEvent, PartialEq, Eq)]
pub struct FaceIntent {
    #[event_target]
    pub entity: Entity,
    pub to: Direction,
}

pub(super) struct FacePlugin;

impl Plugin for FacePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(apply_face_intent);
    }
}

fn apply_face_intent(event: On<FaceIntent>, mut players: Query<&mut Facing>) {
    let entity = event.event_target();
    let Ok(mut facing) = players.get_mut(entity) else {
        return;
    };
    facing.0 = event.to;
}
