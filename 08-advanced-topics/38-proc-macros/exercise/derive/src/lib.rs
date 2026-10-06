//! The procedural macros of module 38. Each is a thin wrapper: parse
//! nothing, call the expansion in `m38_proc_macros_core`, and turn
//! an error into `compile_error!` at its span. The logic -- and the tests --
//! live in the core crate.

use proc_macro::TokenStream;
use syn::parse_quote;

use m38_proc_macros_core as core;

/// The path generated code uses to reach the runtime crate.
fn krate() -> syn::Path {
    parse_quote!(::m38_proc_macros)
}

fn finish(result: syn::Result<proc_macro2::TokenStream>) -> TokenStream {
    result.unwrap_or_else(|e| e.to_compile_error()).into()
}

#[proc_macro_derive(ToJson, attributes(json))]
pub fn derive_to_json(input: TokenStream) -> TokenStream {
    finish(core::ex01_to_json::derive(input.into(), &krate()))
}

#[proc_macro_derive(Table, attributes(table, column))]
pub fn derive_table(input: TokenStream) -> TokenStream {
    finish(core::ex02_table::derive(input.into(), &krate()))
}

#[proc_macro]
pub fn state_machine(input: TokenStream) -> TokenStream {
    finish(core::ex03_state_machine::expand(input.into()))
}

#[proc_macro_attribute]
pub fn memoize(args: TokenStream, item: TokenStream) -> TokenStream {
    finish(core::ex04_memoize::expand(args.into(), item.into()))
}

#[proc_macro_derive(EnumIter)]
pub fn derive_enum_iter(input: TokenStream) -> TokenStream {
    finish(core::bonus_enum_iter::derive(input.into()))
}
