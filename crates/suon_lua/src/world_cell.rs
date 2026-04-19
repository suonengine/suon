//! Thread-local world pointer shared between the Rust call stack and Lua callbacks.
//!
//! [`WorldContext`] sets the pointer for its lifetime; [`with`] retrieves it.
//! The design lets Lua callbacks reach the ECS without passing `&mut World` through
//! the mlua API, which does not support lifetimed state.

use bevy::prelude::World;
use std::{cell::Cell, marker::PhantomData};

thread_local! {
    static WORLD: Cell<*mut World> = const { Cell::new(std::ptr::null_mut()) };
}

/// RAII guard that makes `world` available to Lua callbacks for its lifetime.
///
/// Automatically clears the pointer on drop, so panics inside hooks are safe.
pub(crate) struct WorldContext<'world> {
    /// Lifetime marker tying the guard to the borrowed world reference.
    _world: PhantomData<&'world mut World>,
}

impl<'world> WorldContext<'world> {
    /// Publishes `world` to the thread-local bridge for the lifetime of the guard.
    pub(crate) fn enter(world: &'world mut World) -> Self {
        WORLD.with(|world_cell| world_cell.set(world as *mut World));
        Self {
            _world: PhantomData,
        }
    }
}

impl Drop for WorldContext<'_> {
    /// Clears the thread-local pointer when the guard goes out of scope.
    fn drop(&mut self) {
        WORLD.with(|world_cell| world_cell.set(std::ptr::null_mut()));
    }
}

/// Runs `callback` with exclusive world access. Only valid inside a [`WorldContext`].
///
/// This stays private because correctness depends on the lifecycle established by
/// [`WorldContext::enter`]: callers must ensure the thread-local pointer is live
/// only while the outer Rust stack still owns the corresponding `&mut World`.
pub(crate) fn with<R>(callback: impl FnOnce(&mut World) -> R) -> R {
    WORLD.with(|world_cell| {
        let world_ptr = world_cell.get();
        assert!(
            !world_ptr.is_null(),
            "world_cell accessed outside of a WorldContext"
        );
        // SAFETY: Two invariants make this sound:
        // (i)  The pointer is set in `WorldContext::enter` which ties its validity to 'world —
        //      the WorldContext RAII guard clears it on drop, so the pointer cannot dangle.
        // (ii) mlua's Lua is !Send, which means all Lua execution (and therefore all calls to
        //      `with`) happen on the thread that created the WorldContext, with no concurrent
        //      mutable access possible.
        unsafe { callback(&mut *world_ptr) }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[derive(Resource)]
    struct Marker;

    #[test]
    fn context_provides_access_to_the_correct_world() {
        let mut world = World::new();
        world.insert_resource(Marker);

        let has_marker = {
            let _context = WorldContext::enter(&mut world);
            with(|accessed_world| accessed_world.contains_resource::<Marker>())
        };

        assert!(has_marker);
    }

    #[test]
    #[should_panic(expected = "world_cell accessed outside of a WorldContext")]
    fn with_panics_when_no_context_is_active() {
        with(|_| ());
    }

    #[test]
    fn with_can_mutate_world_through_callback() {
        #[derive(Resource, Default)]
        struct Counter(u32);

        let mut world = World::new();
        world.init_resource::<Counter>();

        {
            let _context = WorldContext::enter(&mut world);
            with(|world| world.resource_mut::<Counter>().0 += 1);
            with(|world| world.resource_mut::<Counter>().0 += 1);
        }

        assert_eq!(world.resource::<Counter>().0, 2);
    }

    #[test]
    #[should_panic(expected = "world_cell accessed outside of a WorldContext")]
    fn ptr_is_cleared_when_context_drops() {
        let mut world = World::new();

        {
            let _ctx = WorldContext::enter(&mut world);
        }

        with(|_| ()); // ptr was cleared on drop — panics
    }

    #[test]
    fn second_enter_overwrites_pointer_and_inner_context_sees_new_world() {
        #[derive(Resource)]
        struct MarkerA;

        let mut world_a = World::new();
        world_a.insert_resource(MarkerA);
        let _ctx = WorldContext::enter(&mut world_a);

        #[derive(Resource)]
        struct MarkerB;

        let mut world_b = World::new();
        world_b.insert_resource(MarkerB);
        let _ctx = WorldContext::enter(&mut world_b);

        // Inside ctx the accessible world is world_b.
        let value = with(|world| world.contains_resource::<MarkerB>());
        assert!(value, "inner context should see world_b");

        // ctx dropped: pointer is now null. ctx drop also clears to null (idempotent).
    }
}
