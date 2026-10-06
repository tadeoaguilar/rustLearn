//! Exercise 2: a semver checker -- what `cargo-semver-checks` does.
//!
//! Before releasing, a maintainer must pick the version number: under
//! semver, a breaking change needs a new major version. Getting it wrong
//! breaks every dependent crate on `cargo update`. This compares the public
//! API of two versions of a crate (as source) and reports each change with
//! the bump it requires.
//!
//! The public API is a map from a key to a signature:
//!
//! | Key | Signature |
//! |---|---|
//! | `fn parse` | the function's signature tokens |
//! | `fn Config::new` | an inherent `pub fn` in `impl Config` |
//! | `struct Config` | `exhaustive` or `non_exhaustive`, and `constructible` if every field is pub |
//! | `field Config.name` | the field's type |
//! | `enum Level` | `exhaustive` or `non_exhaustive` |
//! | `variant Level::Warn` | the variant's fields |
//! | `trait Store` | `trait` |
//! | `trait_fn Store::get` | the signature, then ` [required]` or ` [provided]` |
//! | `impl Display for Config` | `impl` |
//!
//! Items in `pub mod a` get the prefix `a::` (`fn a::helper`). Only `pub`
//! items count (not `pub(crate)`), and `#[doc(hidden)]` ones are skipped.

use std::collections::BTreeMap;
use std::fmt;

use quote::ToTokens;
use syn::{Attribute, Fields, ImplItem, Item, TraitItem, Visibility};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Bump {
    Patch,
    Minor,
    Major,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    Added,
    Removed,
    Changed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    pub key: String,
    pub kind: ChangeKind,
    pub bump: Bump,
}

impl fmt::Display for Change {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} {} ({:?})", self.kind, self.key, self.bump)
    }
}

pub type Api = BTreeMap<String, String>;

fn is_pub(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
}

fn has_attr(attrs: &[Attribute], name: &str) -> bool {
    attrs.iter().any(|a| a.path().is_ident(name))
}

fn doc_hidden(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path().is_ident("doc") && a.parse_args::<syn::Ident>().is_ok_and(|i| i == "hidden")
    })
}

fn exhaustiveness(attrs: &[Attribute]) -> &'static str {
    if has_attr(attrs, "non_exhaustive") {
        "non_exhaustive"
    } else {
        "exhaustive"
    }
}

fn tokens(t: &impl ToTokens) -> String {
    t.to_token_stream().to_string()
}

/// The public API of a crate's source.
pub fn public_api(source: &str) -> syn::Result<Api> {
    todo!("Exercise 2")
}

fn collect(items: &[Item], prefix: &str, api: &mut Api) {
    todo!("Exercise 2")
}

/// The struct or enum that a `field X.y` / `variant X::Y` key belongs to.
fn parent_key(key: &str) -> Option<String> {
    if let Some(rest) = key.strip_prefix("field ") {
        return rest
            .rsplit_once('.')
            .map(|(parent, _)| format!("struct {parent}"));
    }
    if let Some(rest) = key.strip_prefix("variant ") {
        return rest
            .rsplit_once("::")
            .map(|(parent, _)| format!("enum {parent}"));
    }
    None
}

/// What adding `key` (with signature `new_sig`) requires, given the old API.
fn bump_for_addition(key: &str, new_sig: &str, old: &Api) -> Bump {
    todo!("Exercise 2")
}

fn trait_of(trait_fn_key: &str) -> &str {
    let rest = trait_fn_key.trim_start_matches("trait_fn ");
    rest.rsplit_once("::").map_or(rest, |(t, _)| t)
}

/// What changing `key`'s signature requires.
fn bump_for_change(key: &str, old_sig: &str, new_sig: &str) -> Bump {
    todo!("Exercise 2")
}

/// Every difference between two APIs, sorted by key.
pub fn diff(old: &Api, new: &Api) -> Vec<Change> {
    todo!("Exercise 2")
}

/// The bump a set of changes requires (`Patch` if there are none).
pub fn required_bump(changes: &[Change]) -> Bump {
    todo!("Exercise 2")
}

/// The next version under Cargo's semver rules: in `0.y.z`, `y` acts as the
/// major version; in `0.0.z`, every release is breaking.
pub fn next_version(current: &str, bump: Bump) -> Result<String, String> {
    todo!("Exercise 2")
}
