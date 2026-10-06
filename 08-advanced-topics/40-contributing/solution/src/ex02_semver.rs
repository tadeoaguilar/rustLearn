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
    let file = syn::parse_file(source)?;
    let mut api = Api::new();
    collect(&file.items, "", &mut api);
    Ok(api)
}

fn collect(items: &[Item], prefix: &str, api: &mut Api) {
    for item in items {
        match item {
            Item::Fn(f) if is_pub(&f.vis) && !doc_hidden(&f.attrs) => {
                api.insert(format!("fn {prefix}{}", f.sig.ident), tokens(&f.sig));
            }
            Item::Struct(s) if is_pub(&s.vis) && !doc_hidden(&s.attrs) => {
                let name = format!("{prefix}{}", s.ident);
                let all_pub = s.fields.iter().all(|f| is_pub(&f.vis));
                let constructible = if all_pub { " constructible" } else { "" };
                api.insert(
                    format!("struct {name}"),
                    format!("{}{constructible}", exhaustiveness(&s.attrs)),
                );
                for (i, field) in s.fields.iter().enumerate().filter(|(_, f)| is_pub(&f.vis)) {
                    let field_name = field
                        .ident
                        .as_ref()
                        .map_or_else(|| i.to_string(), ToString::to_string);
                    api.insert(format!("field {name}.{field_name}"), tokens(&field.ty));
                }
            }
            Item::Enum(e) if is_pub(&e.vis) && !doc_hidden(&e.attrs) => {
                let name = format!("{prefix}{}", e.ident);
                api.insert(format!("enum {name}"), exhaustiveness(&e.attrs).to_string());
                for variant in &e.variants {
                    let fields = match &variant.fields {
                        Fields::Unit => String::new(),
                        fields => tokens(fields),
                    };
                    api.insert(format!("variant {name}::{}", variant.ident), fields);
                }
            }
            Item::Trait(t) if is_pub(&t.vis) && !doc_hidden(&t.attrs) => {
                let name = format!("{prefix}{}", t.ident);
                api.insert(format!("trait {name}"), "trait".into());
                for trait_item in &t.items {
                    if let TraitItem::Fn(f) = trait_item {
                        let kind = if f.default.is_some() {
                            "provided"
                        } else {
                            "required"
                        };
                        api.insert(
                            format!("trait_fn {name}::{}", f.sig.ident),
                            format!("{} [{kind}]", tokens(&f.sig)),
                        );
                    }
                }
            }
            Item::Impl(imp) => {
                let self_ty = tokens(&imp.self_ty);
                if let Some((_, trait_path, _)) = &imp.trait_ {
                    api.insert(
                        format!("impl {} for {prefix}{self_ty}", tokens(trait_path)),
                        "impl".into(),
                    );
                } else {
                    for impl_item in &imp.items {
                        if let ImplItem::Fn(f) = impl_item
                            && is_pub(&f.vis)
                            && !doc_hidden(&f.attrs)
                        {
                            api.insert(
                                format!("fn {prefix}{self_ty}::{}", f.sig.ident),
                                tokens(&f.sig),
                            );
                        }
                    }
                }
            }
            Item::Mod(m) if is_pub(&m.vis) => {
                if let Some((_, items)) = &m.content {
                    collect(items, &format!("{prefix}{}::", m.ident), api);
                }
            }
            _ => {}
        }
    }
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
    // a new required trait method breaks every implementor
    if key.starts_with("trait_fn ")
        && new_sig.ends_with("[required]")
        && old.contains_key(&format!("trait {}", trait_of(key)))
    {
        return Bump::Major;
    }
    match parent_key(key) {
        // a new variant breaks exhaustive `match`es; a new pub field breaks struct literals
        Some(parent) => match old.get(&parent) {
            Some(sig) if key.starts_with("variant ") && sig == "exhaustive" => Bump::Major,
            Some(sig) if key.starts_with("field ") && sig == "exhaustive constructible" => {
                Bump::Major
            }
            _ => Bump::Minor,
        },
        None => Bump::Minor,
    }
}

