use bevy::prelude::*;

/// Links a client entity to a persistent actor id for market lookups.
#[derive(Debug, Clone, Copy, Component, PartialEq, Eq)]
pub struct MarketActorRef {
    actor_id: u32,
}

impl MarketActorRef {
    /// Creates a new market actor reference.
    pub fn new(actor_id: u32) -> Self {
        Self { actor_id }
    }

    /// Returns the persistent actor identifier.
    pub fn actor_id(&self) -> u32 {
        self.actor_id
    }
}
