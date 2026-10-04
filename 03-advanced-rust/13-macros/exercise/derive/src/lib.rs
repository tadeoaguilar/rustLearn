//! YOUR WORKSPACE: the procedural macros for 13-macros (Exercises 6, 7, bonus).
//!
//! Each entry point is registered and compiles, but generates nothing yet
//! (or, for #[timed], returns the function unchanged). Fill them in.
//! See exercises.md for the hints, and ../../solution/derive/src/lib.rs when stuck.

#![allow(unused)]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, ItemFn, LitStr, Type, parse_macro_input};

/// Exercise 6: generate
///
/// ```text
/// impl #impl_generics #name #ty_generics #where_clause {
///     pub fn describe() -> String { ... }
///     pub fn field_names() -> &'static [&'static str] { ... }   // named-field structs only
/// }
/// ```
#[proc_macro_derive(Describe)]
pub fn derive_describe(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // TODO Exercise 6
    TokenStream::new()
}

/// Exercise 7: generate `{Name}Builder`, `Name::builder()`, one setter per
/// field, and `build(self) -> Result<Name, String>`.
/// `attributes(builder)` lets users write `#[builder(default)]` on fields.
#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive_builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // TODO Exercise 7
    TokenStream::new()
}

/// Bonus: wrap the function body and print how long each call took.
/// `attr` is empty for `#[timed]` and holds the string for `#[timed("label")]`.
#[proc_macro_attribute]
pub fn timed(attr: TokenStream, item: TokenStream) -> TokenStream {
    // TODO Bonus. For now: return the function unchanged.
    item
}
