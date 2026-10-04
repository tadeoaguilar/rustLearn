//! Procedural macros for module 13.
//!
//! Every macro has the same three steps:
//!   1. parse the input tokens into a syntax tree (`syn`)
//!   2. compute what to generate
//!   3. build the output tokens (`quote!`), with `#var` interpolation
//!
//! Errors are reported with `syn::Error`, which points the compiler's error
//! message at the offending span in the *user's* code.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    Data, DeriveInput, Fields, GenericArgument, ItemFn, LitStr, PathArguments, ReturnType, Type,
    parse_macro_input,
};

// ---- Exercise 6: #[derive(Describe)] ----------------------------------------

/// Adds `describe()` (and `field_names()` for named-field structs).
#[proc_macro_derive(Describe)]
pub fn derive_describe(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match describe_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn describe_impl(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let (text, field_names) = match &input.data {
        Data::Struct(s) => {
            let names: Vec<String> = match &s.fields {
                Fields::Named(f) => f
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap().to_string())
                    .collect(),
                _ => Vec::new(),
            };
            (
                format!("struct {name}{}", describe_fields(&s.fields)),
                Some(names),
            )
        }
        Data::Enum(e) => {
            let variants: Vec<String> = e
                .variants
                .iter()
                .map(|v| format!("{}{}", v.ident, describe_fields(&v.fields)))
                .collect();
            (format!("enum {name} {{ {} }}", variants.join(", ")), None)
        }
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "Describe can't be derived for unions",
            ));
        }
    };

    let field_names_fn = field_names.filter(|n| !n.is_empty()).map(|names| {
        quote! {
            pub fn field_names() -> &'static [&'static str] {
                &[#(#names),*]
            }
        }
    });

    Ok(quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            pub fn describe() -> ::std::string::String {
                ::std::string::String::from(#text)
            }
            #field_names_fn
        }
    })
}

/// ` { x: i32, y: i32 }`, `(f64)` or `` (unit).
fn describe_fields(fields: &Fields) -> String {
    match fields {
        Fields::Named(f) => {
            let parts: Vec<String> = f
                .named
                .iter()
                .map(|f| format!("{}: {}", f.ident.as_ref().unwrap(), type_string(&f.ty)))
                .collect();
            format!(" {{ {} }}", parts.join(", "))
        }
        Fields::Unnamed(f) => {
            let parts: Vec<String> = f.unnamed.iter().map(|f| type_string(&f.ty)).collect();
            format!("({})", parts.join(", "))
        }
        Fields::Unit => String::new(),
    }
}

/// `quote!(Vec<String>).to_string()` is "Vec < String >"; make it read like source.
fn type_string(ty: &Type) -> String {
    quote!(#ty)
        .to_string()
        .replace(" < ", "<")
        .replace("< ", "<")
        .replace(" <", "<")
        .replace(" >", ">")
        .replace(" ,", ",")
        .replace("& ", "&")
        .replace(" :: ", "::")
        .replace(":: ", "::")
}

// ---- Exercise 7: #[derive(Builder)] -----------------------------------------

/// `attributes(builder)` registers `#[builder(...)]` as a helper attribute,
/// so the compiler accepts it on fields of a type deriving Builder.
#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive_builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match builder_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

struct BuilderField<'a> {
    ident: &'a syn::Ident,
    ty: &'a Type,
    /// `Some(T)` if the field is `Option<T>`.
    option_inner: Option<&'a Type>,
    has_default: bool,
}

