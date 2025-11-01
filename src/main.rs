use bevy::{
    app::ScheduleRunnerPlugin,
    diagnostic::{
        DiagnosticsPlugin, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
        LogDiagnosticsPlugin, SystemInformationDiagnosticsPlugin,
    },
    log::LogPlugin,
    prelude::*,
};
use std::time::Duration;

const N_THREADS: usize = 2;
const EVENT_LOOP: f64 = 1.0 / 60.0;
const FIXED_EVENT_LOOP: f64 = 1.0 / 20.0;

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins
                .set(TaskPoolPlugin {
                    task_pool_options: TaskPoolOptions::with_num_threads(N_THREADS),
                })
                .set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                    EVENT_LOOP,
                ))),
            DiagnosticsPlugin,
            LogPlugin::default(),
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin::default(),
            EntityCountDiagnosticsPlugin::default(),
            SystemInformationDiagnosticsPlugin,
        ))
        .insert_resource(Time::<Fixed>::from_seconds(FIXED_EVENT_LOOP))
        .run();
}
