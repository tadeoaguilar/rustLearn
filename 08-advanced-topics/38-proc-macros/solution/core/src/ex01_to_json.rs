//! Exercise 1: `#[derive(ToJson)]`.
//!
//! - structs with named fields -> a JSON object, fields in declaration order
//! - `#[json(rename = "name")]` changes a field's key; `#[json(skip)]` leaves it out
//! - enums: a unit variant -> its name as a string (or its rename); a variant
//!   with fields -> `{"Variant": <fields as an object or array>}`
//! - generic parameters get a `ToJson` bound
//! - tuple structs -> arrays; unit structs -> `null`
//!
//! The generated impl calls `ToJson::to_json` on each field, so any field
//! type that implements the trait works -- including other derived types.

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Fields, Path, parse_quote};

use crate::attrs;

pub fn derive(input: TokenStream, krate: &Path) -> syn::Result<TokenStream> {
    let mut input: DeriveInput = syn::parse2(input)?;
    let name = &input.ident;
    let trait_path: Path = parse_quote!(#krate::json::ToJson);
    // Every type parameter must itself be ToJson.
    for param in input.generics.type_params_mut() {
        param.bounds.push(parse_quote!(#trait_path));
    }
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let body = match &input.data {
        Data::Struct(s) => fields_to_json(&s.fields, Access::SelfFields, &trait_path)?,
        Data::Enum(e) => {
            let mut arms = Vec::new();
            for variant in &e.variants {
                let options = attrs::parse(&variant.attrs, "json", &["rename"])?;
                let v = &variant.ident;
                let key = options
                    .value("rename")
                    .map_or_else(|| v.to_string(), str::to_string);
                let key_json = json_string(&key);
                arms.push(match &variant.fields {
                    Fields::Unit => quote!(Self::#v => #key_json.to_string()),
                    Fields::Named(named) => {
                        let idents: Vec<_> = named
                            .named
                            .iter()
                            .map(|f| f.ident.clone().expect("named"))
                            .collect();
                        let inner = fields_to_json(&variant.fields, Access::Bindings, &trait_path)?;
                        quote!(Self::#v { #(#idents),* } => format!("{{{}:{}}}", #key_json, #inner))
                    }
                    Fields::Unnamed(unnamed) => {
                        let idents: Vec<_> = (0..unnamed.unnamed.len())
                            .map(|i| syn::Ident::new(&format!("f{i}"), Span::call_site()))
                            .collect();
                        let inner = fields_to_json(&variant.fields, Access::Bindings, &trait_path)?;
                        quote!(Self::#v ( #(#idents),* ) => format!("{{{}:{}}}", #key_json, #inner))
                    }
                });
            }
            if arms.is_empty() {
                return Err(syn::Error::new_spanned(
                    name,
                    "ToJson can't be derived for an enum with no variants",
                ));
            }
            quote!(match self { #(#arms,)* })
        }
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                name,
                "ToJson can't be derived for unions",
            ));
        }
    };

    Ok(quote! {
        impl #impl_generics #trait_path for #name #ty_generics #where_clause {
            fn to_json(&self) -> String {
                #body
            }
        }
    })
}

/// `"text"` as a JSON string literal (for keys known at expansion time).
pub fn json_string(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// How generated code reaches the fields.
#[derive(Clone, Copy)]
enum Access {
    /// A struct: `&self.name`, `&self.0`.
    SelfFields,
    /// An enum variant matched on `&self`: the bindings `name`, `f0`, ...
    Bindings,
}

/// An expression building the JSON for `fields`.
fn fields_to_json(fields: &Fields, access: Access, trait_path: &Path) -> syn::Result<TokenStream> {
    match fields {
        Fields::Named(named) => {
            let mut parts = Vec::new();
            for field in &named.named {
                let options = attrs::parse(&field.attrs, "json", &["rename", "skip"])?;
                if options.has("skip") {
                    continue;
                }
                let ident = field.ident.as_ref().expect("named field");
                let key = options
                    .value("rename")
                    .map_or_else(|| ident.to_string(), str::to_string);
                let key_json = json_string(&key);
                let value = match access {
                    Access::SelfFields => quote!(&self.#ident),
                    Access::Bindings => quote!(#ident),
                };
                parts.push(quote!(
                    format!("{}:{}", #key_json, #trait_path::to_json(#value))
                ));
            }
            Ok(quote!({
                let parts: ::std::vec::Vec<String> = vec![#(#parts),*];
                format!("{{{}}}", parts.join(","))
            }))
        }
        Fields::Unnamed(unnamed) => {
            let parts: Vec<TokenStream> = (0..unnamed.unnamed.len())
                .map(|i| {
                    let value = match access {
                        Access::SelfFields => {
                            let index = syn::Index::from(i);
                            quote!(&self.#index)
                        }
                        Access::Bindings => {
                            let ident = syn::Ident::new(&format!("f{i}"), Span::call_site());
                            quote!(#ident)
                        }
                    };
                    quote!(#trait_path::to_json(#value))
                })
                .collect();
            Ok(quote!({
                let parts: ::std::vec::Vec<String> = vec![#(#parts),*];
                format!("[{}]", parts.join(","))
            }))
        }
        Fields::Unit => Ok(quote!(String::from("null"))),
    }
}
