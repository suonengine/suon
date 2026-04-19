use bevy::prelude::*;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use serde::{Deserialize, Serialize};
use std::hint::black_box;
use suon_lua::{
    AppLuaExt, LuaCommands, LuaComponent, LuaPlugin, LuaScript, ScriptRegistry,
    deserialize_component, register_component_id,
    runtime::{ComponentAccessor, LuaRuntime},
    serialize_component,
};
use suon_macros::LuaHook;

#[derive(Component, Serialize, Deserialize, Clone)]
struct Health {
    value: i32,
}

impl LuaComponent for Health {
    fn lua_name() -> &'static str {
        "Health"
    }

    fn make_accessor() -> ComponentAccessor {
        ComponentAccessor {
            get: serialize_component::<Health>,
            set: deserialize_component::<Health>,
            component_id: register_component_id::<Health>,
        }
    }
}

#[derive(LuaHook, Serialize)]
#[lua(name = "onTick")]
struct TickHook;

fn app_with_health() -> App {
    let mut app = App::new();
    app.add_plugins(LuaPlugin);
    app.register_lua_component::<Health>();
    app
}

fn with_runtime<R>(app: &mut App, f: impl FnOnce(&LuaRuntime, &mut World) -> R) -> R {
    let runtime = app
        .world_mut()
        .remove_non_send_resource::<LuaRuntime>()
        .expect("LuaRuntime should be present");
    let result = f(&runtime, app.world_mut());
    app.world_mut().insert_non_send_resource(runtime);
    result
}

fn benchmark_lua(c: &mut Criterion) {
    let mut group = c.benchmark_group("lua");

    let mut app_exec = app_with_health();
    group.bench_function("exec_simple_snippet", |b| {
        b.iter(|| {
            with_runtime(&mut app_exec, |runtime, world| {
                runtime
                    .scope(world)
                    .execute(black_box("x = 1 + 2"))
                    .expect("exec should succeed");
            });
        });
    });

    let mut app_cget = app_with_health();
    let entity_cget = app_cget.world_mut().spawn(Health { value: 100 }).id();
    group.bench_function("component_get", |b| {
        b.iter(|| {
            let json = serialize_component::<Health>(black_box(entity_cget), app_cget.world_mut())
                .expect("Health should be present");
            black_box(json);
        });
    });

    let mut app_cset = app_with_health();
    let entity_cset = app_cset.world_mut().spawn(Health { value: 0 }).id();
    let json = serde_json::json!({ "value": 50 });
    group.bench_function("component_set", |b| {
        b.iter(|| {
            deserialize_component::<Health>(
                black_box(entity_cset),
                app_cset.world_mut(),
                black_box(json.clone()),
            );
        });
    });

    let mut app_hook = app_with_health();
    let entity_hook = app_hook.world_mut().spawn_empty().id();
    let hook_source = "function Entity:onTick() x = 1 end";
    group.bench_function("call_hook", |b| {
        b.iter(|| {
            with_runtime(&mut app_hook, |runtime, world| {
                runtime
                    .scope(world)
                    .call_hook(
                        black_box(entity_hook),
                        black_box(hook_source),
                        black_box("onTick"),
                        serde_json::Value::Null,
                    )
                    .expect("hook should succeed");
            });
        });
    });

    for entity_count in [1usize, 10, 100, 1000] {
        let mut app = app_with_health();
        for i in 0..entity_count {
            app.world_mut().spawn(Health { value: i as i32 });
        }

        group.bench_with_input(
            BenchmarkId::new("query_entities", entity_count),
            &entity_count,
            |b, _| {
                b.iter(|| {
                    with_runtime(&mut app, |runtime, world| {
                        runtime
                            .scope(world)
                            .execute(black_box(
                                "local n = 0
                                 for id, hp in Query('Health'):iter() do
                                     n = n + hp.value
                                 end
                                 black_box = n",
                            ))
                            .expect("exec should succeed");
                    });
                });
            },
        );
    }

    let mut app_exec_cmd = app_with_health();
    let entity_exec_cmd = app_exec_cmd.world_mut().spawn(Health { value: 0 }).id();
    let exec_snippet = format!(
        "local e = Entity({}) e:set('Health', {{ value = 1 }})",
        entity_exec_cmd.to_bits()
    );
    group.bench_function("lua_exec_command_flush", |b| {
        b.iter(|| {
            app_exec_cmd
                .world_mut()
                .commands()
                .lua_execute(black_box(exec_snippet.clone()));
            app_exec_cmd.world_mut().flush();
        });
    });

    let mut app_hook_cmd = app_with_health();
    let entity_hook_cmd = app_hook_cmd
        .world_mut()
        .spawn((
            Health { value: 0 },
            LuaScript::new(
                "function Entity:onTick()
                     local hp = self:get('Health')
                     self:set('Health', { value = hp.value + 1 })
                 end",
            ),
        ))
        .id();
    group.bench_function("lua_hook_command_flush", |b| {
        b.iter(|| {
            app_hook_cmd
                .world_mut()
                .commands()
                .lua_hook(black_box(entity_hook_cmd), TickHook)
                .expect("hook should serialize");
            app_hook_cmd.world_mut().flush();
        });
    });

    group.bench_function("registry_register_component", |b| {
        b.iter(|| {
            let mut registry = ScriptRegistry::default();
            registry.register_component(
                black_box("Health"),
                ComponentAccessor {
                    get: serialize_component::<Health>,
                    set: deserialize_component::<Health>,
                    component_id: register_component_id::<Health>,
                },
            );
            black_box(registry);
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_lua);
criterion_main!(benches);
