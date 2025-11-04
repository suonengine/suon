use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input, parse_quote};

/// Derives an implementation of the `Table` trait for the given struct.
///
/// This macro automatically implements the `Table` trait, adding a `where` clause
/// to ensure the struct is `Send`, `Sync`, and `'static`. This can be useful for
/// ensuring thread safety and static lifetime guarantees.
///
/// # Parameters
/// - `input`: The input TokenStream representing the annotated struct.
///
/// # Returns
/// - A TokenStream containing the generated implementation of the `Table` trait.
///
/// # Example
/// ```ignore
/// #[derive(Table)]
/// struct MyTable;
/// ```
///
/// The macro expands to an empty implementation block with trait bounds.
pub fn derive_table(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree representation of the struct.
    let mut ast = parse_macro_input!(input as DeriveInput);

    // Add a `where` clause to the impl to require that Self implements Send + Sync + 'static.
    ast.generics
        .make_where_clause()
        .predicates
        // Append the predicate: Self: Send + Sync + 'static
        .push(parse_quote! { Self: Send + Sync + 'static });

    // Extract the struct's identifier (name).
    let struct_name = &ast.ident;

    // Split the generics into parts for use in the impl block.
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

    // Path to the trait to be implemented.
    let trait_path = quote! { suon_database::Table };

    // Generate the impl block for the trait.
    TokenStream::from(quote! {
        impl #impl_generics #trait_path for #struct_name #type_generics #where_clause {}
    })
}
