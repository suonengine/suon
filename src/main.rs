use suon_app::App;
use suon_lua::LuaPlugin;

fn main() {
    App::new().add_plugin(LuaPlugin).run();
}
