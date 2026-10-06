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
        input.parse::<kw::machine>()?;
        let name: Ident = input.parse()?;
        let content;
        braced!(content in input);
        if !content.peek(kw::initial) {
            return Err(content.error("expected `initial <State>;` first"));
        }
        content.parse::<kw::initial>()?;
        let initial: Ident = content.parse()?;
        content.parse::<Token![;]>()?;
        let mut transitions = Vec::new();
        while !content.is_empty() {
            let from: Ident = content.parse()?;
            content.parse::<Token![->]>()?;
            let to: Ident = content.parse()?;
            content.parse::<kw::on>()?;
            let event: Ident = content.parse()?;
            content.parse::<Token![;]>()?;
            if let Some(dup) = transitions
                .iter()
                .find(|t: &&Transition| t.from == from && t.event == event)
            {
                return Err(syn::Error::new(
                    event.span(),
                    format!(
                        "`{}` already has a transition on `{}` (to `{}`)",
                        from, event, dup.to
                    ),
                ));
            }
            transitions.push(Transition { from, to, event });
        }
        if transitions.is_empty() {
            return Err(syn::Error::new(
                name.span(),
                "a state machine needs at least one transition",
            ));
        }
        Ok(Machine {
            name,
            initial,
            transitions,
        })
    }
}

/// Unique identifiers, in order of first appearance.
fn unique<'a>(idents: impl IntoIterator<Item = &'a Ident>) -> Vec<Ident> {
    let mut out: Vec<Ident> = Vec::new();
    for i in idents {
        if !out.contains(i) {
            out.push(i.clone());
        }
    }
    out
}

pub fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    let machine: Machine = syn::parse2(input)?;
    let name = &machine.name;
    let state_ty = format_ident!("{}State", name);
    let event_ty = format_ident!("{}Event", name);
    let error_ty = format_ident!("{}Error", name);
    let states = unique(
        std::iter::once(&machine.initial)
            .chain(machine.transitions.iter().flat_map(|t| [&t.from, &t.to])),
    );
    let events = unique(machine.transitions.iter().map(|t| &t.event));
    let initial = &machine.initial;
    let arms = machine.transitions.iter().map(|t| {
        let (from, to, event) = (&t.from, &t.to, &t.event);
        quote!((#state_ty::#from, #event_ty::#event) => Some(#state_ty::#to))
    });
    let table = machine.transitions.iter().map(|t| {
        let (from, to, event) = (&t.from, &t.to, &t.event);
        quote!((#state_ty::#from, #event_ty::#event, #state_ty::#to))
    });

    Ok(quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum #state_ty { #(#states),* }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum #event_ty { #(#events),* }

        /// An event that has no transition from the current state.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct #error_ty {
            pub state: #state_ty,
            pub event: #event_ty,
        }

        impl ::std::fmt::Display for #error_ty {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "no transition from {:?} on {:?}", self.state, self.event)
            }
        }

        impl ::std::error::Error for #error_ty {}

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct #name {
            state: #state_ty,
        }

        impl ::std::default::Default for #name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl #name {
            /// Every transition: (from, event, to).
            pub const TRANSITIONS: &'static [(#state_ty, #event_ty, #state_ty)] = &[#(#table),*];

            pub fn new() -> Self {
                #name { state: #state_ty::#initial }
            }

            pub fn state(&self) -> #state_ty {
                self.state
            }

            fn next(state: #state_ty, event: #event_ty) -> ::std::option::Option<#state_ty> {
                match (state, event) {
                    #(#arms,)*
                    #[allow(unreachable_patterns)]
                    _ => None,
                }
            }

            pub fn can_fire(&self, event: #event_ty) -> bool {
                Self::next(self.state, event).is_some()
            }

            /// Take the transition, or report there is none (the state is unchanged).
            pub fn fire(&mut self, event: #event_ty) -> ::std::result::Result<#state_ty, #error_ty> {
                match Self::next(self.state, event) {
                    Some(to) => {
                        self.state = to;
                        Ok(to)
                    }
                    None => Err(#error_ty { state: self.state, event }),
                }
            }
        }
    })
}
