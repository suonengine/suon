//! Procedural macros shared across the Suon workspace.
//!
//! # Examples
//! ```ignore
//! use suon_macros::Table;
//!
//! #[derive(Table)]
//! struct Inventory {
//!     capacity: usize,
//! }
//! ```

mod documented_toml;
mod lua_component;
mod lua_hook;
mod table;

use proc_macro::TokenStream;

/// Procedural macro to automatically generate code for the `Table` trait.
///
/// This macro delegates the implementation to the `derive_table` function
/// defined in the `resource` module, passing along the input TokenStream.
///
/// # Usage
/// ```ignore
/// use suon_macros::Table;
///
/// #[derive(Table)]
/// struct MyTable {
///     id: u32,
/// }
/// ```
///
/// The macro expands into the necessary code to implement the `Table` trait
/// for the annotated struct, based on the logic in `resource::derive_table`.
#[proc_macro_derive(Table)]
pub fn derive_table(input: TokenStream) -> TokenStream {
    table::derive_table(input)
}

/// Derives `suon_lua::LuaComponent` for a Bevy component that implements
/// `serde::Serialize` and `serde::de::DeserializeOwned`.
///
/// The Lua-visible name defaults to the struct name. Override with
/// `#[lua(name = "CustomName")]`.
///
/// # Usage
/// ```ignore
/// use bevy::prelude::*;
/// use serde::{Deserialize, Serialize};
/// use suon_macros::LuaComponent;
///
/// #[derive(Component, Serialize, Deserialize, LuaComponent)]
/// struct Health { value: i32 }
///
/// // In plugin setup:
/// app.register_lua_component::<Health>();
/// ```
#[proc_macro_derive(LuaComponent, attributes(lua))]
pub fn derive_lua_component(input: TokenStream) -> TokenStream {
    lua_component::derive_lua_component(input)
}

/// Derives `suon_lua::Hook` for a serializable hook payload.
///
/// The Lua-visible hook name defaults to `on{StructName}`. Override with
/// `#[lua(name = "onCustom")]`.
///
/// # Usage
/// ```ignore
/// use serde::Serialize;
/// use suon_macros::LuaHook;
///
/// #[derive(Serialize, LuaHook)]
/// struct Move {
///     from: (i32, i32),
///     to: (i32, i32),
/// }
/// ```
#[proc_macro_derive(LuaHook, attributes(lua))]
pub fn derive_lua_hook(input: TokenStream) -> TokenStream {
    lua_hook::derive_lua_hook(input)
}

#[proc_macro_derive(DocumentedToml)]
pub fn derive_documented_toml(input: TokenStream) -> TokenStream {
    documented_toml::derive_documented_toml(input)
}
