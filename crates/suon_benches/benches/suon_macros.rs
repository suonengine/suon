use criterion::{Criterion, criterion_group, criterion_main};
use quote::quote;
use std::hint::black_box;
use syn::{DeriveInput, parse_quote};

fn expand_like_derive_table(input: DeriveInput) -> String {
    let mut ast = input;
    ast.generics
        .make_where_clause()
        .predicates
        .push(parse_quote! { Self: Send + Sync + 'static });

    let struct_name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = ast.generics.split_for_impl();

    quote! {
        impl #impl_generics suon_database::Table for #struct_name #type_generics #where_clause {}
    }
    .to_string()
}

fn benchmark_derive_table(c: &mut Criterion) {
    c.bench_function("macros/derive_table_like", |b| {
        b.iter(|| {
            let input = syn::parse_str::<DeriveInput>(black_box("struct Inventory<T, U>(T, U);"))
                .expect("Input should parse");

            expand_like_derive_table(input)
        })
    });
}

criterion_group!(benches, benchmark_derive_table);
criterion_main!(benches);
