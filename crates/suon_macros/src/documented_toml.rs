use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Field, Fields, Type, parse_macro_input};

pub fn derive_documented_toml(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = &input.ident;

    let Data::Struct(data) = &input.data else {
        return syn::Error::new_spanned(&input, "DocumentedToml only supports structs")
            .to_compile_error()
            .into();
    };

    let Fields::Named(fields) = &data.fields else {
        return syn::Error::new_spanned(&input, "DocumentedToml requires named fields")
            .to_compile_error()
            .into();
    };

    let struct_docs = doc_lines(&input.attrs);

    let field_docs = fields.named.iter().map(|field| {
        let field_ident = field.ident.as_ref().expect("named field");
        let field_name = field_serde_name(field).unwrap_or_else(|| field_ident.to_string());
        let docs = doc_lines(&field.attrs);
        let kind = field_kind(field);

        quote! {
            ::suon_serde::DocumentedField {
                name: #field_name,
                docs: &[#(#docs),*],
                kind: #kind,
            }
        }
    });

    quote! {
        impl ::suon_serde::DocumentedToml for #ident {
            fn documented_toml() -> ::suon_serde::DocumentedStruct {
                ::suon_serde::DocumentedStruct {
                    docs: &[#(#struct_docs),*],
                    fields: &[#(#field_docs),*],
                }
            }
        }
    }
    .into()
}

fn doc_lines(attrs: &[syn::Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .filter_map(|attr| match &attr.meta {
            syn::Meta::NameValue(meta) => match &meta.value {
                syn::Expr::Lit(expr) => match &expr.lit {
                    syn::Lit::Str(value) => Some(value.value().trim().to_string()),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        })
        .collect()
}

fn field_kind(field: &Field) -> proc_macro2::TokenStream {
    match &field.ty {
        Type::Path(path) => classify_path(
            path.path.segments.last().unwrap().ident.to_string(),
            &field.ty,
        ),
        _ => quote!(::suon_serde::DocumentedFieldKind::Value),
    }
}

fn field_serde_name(field: &Field) -> Option<String> {
    for attr in &field.attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }

        let mut renamed = None;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                renamed = Some(lit.value());
            }

            Ok(())
        });

        if renamed.is_some() {
            return renamed;
        }
    }

    None
}

fn classify_path(name: String, ty: &Type) -> proc_macro2::TokenStream {
    if name == "Option" {
        if let Some(inner) = first_generic_type(ty)
            && let Some(inner_name) = type_name(&inner)
            && is_nested_type(&inner_name)
        {
            return quote!(
                ::suon_serde::DocumentedFieldKind::OptionalTable {
                    docs: <#inner as ::suon_serde::DocumentedToml>::documented_toml,
                    sample: <#inner as ::suon_serde::DocumentedToml>::default_toml_value,
                }
            );
        }

        return quote!(::suon_serde::DocumentedFieldKind::Value);
    }

    if name == "Vec" {
        if let Some(inner) = first_generic_type(ty)
            && let Some(inner_name) = type_name(&inner)
            && is_nested_type(&inner_name)
        {
            return quote!(
                ::suon_serde::DocumentedFieldKind::ArrayOfTables {
                    docs: <#inner as ::suon_serde::DocumentedToml>::documented_toml,
                    sample: <#inner as ::suon_serde::DocumentedToml>::default_toml_value,
                }
            );
        }

        return quote!(::suon_serde::DocumentedFieldKind::Value);
    }

    if is_nested_type(&name) {
        quote!(
            ::suon_serde::DocumentedFieldKind::Table {
                docs: <#ty as ::suon_serde::DocumentedToml>::documented_toml,
            }
        )
    } else {
        quote!(::suon_serde::DocumentedFieldKind::Value)
    }
}

fn first_generic_type(ty: &Type) -> Option<Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };

    let segment = type_path.path.segments.last()?;
    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };

    arguments.args.iter().find_map(|argument| match argument {
        syn::GenericArgument::Type(inner) => Some(inner.clone()),
        _ => None,
    })
}

fn type_name(ty: &Type) -> Option<String> {
    let Type::Path(path) = ty else {
        return None;
    };

    Some(path.path.segments.last()?.ident.to_string())
}

fn is_nested_type(name: &str) -> bool {
    matches!(
        name,
        _ if name.ends_with("Settings")
            || name.ends_with("Policy")
            || name.ends_with("Quota")
            || name.ends_with("Rule")
    )
}
