//! Bonus: an unused-variable check -- rustc's `unused_variables` lint, on syntax.
//!
//! For each function: the identifiers bound by its parameters and `let`
//! patterns that are never used afterwards. Names starting with `_` are
//! exempt. A use is a single-identifier path expression (`x`, `x.len()`),
//! an identifier in a macro's tokens, or an inline format argument in a
//! macro's string literal (`"{x}"`, `"{x:?}"`). Shadowing is ignored (each
//! name counts once per function) -- rustc tracks it properly, with scopes.
//! Result: `(function, variable)` pairs in order of declaration.

use proc_macro2::{TokenStream, TokenTree};
use syn::visit::Visit;

pub fn unused_variables(source: &str) -> syn::Result<Vec<(String, String)>> {
    todo!("Bonus")
}

/// Every function (free or in an impl), with its parameters and body.
struct Functions(Vec<(String, Vec<syn::FnArg>, syn::Block)>);

impl<'ast> Visit<'ast> for Functions {
    fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
        todo!("Bonus")
    }
    fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
        todo!("Bonus")
    }
}

#[derive(Default)]
struct Scan {
    bound: Vec<String>,
    used: Vec<String>,
}

impl Scan {
    fn bind_pattern(&mut self, pat: &syn::Pat) {
        todo!("Bonus")
    }

    fn use_tokens(&mut self, tokens: TokenStream) {
        todo!("Bonus")
    }
}

impl<'ast> Visit<'ast> for Scan {
    fn visit_local(&mut self, local: &'ast syn::Local) {
        todo!("Bonus")
    }

    fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
        todo!("Bonus")
    }

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        todo!("Bonus")
    }

    fn visit_expr_closure(&mut self, closure: &'ast syn::ExprClosure) {
        todo!("Bonus")
    }

    fn visit_expr_for_loop(&mut self, for_loop: &'ast syn::ExprForLoop) {
        todo!("Bonus")
    }

    // a nested fn is its own function, scanned separately
    fn visit_item_fn(&mut self, _: &'ast syn::ItemFn) {
        todo!("Bonus")
    }
}
