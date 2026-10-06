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
    todo!("Exercise 1")
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
        todo!("Exercise 1")
    }

    fn check_type_docs(
        &mut self,
        attrs: &[Attribute],
        vis: &Visibility,
        ident: &syn::Ident,
        kind: &str,
    ) {
        todo!("Exercise 1")
    }
}

impl<'ast> Visit<'ast> for Linter {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        todo!("Exercise 1")
    }

    fn visit_item_fn(&mut self, item: &'ast ItemFn) {
        todo!("Exercise 1")
    }

    fn visit_impl_item_fn(&mut self, item: &'ast ImplItemFn) {
        todo!("Exercise 1")
    }

    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        todo!("Exercise 1")
    }

    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        todo!("Exercise 1")
    }

    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        todo!("Exercise 1")
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        todo!("Exercise 1")
    }

    fn visit_macro(&mut self, mac: &'ast Macro) {
        todo!("Exercise 1")
    }

    fn visit_expr_cast(&mut self, cast: &'ast ExprCast) {
        todo!("Exercise 1")
    }
}
