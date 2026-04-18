use bevy::{ecs::component::ComponentId, prelude::*};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value as Json;

use crate::runtime::ComponentAccessor;

/// Implemented by components that can be read and written from Lua scripts.
///
/// Use `#[derive(LuaComponent)]` from `suon_macros`. The derive also generates
/// the [`bevy::prelude::Component`] impl — do **not** use `#[derive(Component)]`
/// alongside it. Registration in [`ScriptRegistry`] happens automatically the
/// first time the component is inserted into any entity.
///
/// ```ignore
/// #[derive(LuaComponent, Serialize, Deserialize)]
/// struct Health { value: i32 }
/// ```
pub trait LuaComponent: Component {
    /// The name exposed to Lua scripts — e.g. `"Health"`.
    fn lua_name() -> &'static str;

    /// Constructs a [`ComponentAccessor`] wired to this type's serde impls.
    fn make_accessor() -> ComponentAccessor;
}

/// Serializes the component on `entity` to JSON using serde.
///
/// Returns `None` when the entity does not have the component.
pub fn component_get<T>(entity: Entity, world: &mut World) -> Option<Json>
where
    T: Component + Serialize,
{
    let component = world.get::<T>(entity)?;
    serde_json::to_value(component).ok()
}

/// Deserializes `json` and inserts/replaces the component on `entity`.
///
/// Silently ignores JSON that cannot be deserialized into `T`.
pub fn component_set<T>(entity: Entity, world: &mut World, json: Json)
where
    T: Component + DeserializeOwned,
{
    if let Ok(component) = serde_json::from_value::<T>(json) {
        world.entity_mut(entity).insert(component);
    }
}

/// Returns the [`ComponentId`] for `T`, registering it with the world if needed.
pub fn component_register_id<T: Component>(world: &mut World) -> ComponentId {
    world.register_component::<T>()
}

/// Extension trait that lets `App` register a [`LuaComponent`] with one call.
pub trait AppLuaExt {
    /// Registers `T` in the [`ScriptRegistry`] under its [`LuaComponent::lua_name`].
    fn register_lua_component<T: LuaComponent>(&mut self) -> &mut Self;
}

impl AppLuaExt for App {
    fn register_lua_component<T: LuaComponent>(&mut self) -> &mut Self {
        self.world_mut()
            .resource_mut::<crate::runtime::ScriptRegistry>()
            .register_component(T::lua_name(), T::make_accessor());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{LuaRuntime, ScriptRegistry};
    use serde::{Deserialize, Serialize};

    #[derive(Component, Serialize, Deserialize)]
    struct Gold {
        amount: i32,
    }

    impl LuaComponent for Gold {
        fn lua_name() -> &'static str {
            "Gold"
        }

        fn make_accessor() -> ComponentAccessor {
            ComponentAccessor {
                get: component_get::<Gold>,
                set: component_set::<Gold>,
                component_id: component_register_id::<Gold>,
            }
        }
    }

    fn setup() -> (LuaRuntime, World) {
        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();
        world
            .resource_mut::<ScriptRegistry>()
            .register_component(Gold::lua_name(), Gold::make_accessor());
        (LuaRuntime::new(), world)
    }

    fn run(runtime: &LuaRuntime, world: &mut World, lua: &str) {
        runtime
            .scope(world)
            .exec(lua)
            .expect("lua exec should succeed");
    }

    #[test]
    fn component_get_serializes_to_json() {
        let mut world = World::new();
        let entity = world.spawn(Gold { amount: 42 }).id();
        let json =
            component_get::<Gold>(entity, &mut world).expect("Gold should be present on entity");
        assert_eq!(json["amount"], serde_json::json!(42));
    }

