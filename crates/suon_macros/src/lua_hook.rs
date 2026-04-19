use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, LitStr, parse_macro_input};

/// Expands `#[derive(LuaHook)]` into a `suon_lua::Hook` implementation.
pub fn derive_lua_hook(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    TokenStream::from(expand_derive_lua_hook(derive_input))
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

/// Builds the generated implementation for `Hook`.
fn expand_derive_lua_hook(derive_input: DeriveInput) -> TokenStream2 {
    let struct_name = &derive_input.ident;
    let (impl_generics, type_generics, where_clause) = derive_input.generics.split_for_impl();

    match derive_input.data {
        Data::Struct(_) => {}
        _ => panic!("LuaHook can only be derived for structs"),
    }

    let hook_name = lua_name_from_attr(&derive_input).unwrap_or_else(|| format!("on{struct_name}"));

    quote! {
        impl #impl_generics suon_lua::prelude::Hook for #struct_name #type_generics #where_clause {
            fn name() -> &'static str {
                #hook_name
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
    fn generates_hook_impl_with_default_name() {
        let input = parse_input("struct Move { x: i32, y: i32 }");
        let output = expand_derive_lua_hook(input).to_string();

        assert!(
            output.contains("impl suon_lua :: prelude :: Hook for Move"),
            "should generate Hook impl for Move"
        );

        assert!(
            output.contains("fn name () -> & 'static str"),
            "Hook impl should expose a name function"
        );

        assert!(
            output.contains("\"onMove\""),
            "default hook name should be derived from the struct name"
        );
    }

    #[test]
    fn lua_attr_name_overrides_default() {
        let input = parse_input(r#"#[lua(name = "onStep")] struct Move;"#);
        let output = expand_derive_lua_hook(input).to_string();

        assert!(
            output.contains("\"onStep\""),
            "hook name should use the value from #[lua(name = ...)]"
        );
    }

    #[test]
    fn generics_and_where_clause_are_preserved() {
        let input = parse_input("struct Event<T>(T) where T: Clone;");
        let output = expand_derive_lua_hook(input).to_string();

        assert!(
            output
                .contains("impl < T > suon_lua :: prelude :: Hook for Event < T > where T : Clone"),
            "generated impl should preserve generics and where clauses"
        );
    }
}
