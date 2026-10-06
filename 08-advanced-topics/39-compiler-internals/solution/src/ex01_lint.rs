//! Exercise 1: a lint tool -- the shape of a clippy lint, on `syn`'s AST.
//!
//! Clippy lints are visitors over the compiler's HIR with type information.
//! This one walks `syn`'s syntax tree instead (no types, but it runs on any
//! file without the compiler internals). It reports, with 1-based
//! line:column:
//!
//! | Lint | Fires on |
//! |---|---|
//! | `unwrap_used` | `.unwrap()` (no arguments) |
//! | `expect_used` | `.expect(..)` (one argument) |
//! | `debug_macro` | `todo!`, `unimplemented!`, `dbg!` |
//! | `as_cast` | any `expr as Type` |
//! | `long_function` | a `fn` spanning more than `max_fn_lines` lines (reported at its name) |
//! | `missing_docs` | a `pub` fn, struct, enum or trait without a doc comment (at its name) |
//!
//! Code in a `#[cfg(test)]` module is skipped entirely (tests may unwrap).
//! Macro arguments aren't parsed, so `println!("{}", x.unwrap())` isn't seen
//! -- a real limitation clippy doesn't have, because it lints *after*
//! expansion.

use std::fmt;

use proc_macro2::Span;
use syn::visit::Visit;
use syn::{Attribute, ExprCast, ExprMethodCall, ImplItemFn, ItemFn, ItemMod, Macro, Visibility};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Lint {
    UnwrapUsed,
    ExpectUsed,
    DebugMacro,
    AsCast,
    LongFunction,
    MissingDocs,
}

impl Lint {
    pub fn name(self) -> &'static str {
        match self {
            Lint::UnwrapUsed => "unwrap_used",
            Lint::ExpectUsed => "expect_used",
            Lint::DebugMacro => "debug_macro",
            Lint::AsCast => "as_cast",
            Lint::LongFunction => "long_function",
            Lint::MissingDocs => "missing_docs",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub line: usize,
    pub column: usize,
    pub lint: Lint,
    pub message: String,
}

impl fmt::Display for Finding {
    /// `12:9: unwrap_used: ...`
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}: {}: {}",
            self.line,
            self.column,
            self.lint.name(),
            self.message
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub max_fn_lines: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config { max_fn_lines: 40 }
    }
}

/// Every finding in `source`, sorted by position.
pub fn lint_source(source: &str, config: &Config) -> syn::Result<Vec<Finding>> {
    let file = syn::parse_file(source)?;
    let mut linter = Linter {
        config: *config,
        findings: Vec::new(),
    };
    linter.visit_file(&file);
    linter.findings.sort_by_key(|f| (f.line, f.column, f.lint));
    Ok(linter.findings)
}

struct Linter {
    config: Config,
    findings: Vec<Finding>,
}

fn is_cfg_test(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path().is_ident("cfg") && a.parse_args::<syn::Ident>().is_ok_and(|i| i == "test")
    })
}

fn has_docs(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|a| a.path().is_ident("doc"))
}

impl Linter {
    fn report(&mut self, span: Span, lint: Lint, message: impl Into<String>) {
        let start = span.start();
        self.findings.push(Finding {
            line: start.line,
            column: start.column + 1,
            lint,
            message: message.into(),
        });
    }

    fn check_fn(
        &mut self,
        attrs: &[Attribute],
        vis: &Visibility,
        sig: &syn::Signature,
        body: &syn::Block,
    ) {
        let name = &sig.ident;
        // the whole function: from its first token (the signature) to the closing brace
        let first = sig.fn_token.span.start().line;
        let last = body.brace_token.span.close().end().line;
        let lines = last - first + 1;
        if lines > self.config.max_fn_lines {
            self.report(
                name.span(),
                Lint::LongFunction,
                format!(
                    "`{name}` is {lines} lines long (max {})",
                    self.config.max_fn_lines
                ),
            );
        }
        if matches!(vis, Visibility::Public(_)) && !has_docs(attrs) {
            self.report(
                name.span(),
                Lint::MissingDocs,
                format!("public function `{name}` has no documentation"),
            );
        }
    }

    fn check_type_docs(
        &mut self,
        attrs: &[Attribute],
        vis: &Visibility,
        ident: &syn::Ident,
        kind: &str,
    ) {
        if matches!(vis, Visibility::Public(_)) && !has_docs(attrs) {
            self.report(
                ident.span(),
                Lint::MissingDocs,
                format!("public {kind} `{ident}` has no documentation"),
            );
        }
    }
}

impl<'ast> Visit<'ast> for Linter {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        if !is_cfg_test(&module.attrs) {
            syn::visit::visit_item_mod(self, module);
        }
    }

    fn visit_item_fn(&mut self, item: &'ast ItemFn) {
        self.check_fn(&item.attrs, &item.vis, &item.sig, &item.block);
        syn::visit::visit_item_fn(self, item);
    }

    fn visit_impl_item_fn(&mut self, item: &'ast ImplItemFn) {
        self.check_fn(&item.attrs, &item.vis, &item.sig, &item.block);
        syn::visit::visit_impl_item_fn(self, item);
    }

    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        self.check_type_docs(&item.attrs, &item.vis, &item.ident, "struct");
        syn::visit::visit_item_struct(self, item);
    }

    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        self.check_type_docs(&item.attrs, &item.vis, &item.ident, "enum");
        syn::visit::visit_item_enum(self, item);
    }

    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        self.check_type_docs(&item.attrs, &item.vis, &item.ident, "trait");
        syn::visit::visit_item_trait(self, item);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        let method = &call.method;
        if method == "unwrap" && call.args.is_empty() {
            self.report(
                method.span(),
                Lint::UnwrapUsed,
                "called `unwrap()`: handle the error or use `?`",
            );
        } else if method == "expect" && call.args.len() == 1 {
            self.report(
                method.span(),
                Lint::ExpectUsed,
                "called `expect(..)`: handle the error or use `?`",
            );
        }
        syn::visit::visit_expr_method_call(self, call);
    }

    fn visit_macro(&mut self, mac: &'ast Macro) {
        if let Some(last) = mac.path.segments.last() {
            let name = last.ident.to_string();
            if matches!(name.as_str(), "todo" | "unimplemented" | "dbg") {
                self.report(
                    last.ident.span(),
                    Lint::DebugMacro,
                    format!("`{name}!` left in the code"),
                );
            }
        }
        syn::visit::visit_macro(self, mac);
    }

    fn visit_expr_cast(&mut self, cast: &'ast ExprCast) {
        self.report(
            cast.as_token.span,
            Lint::AsCast,
            "`as` cast: may truncate or wrap; consider `From`/`TryFrom`",
        );
        syn::visit::visit_expr_cast(self, cast);
    }
}
