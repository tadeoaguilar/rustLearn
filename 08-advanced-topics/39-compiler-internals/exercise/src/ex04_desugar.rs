//! Exercise 4: desugaring, as the compiler does it during lowering to HIR.
//!
//! Much of Rust's surface syntax is sugar that disappears before type
//! checking. Rewrite, with `syn::visit_mut`:
//!
//! ```text
//! 'label: for PAT in EXPR { BODY }
//! ```
//! into (the shape rustc itself uses -- the `match` keeps temporaries in
//! EXPR alive for the whole loop)
//! ```text
//! match ::core::iter::IntoIterator::into_iter(EXPR) {
//!     mut __iter0 => 'label: loop {
//!         match ::core::iter::Iterator::next(&mut __iter0) {
//!             ::core::option::Option::Some(PAT) => BODY,
//!             ::core::option::Option::None => break,
//!         }
//!     },
//! }
//! ```
//! and `EXPR?` (for `Result`) into
//! ```text
//! match EXPR {
//!     ::core::result::Result::Ok(__val) => __val,
//!     ::core::result::Result::Err(__err) => return ::core::result::Result::Err(::core::convert::From::from(__err)),
//! }
//! ```
//!
//! Inner loops are rewritten first; each loop gets its own `__iterN` (a
//! counter), because these names aren't hygienic. (The real `?` goes
//! through the unstable `Try` trait and also works on `Option`; this
//! version handles `Result` only.)

use quote::{ToTokens, format_ident};
use syn::visit::Visit;
use syn::visit_mut::VisitMut;
use syn::{Expr, parse_quote};

#[derive(Default)]
pub struct Desugarer {
    loops: usize,
}

impl VisitMut for Desugarer {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        todo!("Exercise 4")
    }
}

/// Desugar every `for` and `?` in a source file; the result as tokens.
pub fn desugar(source: &str) -> syn::Result<String> {
    let mut file = syn::parse_file(source)?;
    Desugarer::default().visit_file_mut(&mut file);
    Ok(file.into_token_stream().to_string())
}

/// How many `for` loops and `?` operators a source file has (to check that
/// none survive).
pub fn count_sugar(source: &str) -> syn::Result<(usize, usize)> {
    #[derive(Default)]
    struct Counter(usize, usize);
    impl<'ast> Visit<'ast> for Counter {
        fn visit_expr_for_loop(&mut self, e: &'ast syn::ExprForLoop) {
            self.0 += 1;
            syn::visit::visit_expr_for_loop(self, e);
        }
        fn visit_expr_try(&mut self, e: &'ast syn::ExprTry) {
            self.1 += 1;
            syn::visit::visit_expr_try(self, e);
        }
    }
    let file = syn::parse_file(source)?;
    let mut counter = Counter::default();
    counter.visit_file(&file);
    Ok((counter.0, counter.1))
}
