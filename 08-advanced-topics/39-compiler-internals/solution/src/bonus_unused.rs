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
    let file = syn::parse_file(source)?;
    let mut out = Vec::new();
    let mut functions = Functions(Vec::new());
    functions.visit_file(&file);
    for (name, inputs, block) in functions.0 {
        let mut scan = Scan::default();
        for input in &inputs {
            if let syn::FnArg::Typed(typed) = input {
                scan.bind_pattern(&typed.pat);
            }
        }
        scan.visit_block(&block);
        for bound in scan.bound {
            if !bound.starts_with('_') && !scan.used.contains(&bound) {
                out.push((name.clone(), bound));
            }
        }
    }
    Ok(out)
}

/// Every function (free or in an impl), with its parameters and body.
struct Functions(Vec<(String, Vec<syn::FnArg>, syn::Block)>);

impl<'ast> Visit<'ast> for Functions {
    fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
        self.0.push((
            f.sig.ident.to_string(),
            f.sig.inputs.iter().cloned().collect(),
            (*f.block).clone(),
        ));
        syn::visit::visit_item_fn(self, f);
    }
    fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
        self.0.push((
            f.sig.ident.to_string(),
            f.sig.inputs.iter().cloned().collect(),
            f.block.clone(),
        ));
        syn::visit::visit_impl_item_fn(self, f);
    }
}

#[derive(Default)]
struct Scan {
    bound: Vec<String>,
    used: Vec<String>,
}

impl Scan {
    fn bind_pattern(&mut self, pat: &syn::Pat) {
        struct Binder<'a>(&'a mut Vec<String>);
        impl<'ast> Visit<'ast> for Binder<'_> {
            fn visit_pat_ident(&mut self, p: &'ast syn::PatIdent) {
                let name = p.ident.to_string();
                if !self.0.contains(&name) {
                    self.0.push(name);
                }
                syn::visit::visit_pat_ident(self, p);
            }
        }
        Binder(&mut self.bound).visit_pat(pat);
    }

    fn use_tokens(&mut self, tokens: TokenStream) {
        for tree in tokens {
            match tree {
                TokenTree::Ident(ident) => self.used.push(ident.to_string()),
                TokenTree::Group(group) => self.use_tokens(group.stream()),
                TokenTree::Literal(literal) => {
                    // "{name}" / "{name:?}" in a format string
                    let text = literal.to_string();
                    for piece in text.split('{').skip(1) {
                        let name: String = piece
                            .chars()
                            .take_while(|c| c.is_alphanumeric() || *c == '_')
                            .collect();
                        if !name.is_empty()
                            && !name.chars().next().is_some_and(|c| c.is_ascii_digit())
                        {
                            self.used.push(name);
                        }
                    }
                }
                TokenTree::Punct(_) => {}
            }
        }
    }
}

impl<'ast> Visit<'ast> for Scan {
    fn visit_local(&mut self, local: &'ast syn::Local) {
        self.bind_pattern(&local.pat);
        syn::visit::visit_local(self, local);
    }

    fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
        if let Some(ident) = path.path.get_ident() {
            self.used.push(ident.to_string());
        }
        syn::visit::visit_expr_path(self, path);
    }

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        self.use_tokens(mac.tokens.clone());
    }

    fn visit_expr_closure(&mut self, closure: &'ast syn::ExprClosure) {
        for input in &closure.inputs {
            self.bind_pattern(input);
        }
        self.visit_expr(&closure.body);
    }

    fn visit_expr_for_loop(&mut self, for_loop: &'ast syn::ExprForLoop) {
        self.bind_pattern(&for_loop.pat);
        self.visit_expr(&for_loop.expr);
        self.visit_block(&for_loop.body);
    }

    // a nested fn is its own function, scanned separately
    fn visit_item_fn(&mut self, _: &'ast syn::ItemFn) {}
}
