//! Attribute macro that generates Diesel schema and DDL from a Rust struct.

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
    Attribute, Error, Fields, Ident, ItemStruct, LitStr, Result, Token, Type, parse_macro_input,
    punctuated::Punctuated,
};

pub fn database_model(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args =
        parse_macro_input!(attr with Punctuated::<syn::Meta, Token![,]>::parse_terminated);
    let item = parse_macro_input!(item as ItemStruct);

    match expand_database_model(attr_args, item) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

fn expand_database_model(
    attr_args: Punctuated<syn::Meta, Token![,]>,
    mut item: ItemStruct,
) -> Result<TokenStream2> {
    let table_name = parse_table_name(&attr_args)?;
    let struct_ident = item.ident.clone();
    let fields = match &mut item.fields {
        Fields::Named(fields) => &mut fields.named,
        _ => {
            return Err(Error::new_spanned(
                &item,
                "database_model only supports structs with named fields",
            ));
        }
    };

    let table_ident = Ident::new(&table_name.value(), table_name.span());
    let columns_ident = format_ident!("{}Columns", struct_ident);
    let create_sql_fn = format_ident!(
        "__suon_create_table_sql_for_{}",
        to_snake_case(&struct_ident.to_string())
    );

    let primary_key_count = fields
        .iter()
        .filter(|field| {
            parse_field_options(&field.attrs)
                .map(|options| options.primary_key)
                .unwrap_or(false)
        })
        .count();
    let composite_primary_key = primary_key_count > 1;

    let mut table_fields = Vec::new();
    let mut sqlite_columns = Vec::new();
    let mut postgres_columns = Vec::new();
    let mut mysql_columns = Vec::new();
    let mut primary_keys = Vec::new();
    let mut primary_key_names = Vec::new();
    let mut column_methods = Vec::new();
    let mut column_struct_fields = Vec::new();
    let mut column_struct_values = Vec::new();

    for field in fields.iter_mut() {
        let field_ident = field
            .ident
            .clone()
            .ok_or_else(|| Error::new_spanned(&*field, "expected named field"))?;
        let column_name =
            find_column_name(&field.attrs)?.unwrap_or_else(|| field_ident.to_string());
        let options = parse_field_options(&field.attrs)?;
        field.attrs.retain(|attr| !attr.path().is_ident("database"));
        let column = infer_column(&field.ty, &column_name, options, composite_primary_key)?;

        if column.primary_key {
            let primary_key = Ident::new(&column_name, Span::call_site());
            primary_keys.push(primary_key.clone());
            primary_key_names.push(column_name.clone());
        }

        let sql_type_tokens = &column.sql_type_tokens;
        let column_ident = Ident::new(&column_name, Span::call_site());
        table_fields.push(quote!(#column_ident -> #sql_type_tokens,));
        column_methods.push(quote! {
            pub fn #field_ident() -> #table_ident::#column_ident {
                #table_ident::#column_ident
            }
        });
        column_struct_fields.push(quote!(pub #field_ident: #table_ident::#column_ident,));
        column_struct_values.push(quote!(#field_ident: #table_ident::#column_ident,));

        let field_attrs = &mut field.attrs;
        field_attrs.push(syn::parse_quote!(#[diesel(column_name = #column_ident)]));

        sqlite_columns.push(LitStr::new(&column.sqlite_sql, Span::call_site()));
        postgres_columns.push(LitStr::new(&column.postgres_sql, Span::call_site()));
        mysql_columns.push(LitStr::new(&column.mysql_sql, Span::call_site()));
    }

    if primary_keys.is_empty() {
        return Err(Error::new_spanned(
            &item,
            "database_model requires at least one #[database(primary_key)] field",
        ));
    }

    if composite_primary_key {
        let constraint = format!("PRIMARY KEY ({})", primary_key_names.join(", "));
        let constraint_lit = LitStr::new(&constraint, Span::call_site());
        sqlite_columns.push(constraint_lit.clone());
        postgres_columns.push(constraint_lit.clone());
        mysql_columns.push(constraint_lit);
    }

    item.attrs.push(
        syn::parse_quote!(#[derive(diesel::Queryable, diesel::Selectable, diesel::Insertable)]),
    );
    item.attrs
        .push(syn::parse_quote!(#[diesel(table_name = #table_ident)]));

    let primary_key_decl = if primary_keys.len() == 1 {
        let key = &primary_keys[0];
        quote!((#key))
    } else {
        quote!((#(#primary_keys),*))
    };

    Ok(quote! {
        diesel::table! {
            #table_ident #primary_key_decl {
                #(#table_fields)*
            }
        }

        #item

        impl diesel::associations::HasTable for #struct_ident {
            type Table = #table_ident::table;

            fn table() -> Self::Table {
                #table_ident::table
            }
        }

        #[derive(Clone, Copy)]
        pub struct #columns_ident {
            #(#column_struct_fields)*
        }

        impl suon_database::prelude::DbRecord for #struct_ident {
            type Query = diesel::helper_types::Select<
                #table_ident::table,
                diesel::helper_types::AsSelect<
                    Self,
                    <suon_database::prelude::DbDriver as diesel::Connection>::Backend,
                >,
            >;
            type Columns = #columns_ident;

            fn query() -> Self::Query {
                use diesel::{QueryDsl, SelectableHelper};

                #table_ident::table.select(Self::as_select())
            }

            fn columns() -> Self::Columns {
                #columns_ident {
                    #(#column_struct_values)*
                }
            }
        }

        impl #struct_ident {
            #(#column_methods)*

            /// Creates a typed select query for this model.
            pub fn query(
                driver: &mut suon_database::prelude::DbDriver,
            ) -> suon_database::prelude::PendingStatement<
                '_,
                <Self as suon_database::prelude::DbRecord>::Query,
                Self,
            > {
                driver.query::<Self>()
            }

            /// Creates the target table if it does not already exist.
            pub fn ensure_table(
                driver: &mut suon_database::prelude::DbDriver,
                backend: suon_database::prelude::DbBackend,
            ) -> anyhow::Result<usize> {
                use diesel::RunQueryDsl;

                diesel::sql_query(Self::create_table_sql(backend))
                    .execute(driver)
                    .map_err(anyhow::Error::from)
            }

            /// Returns backend-specific `CREATE TABLE IF NOT EXISTS` DDL.
            pub fn create_table_sql(backend: suon_database::prelude::DbBackend) -> ::std::string::String {
                #create_sql_fn(backend)
            }
        }

        fn #create_sql_fn(backend: suon_database::prelude::DbBackend) -> ::std::string::String {
            let columns = match backend {
                suon_database::prelude::DbBackend::Sqlite => [#(#sqlite_columns),*].join(",\n    "),
                suon_database::prelude::DbBackend::Postgres => [#(#postgres_columns),*].join(",\n    "),
                suon_database::prelude::DbBackend::MySql | suon_database::prelude::DbBackend::MariaDb => [#(#mysql_columns),*].join(",\n    "),
            };

            format!(
                "CREATE TABLE IF NOT EXISTS {} (\n    {}\n)",
                #table_name,
                columns,
            )
        }
    })
}

fn parse_table_name(args: &Punctuated<syn::Meta, Token![,]>) -> Result<LitStr> {
    for arg in args {
        match arg {
            syn::Meta::NameValue(name_value) if name_value.path.is_ident("table") => {
                if let syn::Expr::Lit(expr_lit) = &name_value.value
                    && let syn::Lit::Str(value) = &expr_lit.lit
                {
                    return Ok(LitStr::new(&value.value(), value.span()));
                }

                return Err(Error::new_spanned(
                    name_value,
                    "expected table = \"table_name\"",
                ));
            }
            syn::Meta::Path(path) if path.is_ident("table") => {
                return Err(Error::new_spanned(path, "expected table = \"table_name\""));
            }
            _ => {}
        }
    }

    Err(Error::new(
        Span::call_site(),
        "database_model requires table = \"table_name\"",
    ))
}

fn find_column_name(attrs: &[Attribute]) -> Result<Option<String>> {
    for attr in attrs {
        if !attr.path().is_ident("database") {
            continue;
        }

        let metas = attr.parse_args_with(Punctuated::<syn::Meta, Token![,]>::parse_terminated)?;
        for meta in metas {
            if let syn::Meta::NameValue(name_value) = meta
                && name_value.path.is_ident("column_name")
            {
                if let syn::Expr::Lit(ref expr_lit) = name_value.value
                    && let syn::Lit::Str(value) = &expr_lit.lit
                {
                    return Ok(Some(value.value()));
                }

                return Err(Error::new_spanned(
                    name_value,
                    "expected column_name = \"column_name\"",
                ));
            }
        }
    }

    Ok(None)
}

#[derive(Default)]
struct FieldOptions {
    primary_key: bool,
    auto: bool,
}

fn parse_field_options(attrs: &[Attribute]) -> Result<FieldOptions> {
    let mut options = FieldOptions::default();

    for attr in attrs {
        if !attr.path().is_ident("database") {
            continue;
        }

        let metas = attr.parse_args_with(Punctuated::<syn::Meta, Token![,]>::parse_terminated)?;
        for meta in metas {
            match meta {
                syn::Meta::Path(path) if path.is_ident("primary_key") => options.primary_key = true,
                syn::Meta::Path(path) if path.is_ident("auto") => options.auto = true,
                syn::Meta::NameValue(_) => {}
                other => {
                    return Err(Error::new_spanned(
                        other,
                        "unsupported #[database(...)] field option",
                    ));
                }
            }
        }
    }

    Ok(options)
}

struct InferredColumn {
    primary_key: bool,
    sql_type_tokens: TokenStream2,
    sqlite_sql: String,
    postgres_sql: String,
    mysql_sql: String,
}

fn infer_column(
    ty: &Type,
    column_name: &str,
    options: FieldOptions,
    composite_primary_key: bool,
) -> Result<InferredColumn> {
    let (base_ty, nullable) = unwrap_option(ty);
    let base_ident = extract_type_ident(base_ty)?;
    let base_name = base_ident.to_string();

    let (sql_type_tokens, sqlite_type, postgres_type, mysql_type) = match base_name.as_str() {
        "i16" => (
            quote!(SmallInt),
            "SMALLINT".to_string(),
            "SMALLINT".to_string(),
            "SMALLINT".to_string(),
        ),
        "i32" => (
            quote!(Integer),
            "INTEGER".to_string(),
            "INTEGER".to_string(),
            "INTEGER".to_string(),
        ),
        "i64" => (
            quote!(BigInt),
            "BIGINT".to_string(),
            "BIGINT".to_string(),
            "BIGINT".to_string(),
        ),
        "bool" => (
            quote!(Bool),
            "INTEGER".to_string(),
            "BOOLEAN".to_string(),
            "BOOLEAN".to_string(),
        ),
        "String" => (
            quote!(Text),
            "TEXT".to_string(),
            "TEXT".to_string(),
            "TEXT".to_string(),
        ),
        other => {
            return Err(Error::new_spanned(
                ty,
                format!("database_model cannot infer a Diesel SQL type for `{other}`"),
            ));
        }
    };

    let sql_type_tokens = if nullable {
        quote!(Nullable<#sql_type_tokens>)
    } else {
        sql_type_tokens
    };

    let inline_primary_key = options.primary_key && !composite_primary_key;
    let sqlite_sql = build_column_sql(
        column_name,
        &sqlite_type,
        nullable,
        inline_primary_key,
        options.auto,
        base_name.as_str(),
        DatabaseFlavor::Sqlite,
    );
    let postgres_sql = build_column_sql(
        column_name,
        &postgres_type,
        nullable,
        inline_primary_key,
        options.auto,
        base_name.as_str(),
        DatabaseFlavor::Postgres,
    );
    let mysql_sql = build_column_sql(
        column_name,
        &mysql_type,
        nullable,
        inline_primary_key,
        options.auto,
        base_name.as_str(),
        DatabaseFlavor::MySql,
    );

    Ok(InferredColumn {
        primary_key: options.primary_key,
        sql_type_tokens,
        sqlite_sql,
        postgres_sql,
        mysql_sql,
    })
}

#[derive(Clone, Copy)]
enum DatabaseFlavor {
    Sqlite,
    Postgres,
    MySql,
}

fn build_column_sql(
    column_name: &str,
    base_sql_type: &str,
    nullable: bool,
    primary_key: bool,
    auto: bool,
    rust_type: &str,
    flavor: DatabaseFlavor,
) -> String {
    let null_sql = if nullable { "NULL" } else { "NOT NULL" };

    let definition = if primary_key && auto {
        match flavor {
            DatabaseFlavor::Sqlite => "INTEGER PRIMARY KEY AUTOINCREMENT".to_string(),
            DatabaseFlavor::Postgres => match rust_type {
                "i32" => "SERIAL PRIMARY KEY".to_string(),
                _ => "BIGSERIAL PRIMARY KEY".to_string(),
            },
            DatabaseFlavor::MySql => format!("{base_sql_type} AUTO_INCREMENT PRIMARY KEY"),
        }
    } else if primary_key {
        format!("{base_sql_type} PRIMARY KEY {null_sql}")
    } else {
        format!("{base_sql_type} {null_sql}")
    };

    format!("{column_name} {definition}")
}

fn unwrap_option(ty: &Type) -> (&Type, bool) {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && segment.ident == "Option"
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
    {
        return (inner, true);
    }

    (ty, false)
}

fn extract_type_ident(ty: &Type) -> Result<&Ident> {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return Ok(&segment.ident);
    }

    Err(Error::new_spanned(
        ty,
        "database_model only supports primitive field types and Option<T>",
    ))
}

fn to_snake_case(input: &str) -> String {
    let mut out = String::with_capacity(input.len());

    for (index, ch) in input.chars().enumerate() {
        if ch.is_uppercase() {
            if index != 0 {
                out.push('_');
            }

            for lower in ch.to_lowercase() {
                out.push(lower);
            }
        } else {
            out.push(ch);
        }
    }

    out
}
