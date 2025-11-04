mod resource;

use proc_macro::TokenStream;

/// Procedural macro to automatically generate code for the `Table` trait.
///
/// This macro delegates the implementation to the `derive_table` function
/// defined in the `resource` module, passing along the input TokenStream.
///
/// # Usage
/// ```ignore
/// #[derive(Table)]
/// struct MyTable;
/// ```
///
/// The macro expands into the necessary code to implement the `Table` trait
/// for the annotated struct, based on the logic in `resource::derive_table`.
///
/// # Arguments
/// - `input`: The input TokenStream representing the annotated item.
///
/// # Returns
/// - A TokenStream containing the generated code for the `Table` implementation.
#[proc_macro_derive(Table)]
pub fn derive_table(input: TokenStream) -> TokenStream {
    // Delegate to the `derive_table` function in the `resource` module.
    resource::derive_table(input)
}