fn trait_of(trait_fn_key: &str) -> &str {
    let rest = trait_fn_key.trim_start_matches("trait_fn ");
    rest.rsplit_once("::").map_or(rest, |(t, _)| t)
}

/// What changing `key`'s signature requires.
fn bump_for_change(key: &str, old_sig: &str, new_sig: &str) -> Bump {
    if key.starts_with("struct ") || key.starts_with("enum ") {
        // becoming non_exhaustive (or losing constructibility) breaks users;
        // the other direction only allows more
        let became_exhaustive =
            old_sig.starts_with("non_exhaustive") && new_sig.starts_with("exhaustive");
        let became_constructible =
            !old_sig.ends_with("constructible") && new_sig.ends_with("constructible");
        let only_relaxed = (became_exhaustive
            || old_sig.split(' ').next() == new_sig.split(' ').next())
            && (became_constructible
                || old_sig.ends_with("constructible") == new_sig.ends_with("constructible"));
        return if only_relaxed {
            Bump::Minor
        } else {
            Bump::Major
        };
    }
    if key.starts_with("trait_fn ") {
        // a required method gaining a default is fine; anything else isn't
        let same_sig =
            old_sig.rsplit_once(" [").map(|p| p.0) == new_sig.rsplit_once(" [").map(|p| p.0);
        if same_sig && old_sig.ends_with("[required]") && new_sig.ends_with("[provided]") {
            return Bump::Minor;
        }
    }
    Bump::Major
}

/// Every difference between two APIs, sorted by key.
pub fn diff(old: &Api, new: &Api) -> Vec<Change> {
    let mut changes = Vec::new();
    for (key, old_sig) in old {
        match new.get(key) {
            None => changes.push(Change {
                key: key.clone(),
                kind: ChangeKind::Removed,
                bump: Bump::Major,
            }),
            Some(new_sig) if new_sig != old_sig => changes.push(Change {
                key: key.clone(),
                kind: ChangeKind::Changed,
                bump: bump_for_change(key, old_sig, new_sig),
            }),
            Some(_) => {}
        }
    }
    for (key, new_sig) in new {
        if !old.contains_key(key) {
            changes.push(Change {
                key: key.clone(),
                kind: ChangeKind::Added,
                bump: bump_for_addition(key, new_sig, old),
            });
        }
    }
    changes.sort_by(|a, b| a.key.cmp(&b.key));
    changes
}

/// The bump a set of changes requires (`Patch` if there are none).
pub fn required_bump(changes: &[Change]) -> Bump {
    changes.iter().map(|c| c.bump).max().unwrap_or(Bump::Patch)
}

/// The next version under Cargo's semver rules: in `0.y.z`, `y` acts as the
/// major version; in `0.0.z`, every release is breaking.
pub fn next_version(current: &str, bump: Bump) -> Result<String, String> {
    let parts: Vec<u64> = current
        .split('.')
        .map(|p| {
            p.parse::<u64>()
                .map_err(|_| format!("not a MAJOR.MINOR.PATCH version: {current:?}"))
        })
        .collect::<Result<_, _>>()?;
    let [major, minor, patch] = parts[..] else {
        return Err(format!("not a MAJOR.MINOR.PATCH version: {current:?}"));
    };
    Ok(match (major, minor, bump) {
        (0, 0, _) => format!("0.0.{}", patch + 1),
        (0, _, Bump::Major) => format!("0.{}.0", minor + 1),
        (0, _, _) => format!("0.{minor}.{}", patch + 1),
        (_, _, Bump::Major) => format!("{}.0.0", major + 1),
        (_, _, Bump::Minor) => format!("{major}.{}.0", minor + 1),
        (_, _, Bump::Patch) => format!("{major}.{minor}.{}", patch + 1),
    })
}
