//! Bonus: `#[derive(EnumIter)]` for enums of unit variants.
//!
//! Generates `ALL` (every variant, in order), `name()` (the variant's name),
//! `from_name` (the inverse) and `COUNT`. A variant with fields is a compile
//! error at that variant.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

pub fn derive(input: TokenStream) -> syn::Result<TokenStream> {
    let input: DeriveInput = syn::parse2(input)?;
    let name = &input.ident;
    let Data::Enum(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            name,
            "EnumIter can only be derived for enums",
        ));
    };
    let mut variants = Vec::new();
    for variant in &data.variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Err(syn::Error::new_spanned(
                &variant.fields,
                "EnumIter needs unit variants (no fields)",
            ));
        }
        variants.push(&variant.ident);
    }
    let count = variants.len();
    let names: Vec<String> = variants.iter().map(|v| v.to_string()).collect();
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            pub const COUNT: usize = #count;
            pub const ALL: [Self; #count] = [#(Self::#variants),*];

            pub fn name(&self) -> &'static str {
                match self { #(Self::#variants => #names,)* }
            }

            pub fn from_name(name: &str) -> ::std::option::Option<Self> {
                match name {
                    #(#names => Some(Self::#variants),)*
                    _ => None,
                }
            }
        }
    })
}
