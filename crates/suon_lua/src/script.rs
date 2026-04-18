use bevy::prelude::*;

/// Lua script source attached to an entity.
/// The script defines hook functions such as `function Entity:onTeleport(from, to)`.
#[derive(Component, Clone)]
pub struct LuaScript {
    source: String,
}

impl LuaScript {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
        }
    }

    pub fn source(&self) -> &str {
        &self.source
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_source_string() {
        let script = LuaScript::new("x = 1");
        assert_eq!(script.source(), "x = 1");
    }

    #[test]
    fn new_accepts_owned_string() {
        let script = LuaScript::new(String::from("y = 2"));
        assert_eq!(script.source(), "y = 2");
    }

    #[test]
    fn source_returns_full_content() {
        let src = "function Entity:onTick() end";
        assert_eq!(LuaScript::new(src).source(), src);
    }

    #[test]
    fn clone_preserves_source() {
        let original = LuaScript::new("z = 3");
        let cloned = original.clone();
        assert_eq!(cloned.source(), "z = 3");
    }

    #[test]
    fn new_stores_empty_string() {
        let script = LuaScript::new("");
        assert_eq!(script.source(), "");
    }

    #[test]
    fn source_with_unicode_characters() {
        let src = "-- 你好 🌍\nlocal x = 1";
        assert_eq!(LuaScript::new(src).source(), src);
    }

    #[test]
    fn source_with_special_lua_characters() {
        let src = "local t = { [1] = 'a', [\"b\"] = 2 }";
        assert_eq!(LuaScript::new(src).source(), src);
    }
}
