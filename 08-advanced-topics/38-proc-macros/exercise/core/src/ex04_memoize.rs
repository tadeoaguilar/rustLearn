//! Exercise 4: `#[memoize]` -- an attribute macro that rewrites a function.
//!
//! ```text
//! #[memoize]
//! fn fib(n: u64) -> u64 { if n < 2 { n } else { fib(n - 1) + fib(n - 2) } }
//! ```
//!
//! becomes a function with a per-thread cache from arguments to results:
//! check the cache, otherwise run the original body and store the result.
//! Recursive calls go through the cache too, so `fib(90)` is instant. A
//! companion `fib_cache_len()` reports how many results are cached.
//!
//! Requirements (compile errors otherwise, pointing at the problem): a free
//! function (no `self`), not `async`, no generic parameters, a return type,
//! and plain identifier arguments. Arguments must be `Clone + Hash + Eq`
//! and the result `Clone` -- the compiler checks those itself.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, ItemFn, Pat, ReturnType};

pub fn expand(args: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    todo!("Exercise 4")
}
