use bevy::prelude::World;
use std::{cell::Cell, marker::PhantomData};

thread_local! {
    static WORLD: Cell<*mut World> = const { Cell::new(std::ptr::null_mut()) };
}

/// RAII guard that makes `world` available to Lua callbacks for its lifetime.
///
/// Automatically clears the pointer on drop, so panics inside hooks are safe.
// Usage:
//   let _context = WorldContext::enter(world);
//   exec_hook(...);  // callbacks can call world_cell::with(...)
//   // _context drops here, pointer cleared
pub(crate) struct WorldContext<'world> {
    _world: PhantomData<&'world mut World>,
}

impl<'world> WorldContext<'world> {
    pub(crate) fn enter(world: &'world mut World) -> Self {
        WORLD.with(|world_cell| world_cell.set(world as *mut World));
        Self {
            _world: PhantomData,
        }
    }
}

impl Drop for WorldContext<'_> {
    fn drop(&mut self) {
        WORLD.with(|world_cell| world_cell.set(std::ptr::null_mut()));
    }
}

/// Runs `callback` with exclusive world access. Only valid inside a [`WorldContext`].
pub(crate) fn with<R>(callback: impl FnOnce(&mut World) -> R) -> R {
    WORLD.with(|world_cell| {
        let world_ptr = world_cell.get();
        assert!(
            !world_ptr.is_null(),
            "world_cell accessed outside of a WorldContext"
        );
        // SAFETY: pointer is valid for 'world (tied to WorldContext lifetime).
        // Lua is !Send so no concurrent access is possible.
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
            let _context = WorldContext::enter(&mut world);
        }
        with(|_| ()); // ptr was cleared on drop — panics
    }

    #[test]
    fn second_enter_overwrites_pointer_and_inner_context_sees_new_world() {
        let mut world_a = World::new();
        let mut world_b = World::new();

        #[derive(Resource)]
        struct MarkerA;
        #[derive(Resource)]
        struct MarkerB;

        world_a.insert_resource(MarkerA);
        world_b.insert_resource(MarkerB);

        let _ctx_a = WorldContext::enter(&mut world_a);
        {
            // Second enter overwrites the pointer.
            let _ctx_b = WorldContext::enter(&mut world_b);
            // Inside ctx_b the accessible world is world_b.
            let has_b = with(|w| w.contains_resource::<MarkerB>());
            assert!(has_b, "inner context should see world_b");
        }
        // ctx_b dropped: pointer is now null. ctx_a drop also clears to null (idempotent).
    }
}
