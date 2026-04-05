use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
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
    let mut group = c.benchmark_group("macros");

    for (name, source) in [
        ("unit_struct", "struct Inventory;"),
        ("tuple_struct", "struct Inventory<T, U>(T, U);"),
        (
            "named_struct",
            "struct Inventory<T> where T: Clone { items: Vec<T>, slots: usize }",
        ),
        (
            "lifetime_generic",
            "struct Borrowed<'a, T>(&'a T) where T: Send + Sync;",
        ),
    ] {
        group.bench_with_input(
            BenchmarkId::new("derive_table_like", name),
            &source,
            |b, source| {
                b.iter(|| {
                    let input = syn::parse_str::<DeriveInput>(black_box(source))
                        .expect("Input should parse");

                    expand_like_derive_table(input)
                })
            },
        );
    }

    group.bench_function("derive_table_like/parse_quote_input", |b| {
        b.iter(|| {
            let input: DeriveInput = parse_quote! {
                struct Inventory<T, U>
                where
                    T: Clone,
                    U: Send
                {
                    primary: T,
                    secondary: U,
                }
            };

            expand_like_derive_table(black_box(input))
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_derive_table);
criterion_main!(benches);
