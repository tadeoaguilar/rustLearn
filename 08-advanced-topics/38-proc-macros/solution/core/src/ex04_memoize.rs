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
    if !args.is_empty() {
        return Err(syn::Error::new_spanned(
            args,
            "#[memoize] takes no arguments",
        ));
    }
    let function: ItemFn = syn::parse2(item)?;
    let sig = &function.sig;
    if let Some(asyncness) = &sig.asyncness {
        return Err(syn::Error::new_spanned(
            asyncness,
            "#[memoize] doesn't support async functions",
        ));
    }
    if !sig.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &sig.generics,
            "#[memoize] doesn't support generic functions",
        ));
    }
    let ReturnType::Type(_, output) = &sig.output else {
        return Err(syn::Error::new_spanned(
            &sig.ident,
            "#[memoize] needs a function that returns a value",
        ));
    };
    let mut names = Vec::new();
    let mut types = Vec::new();
    for arg in &sig.inputs {
        match arg {
            FnArg::Receiver(receiver) => {
                return Err(syn::Error::new_spanned(
                    receiver,
                    "#[memoize] works on free functions, not methods",
                ));
            }
            FnArg::Typed(typed) => match &*typed.pat {
                Pat::Ident(pat) if pat.by_ref.is_none() && pat.subpat.is_none() => {
                    names.push(pat.ident.clone());
                    types.push((*typed.ty).clone());
                }
                other => {
                    return Err(syn::Error::new_spanned(
                        other,
                        "#[memoize] needs plain identifier arguments",
                    ));
                }
            },
        }
    }

    let vis = &function.vis;
    let attrs = &function.attrs;
    let block = &function.block;
    let len_fn = format_ident!("{}_cache_len", sig.ident);
    let cache = format_ident!("__{}_MEMO", sig.ident.to_string().to_uppercase());

    Ok(quote! {
        ::std::thread_local! {
            #[allow(non_upper_case_globals)]
            static #cache: ::std::cell::RefCell<::std::collections::HashMap<(#(#types,)*), #output>> =
                ::std::cell::RefCell::new(::std::collections::HashMap::new());
        }

        #(#attrs)*
        #vis #sig {
            let key = (#(::std::clone::Clone::clone(&#names),)*);
            if let Some(hit) = #cache.with(|c| c.borrow().get(&key).cloned()) {
                return hit;
            }
            // The original body, as a closure so its `return`s return from it.
            // No borrow of the cache is held here: recursive calls are fine.
            #[allow(clippy::redundant_closure_call)]
            let result: #output = (move || #block)();
            #cache.with(|c| c.borrow_mut().insert(key, ::std::clone::Clone::clone(&result)));
            result
        }

        /// How many results this thread's cache holds.
        #vis fn #len_fn() -> usize {
            #cache.with(|c| c.borrow().len())
        }
    })
}
