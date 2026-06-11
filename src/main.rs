use suon_app::App;
use suon_lua::LuaPlugin;
use suon_network::NetworkPlugin;

fn main() {
    App::new()
        .add_plugin(LuaPlugin)
        .add_plugin(NetworkPlugin)
        .run();
}
