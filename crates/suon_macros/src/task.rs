use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

pub fn derive_task(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, generics, ..
    } = parse_macro_input!(input);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    TokenStream::from(quote! {
        impl #impl_generics ::suon_channel::IntoTask for #ident #ty_generics #where_clause {
            type Task = #ident #ty_generics;
            fn into_task(self) -> #ident #ty_generics { self }
        }
    })
}
