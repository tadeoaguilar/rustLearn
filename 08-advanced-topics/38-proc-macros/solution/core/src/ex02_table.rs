//! Exercise 2: `#[derive(Table)]` -- an ORM-style derive.
//!
//! From a struct, generate everything needed to store it in a SQL table:
//! the table name, its columns (name, SQL type, nullability, primary key),
//! the values of an instance in column order, and `from_row` to rebuild one.
//! The runtime trait (`orm::Table`) turns those into `CREATE TABLE`,
//! `INSERT` and `SELECT` statements.
//!
//! - `#[table(name = "people")]` on the struct; default: snake_case + `s`
//! - `#[column(primary_key)]` -- exactly one field must have it
//! - `#[column(name = "full_name")]` -- rename a column
//! - `#[column(skip)]` -- not stored; rebuilt with `Default::default()`
//!
//! A field's SQL type comes from its Rust type through `orm::SqlType`, so a
//! field of an unsupported type is a compile error at that field.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, Path, parse_quote};

use crate::attrs;

pub fn derive(input: TokenStream, krate: &Path) -> syn::Result<TokenStream> {
    let input: DeriveInput = syn::parse2(input)?;
    let name = &input.ident;
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "Table can't be derived for generic structs",
        ));
    }
    let Data::Struct(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            name,
            "Table can only be derived for structs",
        ));
    };
    let Fields::Named(fields) = &data.fields else {
        return Err(syn::Error::new_spanned(
            name,
            "Table needs a struct with named fields",
        ));
    };
    let table_options = attrs::parse(&input.attrs, "table", &["name"])?;
    let table = table_options.value("name").map_or_else(
        || format!("{}s", attrs::snake_case(&name.to_string())),
        str::to_string,
    );

    let sql_type: Path = parse_quote!(#krate::orm::SqlType);
    let mut columns = Vec::new();
    let mut values = Vec::new();
    let mut builders = Vec::new();
    let mut primary_key: Option<syn::Ident> = None;
    let mut index = 0usize;
    for field in &fields.named {
        let ident = field.ident.clone().expect("named field");
        let ty = &field.ty;
        let options = attrs::parse(&field.attrs, "column", &["primary_key", "name", "skip"])?;
        if options.has("skip") {
            if options.has("primary_key") {
                return Err(syn::Error::new_spanned(
                    &ident,
                    "a skipped field can't be the primary key",
                ));
            }
            builders.push(quote!(#ident: ::std::default::Default::default()));
            continue;
        }
        let column = options
            .value("name")
            .map_or_else(|| ident.to_string(), str::to_string);
        let is_pk = options.has("primary_key");
        if is_pk {
            if let Some(first) = &primary_key {
                return Err(syn::Error::new_spanned(
                    &ident,
                    format!("only one primary key is allowed (`{first}` is already one)"),
                ));
            }
            primary_key = Some(ident.clone());
        }
        columns.push(quote! {
            #krate::orm::Column {
                name: #column,
                sql_type: <#ty as #sql_type>::SQL,
                nullable: <#ty as #sql_type>::NULLABLE,
                primary_key: #is_pk,
            }
        });
        values.push(quote!(#sql_type::to_value(&self.#ident)));
        builders.push(quote! {
            #ident: <#ty as #sql_type>::from_value(&row[#index]).map_err(|e| e.in_column(#column))?
        });
        index += 1;
    }
    if primary_key.is_none() {
        return Err(syn::Error::new_spanned(
            name,
            "Table needs one field marked #[column(primary_key)]",
        ));
    }
    let count = index;
    let columns_static = format_ident!("__{}_COLUMNS", name.to_string().to_uppercase());

    Ok(quote! {
        #[allow(non_upper_case_globals)]
        static #columns_static: [#krate::orm::Column; #count] = [#(#columns),*];

        impl #krate::orm::Table for #name {
            const TABLE: &'static str = #table;

            fn columns() -> &'static [#krate::orm::Column] {
                &#columns_static
            }

            fn values(&self) -> ::std::vec::Vec<#krate::orm::Value> {
                vec![#(#values),*]
            }

            fn from_row(row: &[#krate::orm::Value]) -> ::std::result::Result<Self, #krate::orm::OrmError> {
                if row.len() != #count {
                    return Err(#krate::orm::OrmError::WrongColumnCount { expected: #count, got: row.len() });
                }
                Ok(#name { #(#builders),* })
            }
        }
    })
}
