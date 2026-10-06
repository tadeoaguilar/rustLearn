//! Exercise 1: `#[derive(ToJson)]`.
//!
//! - structs with named fields -> a JSON object, fields in declaration order
//! - `#[json(rename = "name")]` changes a field's key; `#[json(skip)]` leaves it out
//! - enums: a unit variant -> its name as a string (or its rename); a variant
//!   with fields -> `{"Variant": <fields as an object or array>}`
//! - generic parameters get a `ToJson` bound
//! - tuple structs -> arrays; unit structs -> `null`
//!
//! The generated impl calls `ToJson::to_json` on each field, so any field
//! type that implements the trait works -- including other derived types.

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Fields, Path, parse_quote};

use crate::attrs;

pub fn derive(input: TokenStream, krate: &Path) -> syn::Result<TokenStream> {
    todo!("Exercise 1")
}

/// `"text"` as a JSON string literal (for keys known at expansion time).
pub fn json_string(text: &str) -> String {
    todo!("Exercise 1")
}

/// How generated code reaches the fields.
#[derive(Clone, Copy)]
enum Access {
    /// A struct: `&self.name`, `&self.0`.
    SelfFields,
    /// An enum variant matched on `&self`: the bindings `name`, `f0`, ...
    Bindings,
}

/// An expression building the JSON for `fields`.
fn fields_to_json(fields: &Fields, access: Access, trait_path: &Path) -> syn::Result<TokenStream> {
    todo!("Exercise 1")
}
