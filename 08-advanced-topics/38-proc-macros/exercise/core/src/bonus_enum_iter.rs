//! Bonus: `#[derive(EnumIter)]` for enums of unit variants.
//!
//! Generates `ALL` (every variant, in order), `name()` (the variant's name),
//! `from_name` (the inverse) and `COUNT`. A variant with fields is a compile
//! error at that variant.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

pub fn derive(input: TokenStream) -> syn::Result<TokenStream> {
    todo!("Bonus")
}
