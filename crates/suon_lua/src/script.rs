//! [`LuaScript`] component — stores Lua source attached to an entity.
//!
//! Hook functions defined in the source (e.g. `function Entity:onTick()`) are
//! invoked by [`crate::LuaCommands::lua_hook`] at command-flush time.

use bevy::prelude::*;
use std::sync::Arc;

/// Lua source code attached to a Bevy entity.
///
/// Hook functions defined in the source are invoked by [`crate::LuaCommands::lua_hook`].
/// The conventional hook style is `function Entity:onTick() ... end`; the entity
/// itself is passed as `self` so scripts can call `self:get(...)` and `self:set(...)`.
///
/// # Examples
///
/// ```rust,ignore
/// commands.spawn(LuaScript::new(
///     "function Entity:onHeal()\
///         local hp = self:get('Health')\
///         self:set('Health', { value = hp.value + 10 })\
///     end",
/// ));
/// commands.lua_hook(entity, "onHeal");
/// ```
#[derive(Component, Clone)]
pub struct LuaScript {
    source: Arc<str>,
}

impl LuaScript {
    /// Creates a new script component with the given Lua `source`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use suon_lua::LuaScript;
    /// let script = LuaScript::new("function Entity:onTick() end");
    /// # let _ = script;
    /// ```
    pub fn new(source: impl Into<Arc<str>>) -> Self {
        Self {
            source: source.into(),
        }
    }

    /// Returns the stored Lua source.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use suon_lua::LuaScript;
    /// let script = LuaScript::new("print('hello')");
    /// assert_eq!(script.source(), "print('hello')");
    /// ```
    pub fn source(&self) -> &str {
        &self.source
    }

    pub(crate) fn shared_source(&self) -> Arc<str> {
        Arc::clone(&self.source)
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
