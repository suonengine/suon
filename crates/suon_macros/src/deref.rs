use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

pub fn derive_deref(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let target = single_field_type(&input);
    let access = field_accessor(&input);

    TokenStream::from(quote! {
        impl #impl_generics ::std::ops::Deref for #ident #ty_generics #where_clause {
            type Target = #target;

            fn deref(&self) -> &Self::Target {
                &#access
            }
        }
    })
}

pub fn derive_deref_mut(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let access = field_accessor(&input);

    TokenStream::from(quote! {
        impl #impl_generics ::std::ops::DerefMut for #ident #ty_generics #where_clause {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut #access
            }
        }
    })
}

fn single_field_type(input: &DeriveInput) -> syn::Type {
    match &input.data {
        Data::Struct(ds) => match &ds.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => fields
                .unnamed
                .first()
                .expect("guarded by len == 1")
                .ty
                .clone(),

            Fields::Named(fields) if fields.named.len() == 1 => fields
                .named
                .first()
                .expect("guarded by len == 1")
                .ty
                .clone(),

            Fields::Unit => panic!("Deref requires a struct with exactly one field"),
            _ => panic!("Deref requires a struct with exactly one field"),
        },
        _ => panic!("Deref can only be derived on structs"),
    }
}

fn field_accessor(input: &DeriveInput) -> proc_macro2::TokenStream {
    match &input.data {
        Data::Struct(ds) => match &ds.fields {
            Fields::Unnamed(_) => quote! { self.0 },
            Fields::Named(fields) => {
                let name = fields
                    .named
                    .first()
                    .expect("guarded by len == 1")
                    .ident
                    .as_ref()
                    .expect("named field always has an ident");

                quote! { self.#name }
            }
            Fields::Unit => unreachable!(),
        },
        _ => unreachable!(),
    }
}
