//! Benchmarks for the suon_lua crate.
//!
//! Measures VM creation, expression evaluation, function store/restore
//! roundtrip, and event dispatch throughput.

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use suon_lua::LuaVm;

fn bench_vm_creation(criterion: &mut Criterion) {
    criterion.bench_function("lua/vm_creation", |bencher| {
        bencher.iter(|| {
            black_box(LuaVm::new());
        });
    });
}

fn bench_expression_eval(criterion: &mut Criterion) {
    let vm = LuaVm::new();

    criterion.bench_function("lua/expression_eval", |bencher| {
        bencher.iter(|| {
            vm.execute(|lua| {
                let result: i32 = lua
                    .load("return 1 + 2 * 3")
                    .eval()
                    .expect("bench expression must evaluate");
                black_box(result);
            });
        });
    });
}

fn bench_store_restore_call(criterion: &mut Criterion) {
    let vm = LuaVm::new();

    criterion.bench_function("lua/store_restore_call", |bencher| {
        bencher.iter(|| {
            vm.execute(|lua| {
                let func = lua
                    .create_function(|_, ()| -> mlua::Result<()> { Ok(()) })
                    .expect("bench func creation must succeed");
                let id = vm.store(func).expect("bench store must succeed");
                let restored = vm.restore(id).expect("bench restore must succeed");
                restored.call::<()>(()).expect("bench call must succeed");
                vm.remove(id).expect("bench remove must succeed");
            });
        });
    });
}

fn bench_dispatch_no_events(criterion: &mut Criterion) {
    let vm = LuaVm::new();

    criterion.bench_function("lua/dispatch_no_events", |bencher| {
        bencher.iter(|| {
            vm.trigger_event("NonExistent", black_box(42))
                .expect("bench dispatch must succeed");
        });
    });
}

fn bench_dispatch_with_events(criterion: &mut Criterion) {
    let vm = LuaVm::new();

    vm.execute(|lua| {
        let class = lua
            .create_table()
            .expect("bench: failed to create event class table");

        let trigger = lua
            .create_function(|_, ()| Ok(true))
            .expect("bench: failed to create trigger function");

        class
            .set("trigger", trigger)
            .expect("bench: failed to set trigger");

        lua.globals()
            .set("BenchEvent", class)
            .expect("bench: failed to set BenchEvent global");
    });

    criterion.bench_function("lua/dispatch_with_events", |bencher| {
        bencher.iter(|| {
            vm.trigger_event("BenchEvent", ())
                .expect("bench dispatch must succeed");
        });
    });
}

criterion_group!(
    name = lua_benchmarks;
    config = Criterion::default();
    targets =
        bench_vm_creation,
        bench_expression_eval,
        bench_store_restore_call,
        bench_dispatch_no_events,
        bench_dispatch_with_events,
);
criterion_main!(lua_benchmarks);
