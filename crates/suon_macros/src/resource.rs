use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
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
/// use suon_macros::Table;
///
/// #[derive(Table)]
/// struct MyTable<T> {
///     value: T,
/// }
/// ```
///
/// The macro expands to an empty implementation block with trait bounds.
pub fn derive_table(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree representation of the struct.
    let ast = parse_macro_input!(input as DeriveInput);

    TokenStream::from(expand_derive_table(ast))
}

fn expand_derive_table(mut ast: DeriveInput) -> TokenStream2 {
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
    quote! {
        impl #impl_generics #trait_path for #struct_name #type_generics #where_clause {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_table_adds_impl_and_thread_safety_bounds() {
        let input: DeriveInput =
            syn::parse_str("struct Inventory<T>(T);").expect("Input should parse");
        let output = expand_derive_table(input).to_string();

        assert!(
            output.contains("impl < T > suon_database :: Table for Inventory < T >"),
            "The derive macro should generate a Table impl for the annotated type"
        );

        assert!(
            output.contains("Self : Send + Sync + 'static"),
            "The derive macro should inject Send + Sync + 'static bounds into the where clause"
        );
    }

    #[test]
    fn derive_table_preserves_existing_where_clause() {
        let input: DeriveInput =
            syn::parse_str("struct Inventory<T>(T) where T: Clone;").expect("Input should parse");
        let output = expand_derive_table(input).to_string();

        assert!(
            output.contains("where T : Clone , Self : Send + Sync + 'static"),
            "The derive macro should preserve existing where predicates while appending \
             thread-safety bounds"
        );
    }

    #[test]
    fn derive_table_supports_named_structs() {
        let input: DeriveInput =
            syn::parse_str("struct Inventory { slots: usize }").expect("Input should parse");
        let output = expand_derive_table(input).to_string();

        assert!(
            output.contains("impl suon_database :: Table for Inventory"),
            "The derive macro should support named-field structs"
        );
    }

    #[test]
    fn derive_table_supports_lifetime_and_type_generics() {
        let input: DeriveInput =
            syn::parse_str("struct Borrowed<'a, T>(&'a T);").expect("Input should parse");
        let output = expand_derive_table(input).to_string();

        assert!(
            output.contains("impl < 'a , T > suon_database :: Table for Borrowed < 'a , T >"),
            "The derive macro should carry lifetime and type generics into the generated impl"
        );

        assert!(
            output.contains("Self : Send + Sync + 'static"),
            "The generated impl should still add thread-safety bounds for generic structs"
        );
    }
}
