use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, LitStr, parse_macro_input};

pub fn derive_lua_component(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    TokenStream::from(expand_derive_lua_component(derive_input))
}

fn lua_name_from_attr(derive_input: &DeriveInput) -> Option<String> {
    for attr in &derive_input.attrs {
        if !attr.path().is_ident("lua") {
            continue;
        }
        let mut name = None;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                let value: LitStr = meta.value()?.parse()?;
                name = Some(value.value());
            }
            Ok(())
        });
        return name;
    }
    None
}

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
                        .resource::<suon_lua::ScriptRegistry>()
                        .components
                        .contains_key(<#struct_name as suon_lua::LuaComponent>::lua_name())
                    {
                        world.resource_mut::<suon_lua::ScriptRegistry>().register_component(
                            <#struct_name as suon_lua::LuaComponent>::lua_name(),
                            <#struct_name as suon_lua::LuaComponent>::make_accessor(),
                        );
                    }
                })
            }
        }

        impl #impl_generics suon_lua::LuaComponent for #struct_name #type_generics #where_clause {
            fn lua_name() -> &'static str {
                #lua_name
            }

            fn make_accessor() -> suon_lua::ComponentAccessor {
                suon_lua::ComponentAccessor {
                    get: suon_lua::serialize_component::<Self>,
                    set: suon_lua::deserialize_component::<Self>,
                    component_id: suon_lua::register_component_id::<Self>,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_component_impl_with_table_storage() {
        let input: DeriveInput = syn::parse_str("struct Health { value: i32 }").unwrap();
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
        let input: DeriveInput = syn::parse_str("struct Health { value: i32 }").unwrap();
        let output = expand_derive_lua_component(input).to_string();

        assert!(
            output.contains("fn on_add"),
            "Component impl should define an on_add hook"
        );

        assert!(
            output.contains("suon_lua :: ScriptRegistry"),
            "on_add hook should reference ScriptRegistry"
        );

        assert!(
            output.contains("register_component"),
            "on_add hook should call register_component"
        );
    }

    #[test]
    fn hook_checks_registry_before_registering() {
        let input: DeriveInput = syn::parse_str("struct Health { value: i32 }").unwrap();
        let output = expand_derive_lua_component(input).to_string();

        assert!(
            output.contains("contains_key"),
            "on_add hook should guard against duplicate registration"
        );
    }

    #[test]
    fn generates_lua_component_impl_with_struct_name_as_lua_name() {
        let input: DeriveInput = syn::parse_str("struct Health { value: i32 }").unwrap();
        let output = expand_derive_lua_component(input).to_string();

        assert!(
            output.contains("impl suon_lua :: LuaComponent for Health"),
            "should generate LuaComponent impl for Health"
        );

        assert!(
            output.contains("\"Health\""),
            "lua_name should default to the struct name"
        );
    }

    #[test]
    fn generates_make_accessor_with_helper_fns() {
        let input: DeriveInput = syn::parse_str("struct Health { value: i32 }").unwrap();
        let output = expand_derive_lua_component(input).to_string();

        assert!(
            output.contains("suon_lua :: serialize_component :: < Self >"),
            "make_accessor get field should use serialize_component"
        );

        assert!(
            output.contains("suon_lua :: deserialize_component :: < Self >"),
            "make_accessor set field should use deserialize_component"
        );

        assert!(
            output.contains("suon_lua :: register_component_id :: < Self >"),
            "make_accessor component_id field should use register_component_id"
        );
    }

    #[test]
    fn lua_attr_name_overrides_struct_name() {
        let input: DeriveInput =
            syn::parse_str(r#"#[lua(name = "HP")] struct Health { value: i32 }"#).unwrap();
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
        let input: DeriveInput = syn::parse_str("struct Container<T>(T);").unwrap();
        let output = expand_derive_lua_component(input).to_string();

        assert!(
            output.contains("impl < T > bevy :: ecs :: component :: Component for Container < T >"),
            "Component generics should be forwarded"
        );

        assert!(
            output.contains("impl < T > suon_lua :: LuaComponent for Container < T >"),
            "LuaComponent generics should be forwarded"
        );
    }

    #[test]
    fn generates_correct_impl_with_existing_where_clause() {
        let input: DeriveInput = syn::parse_str("struct Container<T>(T) where T: Clone;").unwrap();
        let output = expand_derive_lua_component(input).to_string();

        assert!(
            output.contains("where T : Clone"),
            "existing where clause should be preserved in both impls"
        );
    }

    #[test]
    fn lua_name_fn_returns_static_str() {
        let input: DeriveInput = syn::parse_str("struct Mana { points: f32 }").unwrap();
        let output = expand_derive_lua_component(input).to_string();

        assert!(
            output.contains("fn lua_name () -> & 'static str"),
            "lua_name should return &'static str"
        );
    }
}
