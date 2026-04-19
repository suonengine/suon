use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, LitStr, parse_macro_input};

/// Expands `#[derive(LuaComponent)]` into the corresponding Bevy and `suon_lua`
/// trait implementations.
pub fn derive_lua_component(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    TokenStream::from(expand_derive_lua_component(derive_input))
}

/// Reads `#[lua(name = "...")]` from the derive input when present.
fn lua_name_from_attr(derive_input: &DeriveInput) -> Option<String> {
    for attr in &derive_input.attrs {
        if !attr.path().is_ident("lua") {
            continue;
        }

        let mut name = None;
        let _ = attr.parse_nested_meta(|context| {
            if context.path.is_ident("name") {
                let value: LitStr = context.value()?.parse()?;
                name = Some(value.value());
            }
            Ok(())
        });
        return name;
    }
    None
}

/// Builds the generated implementation for `LuaComponent`.
fn expand_derive_lua_component(derive_input: DeriveInput) -> TokenStream2 {
    let struct_name = &derive_input.ident;
    let (impl_generics, type_generics, where_clause) = derive_input.generics.split_for_impl();
    let lua_name = lua_name_from_attr(&derive_input).unwrap_or_else(|| struct_name.to_string());

    quote! {
        impl #impl_generics bevy::ecs::component::Component
            for #struct_name #type_generics #where_clause
        {
            const STORAGE_TYPE: bevy::ecs::component::StorageType =
                bevy::ecs::component::StorageType::Table;
            type Mutability = bevy::ecs::component::Mutable;

            fn on_add() -> Option<bevy::ecs::lifecycle::ComponentHook> {
                Some(|mut world, _context| {
                    if !world
                        .resource::<suon_lua::prelude::ScriptRegistry>()
                        .has_component(<#struct_name as suon_lua::prelude::LuaComponent>::lua_name())
                    {
                        world.resource_mut::<suon_lua::prelude::ScriptRegistry>().register_component(
                            <#struct_name as suon_lua::prelude::LuaComponent>::lua_name(),
                            <#struct_name as suon_lua::prelude::LuaComponent>::make_accessor(),
                        );
                    }
                })
            }
        }

        impl #impl_generics suon_lua::prelude::LuaComponent for #struct_name #type_generics #where_clause {
            fn lua_name() -> &'static str {
                #lua_name
            }

            fn make_accessor() -> suon_lua::prelude::ComponentAccessor {
                suon_lua::prelude::ComponentAccessor {
                    get: |entity, world| {
                        <bevy::prelude::World as suon_lua::prelude::WorldLuaComponentExt>
                            ::serialize_lua_component::<Self>(world, entity)
                    },
                    set: |entity, world, json| {
                        <bevy::prelude::World as suon_lua::prelude::WorldLuaComponentExt>
                            ::deserialize_lua_component::<Self>(world, entity, json)
                    },
                    component_id: |world| world.register_component::<Self>(),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_input(source: &str) -> DeriveInput {
        match syn::parse_str(source) {
            Ok(input) => input,
            Err(error) => panic!("input should parse: {error}"),
        }
    }

    #[test]
    fn generates_component_impl_with_table_storage() {
        let input = parse_input("struct Health { value: i32 }");
        let output = expand_derive_lua_component(input).to_string();

        assert!(
            output.contains("impl bevy :: ecs :: component :: Component for Health"),
            "should generate Component impl for Health"
        );

        assert!(
            output.contains("StorageType :: Table"),
            "storage type should default to Table"
        );
    }

    #[test]
    fn generates_on_add_hook_that_registers_in_script_registry() {
        let input = parse_input("struct Health { value: i32 }");
        let output = expand_derive_lua_component(input).to_string();

        assert!(
            output.contains("fn on_add"),
            "Component impl should define an on_add hook"
        );

        assert!(
            output.contains("suon_lua :: prelude :: ScriptRegistry"),
            "on_add hook should reference ScriptRegistry"
        );

        assert!(
            output.contains("register_component"),
            "on_add hook should call register_component"
        );
    }

    #[test]
    fn hook_checks_registry_before_registering() {
        let input = parse_input("struct Health { value: i32 }");
        let output = expand_derive_lua_component(input).to_string();

        assert!(
            output.contains("has_component"),
            "on_add hook should guard against duplicate registration"
        );
    }

    #[test]
    fn generates_lua_component_impl_with_struct_name_as_lua_name() {
        let input = parse_input("struct Health { value: i32 }");
        let output = expand_derive_lua_component(input).to_string();

        assert!(
            output.contains("impl suon_lua :: prelude :: LuaComponent for Health"),
            "should generate LuaComponent impl for Health"
        );

        assert!(
            output.contains("\"Health\""),
            "lua_name should default to the struct name"
        );
    }

    #[test]
    fn generates_make_accessor_with_helper_fns() {
        let input = parse_input("struct Health { value: i32 }");
        let output = expand_derive_lua_component(input).to_string();

        assert!(
            output.contains("WorldLuaComponentExt"),
            "make_accessor should route through WorldLuaComponentExt"
        );

        assert!(
            output.contains("serialize_lua_component"),
            "make_accessor get field should use serialize_lua_component"
        );

        assert!(
            output.contains("deserialize_lua_component"),
            "make_accessor set field should use deserialize_lua_component"
        );

        assert!(
            output.contains("register_component :: < Self >"),
            "make_accessor component_id field should register the component directly"
        );
    }

    #[test]
    fn lua_attr_name_overrides_struct_name() {
        let input = parse_input(r#"#[lua(name = "HP")] struct Health { value: i32 }"#);
        let output = expand_derive_lua_component(input).to_string();

        assert!(
            output.contains("\"HP\""),
            "lua_name should use the value from #[lua(name = ...)]"
        );

        assert!(
            !output.contains("\"Health\""),
            "struct name should not appear as the lua_name when overridden"
        );
    }

    #[test]
    fn generates_correct_impl_for_generic_struct() {
        let input = parse_input("struct Container<T>(T);");
        let output = expand_derive_lua_component(input).to_string();

        assert!(
            output.contains("impl < T > bevy :: ecs :: component :: Component for Container < T >"),
            "Component generics should be forwarded"
        );

        assert!(
            output.contains("impl < T > suon_lua :: prelude :: LuaComponent for Container < T >"),
            "LuaComponent generics should be forwarded"
        );
    }

    #[test]
    fn generates_correct_impl_with_existing_where_clause() {
        let input = parse_input("struct Container<T>(T) where T: Clone;");
        let output = expand_derive_lua_component(input).to_string();

        assert!(
            output.contains("where T : Clone"),
            "existing where clause should be preserved in both impls"
        );
    }

    #[test]
    fn lua_name_fn_returns_static_str() {
        let input = parse_input("struct Mana { points: f32 }");
        let output = expand_derive_lua_component(input).to_string();

        assert!(
            output.contains("fn lua_name () -> & 'static str"),
            "lua_name should return &'static str"
        );
    }

    #[test]
    fn generates_correct_impl_for_unit_struct() {
        let input = parse_input("struct Marker;");
        let output = expand_derive_lua_component(input).to_string();

        assert!(
            output.contains("impl bevy :: ecs :: component :: Component for Marker"),
            "Component impl should support unit structs"
        );

        assert!(
            output.contains("impl suon_lua :: prelude :: LuaComponent for Marker"),
            "LuaComponent impl should support unit structs"
        );
    }

    #[test]
    fn generates_correct_impl_for_tuple_struct() {
        let input = parse_input("struct Health(i32);");
        let output = expand_derive_lua_component(input).to_string();

        assert!(
            output.contains("impl bevy :: ecs :: component :: Component for Health"),
            "Component impl should support tuple structs"
        );

        assert!(
            output.contains("impl suon_lua :: prelude :: LuaComponent for Health"),
            "LuaComponent impl should support tuple structs"
        );
    }
}