fn builder_impl(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let vis = &input.vis;
    let builder_name = format_ident!("{name}Builder");

    let Data::Struct(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            name,
            "Builder can only be derived for structs",
        ));
    };
    let Fields::Named(named) = &data.fields else {
        return Err(syn::Error::new_spanned(
            name,
            "Builder needs a struct with named fields",
        ));
    };
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "Builder doesn't support generic structs (yet)",
        ));
    }

    let fields = named
        .named
        .iter()
        .map(|f| {
            Ok(BuilderField {
                ident: f.ident.as_ref().expect("named field"),
                ty: &f.ty,
                option_inner: option_inner_type(&f.ty),
                has_default: has_builder_default(&f.attrs)?,
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;

    // The builder stores every field as Option<..>. For an `Option<T>` field
    // that's just the field's own type.
    let storage = fields.iter().map(|f| {
        let (ident, ty) = (f.ident, f.ty);
        if f.option_inner.is_some() {
            quote!(#ident: #ty)
        } else {
            quote!(#ident: ::std::option::Option<#ty>)
        }
    });
    let empty = fields.iter().map(|f| {
        let ident = f.ident;
        quote!(#ident: ::std::option::Option::None)
    });
    let setters = fields.iter().map(|f| {
        let ident = f.ident;
        let arg_ty = f.option_inner.unwrap_or(f.ty);
        let doc = format!("Sets `{ident}`.");
        quote! {
            #[doc = #doc]
            pub fn #ident(mut self, value: #arg_ty) -> Self {
                self.#ident = ::std::option::Option::Some(value);
                self
            }
        }
    });
    let build_fields = fields.iter().map(|f| {
        let ident = f.ident;
        if f.option_inner.is_some() {
            quote!(#ident: self.#ident)
        } else if f.has_default {
            quote!(#ident: self.#ident.unwrap_or_default())
        } else {
            let msg = format!("missing field `{ident}`");
            quote!(#ident: self.#ident.ok_or_else(|| ::std::string::String::from(#msg))?)
        }
    });

    let builder_doc = format!("Builder for [`{name}`], created by `{name}::builder()`.");
    Ok(quote! {
        #[doc = #builder_doc]
        #[derive(Debug, Default)]
        #vis struct #builder_name {
            #(#storage,)*
        }

        impl #name {
            /// Starts building with every field unset.
            #vis fn builder() -> #builder_name {
                #builder_name { #(#empty,)* }
            }
        }

        impl #builder_name {
            #(#setters)*

            /// Fails with "missing field `x`" for the first required field not set.
            pub fn build(self) -> ::std::result::Result<#name, ::std::string::String> {
                ::std::result::Result::Ok(#name { #(#build_fields,)* })
            }
        }
    })
}

/// `Option<T>` -> `Some(T)`. Matches the last path segment, so
/// `std::option::Option<T>` works too (a type alias named differently won't).
fn option_inner_type(ty: &Type) -> Option<&Type> {
    let Type::Path(p) = ty else { return None };
    let seg = p.path.segments.last()?;
    if seg.ident != "Option" {
        return None;
    }
    let PathArguments::AngleBracketed(args) = &seg.arguments else {
        return None;
    };
    match args.args.first()? {
        GenericArgument::Type(inner) if args.args.len() == 1 => Some(inner),
        _ => None,
    }
}

/// True for `#[builder(default)]`; an error for anything else inside `builder(...)`.
fn has_builder_default(attrs: &[syn::Attribute]) -> syn::Result<bool> {
    let mut found = false;
    for attr in attrs.iter().filter(|a| a.path().is_ident("builder")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("default") {
                found = true;
                Ok(())
            } else {
                Err(meta.error("expected `default`"))
            }
        })?;
    }
    Ok(found)
}

// ---- Bonus: #[timed] --------------------------------------------------------

/// Prints how long each call took to stderr. `#[timed]` or `#[timed("label")]`.
#[proc_macro_attribute]
pub fn timed(attr: TokenStream, item: TokenStream) -> TokenStream {
    let label = if attr.is_empty() {
        None
    } else {
        Some(parse_macro_input!(attr as LitStr).value())
    };
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = parse_macro_input!(item as ItemFn);
    if sig.asyncness.is_some() {
        return syn::Error::new_spanned(sig.fn_token, "#[timed] doesn't support async fns")
            .to_compile_error()
            .into();
    }
    let label = label.unwrap_or_else(|| sig.ident.to_string());
    let ret = match &sig.output {
        ReturnType::Default => quote!(()),
        ReturnType::Type(_, ty) => quote!(#ty),
    };
    // The body runs in a closure annotated with the function's return type,
    // so `return x` and `?` inside it mean exactly what they meant before.
    quote! {
        #(#attrs)*
        #vis #sig {
            let __timed_start = ::std::time::Instant::now();
            #[allow(clippy::redundant_closure_call)]
            let __timed_result = (move || -> #ret #block)();
            ::std::eprintln!("[timed] {} took {:?}", #label, __timed_start.elapsed());
            __timed_result
        }
    }
    .into()
}
