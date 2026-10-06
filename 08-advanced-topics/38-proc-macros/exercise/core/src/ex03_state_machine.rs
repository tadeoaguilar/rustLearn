//! Exercise 3: `state_machine! { .. }` -- a DSL with its own grammar.
//!
//! ```text
//! state_machine! {
//!     machine TrafficLight {
//!         initial Red;
//!         Red -> Green on Go;
//!         Green -> Yellow on Slow;
//!         Yellow -> Red on Stop;
//!     }
//! }
//! ```
//!
//! generates `TrafficLightState` and `TrafficLightEvent` enums (variants in
//! order of first appearance, the initial state first), a
//! `TrafficLightError { state, event }` for invalid transitions, and a
//! `TrafficLight` struct with `new`, `state`, `can_fire`, `fire` and a
//! `TRANSITIONS` table. The grammar is parsed with a hand-written
//! `syn::parse::Parse` implementation; mistakes become compile errors at the
//! right token: a missing `initial`, no transitions, or two transitions for
//! the same state and event.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Token, braced};

mod kw {
    syn::custom_keyword!(machine);
    syn::custom_keyword!(initial);
    syn::custom_keyword!(on);
}

/// `from -> to on event;`
#[derive(Debug, Clone)]
pub struct Transition {
    pub from: Ident,
    pub to: Ident,
    pub event: Ident,
}

#[derive(Debug, Clone)]
pub struct Machine {
    pub name: Ident,
    pub initial: Ident,
    pub transitions: Vec<Transition>,
}

impl Parse for Machine {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        todo!("Exercise 3")
    }
}

/// Unique identifiers, in order of first appearance.
fn unique<'a>(idents: impl IntoIterator<Item = &'a Ident>) -> Vec<Ident> {
    todo!("Exercise 3")
}

pub fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    todo!("Exercise 3")
}
