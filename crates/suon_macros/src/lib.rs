//! Proc-macro derives for the Suon engine.

mod deref;
mod resource;
mod task;

/// Derives `::suon_resource::Resource` for a struct.
///
/// The struct must satisfy `Send + Sync + 'static`.
#[proc_macro_derive(Resource)]
pub fn derive_resource(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    resource::derive_resource(input)
}

/// Derives `::std::ops::Deref` for a single-field struct.
#[proc_macro_derive(Deref)]
pub fn derive_deref(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    deref::derive_deref(input)
}

/// Derives `::std::ops::DerefMut` for a single-field struct.
#[proc_macro_derive(DerefMut)]
pub fn derive_deref_mut(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    deref::derive_deref_mut(input)
}

/// Derives `::suon_channel::IntoTask` for a struct.
///
/// The struct must also implement [`TaskHandler`] manually.
#[proc_macro_derive(Task)]
pub fn derive_task(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    task::derive_task(input)
}
