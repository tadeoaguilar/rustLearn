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
    todo!("Exercise 2")
}