    #[test]
    fn component_get_returns_none_when_component_absent() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        assert!(component_get::<Gold>(entity, &mut world).is_none());
    }

    #[test]
    fn component_set_inserts_deserialized_component() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        component_set::<Gold>(entity, &mut world, serde_json::json!({ "amount": 99 }));
        assert_eq!(
            world
                .get::<Gold>(entity)
                .expect("Gold should be present")
                .amount,
            99
        );
    }

    #[test]
    fn component_set_updates_existing_component() {
        let mut world = World::new();
        let entity = world.spawn(Gold { amount: 1 }).id();
        component_set::<Gold>(entity, &mut world, serde_json::json!({ "amount": 50 }));
        assert_eq!(
            world
                .get::<Gold>(entity)
                .expect("Gold should be present")
                .amount,
            50
        );
    }

    #[test]
    fn component_set_ignores_malformed_json() {
        let mut world = World::new();
        let entity = world.spawn(Gold { amount: 7 }).id();
        component_set::<Gold>(entity, &mut world, serde_json::json!("not an object"));
        assert_eq!(
            world
                .get::<Gold>(entity)
                .expect("Gold should be present")
                .amount,
            7
        );
    }

    #[test]
    fn component_register_id_returns_stable_id() {
        let mut world = World::new();
        let first = component_register_id::<Gold>(&mut world);
        let second = component_register_id::<Gold>(&mut world);
        assert_eq!(first, second);
    }

    #[test]
    fn lua_name_is_correct() {
        assert_eq!(Gold::lua_name(), "Gold");
    }

    #[test]
    fn make_accessor_get_works_via_lua() {
        let (runtime, mut world) = setup();
        world.spawn(Gold { amount: 10 });

        run(
            &runtime,
            &mut world,
            "
            for id, gold in world:query('Gold'):iter() do
                assert(gold.amount == 10, 'expected 10, got ' .. tostring(gold.amount))
            end
        ",
        );
    }

    #[test]
    fn make_accessor_set_works_via_lua() {
        let (runtime, mut world) = setup();
        let entity = world.spawn(Gold { amount: 0 }).id();

        run(
            &runtime,
            &mut world,
            &format!(
                "
            local entity = world:entity({})
            entity:set('Gold', {{ amount = 77 }})
        ",
                entity.to_bits()
            ),
        );

        assert_eq!(
            world
                .get::<Gold>(entity)
                .expect("Gold should be present")
                .amount,
            77
        );
    }

    #[test]
    fn app_register_lua_component_adds_to_registry() {
        let mut app = App::new();
        app.init_resource::<ScriptRegistry>();
        app.register_lua_component::<Gold>();
        assert!(
            app.world()
                .resource::<ScriptRegistry>()
                .components
                .contains_key("Gold")
        );
    }

    #[test]
    fn component_get_returns_none_for_despawned_entity() {
        let mut world = World::new();
        let entity = world.spawn(Gold { amount: 5 }).id();
        world.despawn(entity);
        assert!(component_get::<Gold>(entity, &mut world).is_none());
    }

    #[test]
    fn component_set_with_null_json_does_not_insert_component() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        component_set::<Gold>(entity, &mut world, serde_json::Value::Null);
        assert!(world.get::<Gold>(entity).is_none());
    }

    #[test]
    fn component_roundtrip_via_get_then_set() {
        let mut world = World::new();
        let entity = world.spawn(Gold { amount: 33 }).id();
        let json = component_get::<Gold>(entity, &mut world).expect("should serialize");
        component_set::<Gold>(entity, &mut world, json);
        assert_eq!(world.get::<Gold>(entity).unwrap().amount, 33);
    }

    #[test]
    fn app_register_lua_component_is_callable_from_lua() {
        let mut app = App::new();
        app.init_resource::<ScriptRegistry>();
        app.register_lua_component::<Gold>();

        let entity = app.world_mut().spawn(Gold { amount: 5 }).id();
        let runtime = LuaRuntime::new();

        runtime
            .scope(app.world_mut())
            .exec(&format!(
                "
            local entity = world:entity({})
            local gold = entity:get('Gold')
            assert(gold ~= nil)
            assert(gold.amount == 5)
        ",
                entity.to_bits()
            ))
            .expect("lua exec should succeed");
    }
}
