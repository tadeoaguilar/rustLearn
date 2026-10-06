// 38-proc-macros -- YOUR WORKSPACE.
//
//     cargo run -p m38-proc-macros -- <1-4|bonus>
//
// Prints what your expansion function in core/src/ generates for a sample
// input (or the compile error it produces). This binary doesn't *use* the
// macros -- while they're todo!() that wouldn't compile; the tests do that,
// one exercise at a time (see tests/Cargo.toml).

use m38_proc_macros_core as core;
use quote::quote;
use syn::parse_quote;

fn show(result: syn::Result<proc_macro2::TokenStream>) {
    match result {
        Ok(tokens) => println!("{tokens}\n"),
        Err(e) => println!("compile error: {e}\n"),
    }
}

fn main() {
    let krate: syn::Path = parse_quote!(::m38_proc_macros);
    match std::env::args().nth(1).as_deref() {
        Some("1") => show(core::ex01_to_json::derive(
            quote! { struct User { id: u64, #[json(rename = "displayName")] name: String, #[json(skip)] secret: String } },
            &krate,
        )),
        Some("2") => show(core::ex02_table::derive(
            quote! { #[table(name = "people")] struct Person { #[column(primary_key)] id: i64, name: String, email: Option<String> } },
            &krate,
        )),
        Some("3") => show(core::ex03_state_machine::expand(quote! {
            machine Door { initial Closed; Closed -> Open on Push; Open -> Closed on Pull; }
        })),
        Some("4") => show(core::ex04_memoize::expand(
            quote! {},
            quote! { fn fib(n: u64) -> u64 { if n < 2 { n } else { fib(n - 1) + fib(n - 2) } } },
        )),
        Some("bonus") => show(core::bonus_enum_iter::derive(
            quote! { enum Planet { Mercury, Venus, Earth } },
        )),
        _ => println!(
            "38-proc-macros -- your workspace\n\n  cargo run -p m38-proc-macros -- <1-4|bonus>   print your expansion for a sample input"
        ),
    }
}
