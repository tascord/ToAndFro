use quote::quote;
use syn::{punctuated::Punctuated, token::Comma, Variant};

pub fn get_name_for(attrs: &Vec<syn::Attribute>, ident: &str) -> Option<String> {
    attrs
        .iter()
        .find(|attr| attr.path().is_ident(ident))
        .map(|attr| attr.parse_args::<syn::LitStr>().unwrap().value())
        .or(attrs
            .iter()
            .find(|attr| attr.path().is_ident("rename"))
            .map(|attr| attr.parse_args::<syn::LitStr>().unwrap().value()))
}

pub fn get_alias_for(attrs: &Vec<syn::Attribute>) -> Option<String> {
    attrs
        .iter()
        .find(|attr| attr.path().is_ident("alias"))
        .map(|attr| attr.parse_args::<syn::LitStr>().unwrap().value())
}

pub fn create_alias_arms(variants: Punctuated<Variant, Comma>) -> proc_macro2::TokenStream {
    variants
        .iter()
        .map(|variant| match get_alias_for(&variant.attrs) {
            Some(alias) => {
                let variant_name = &variant.ident;
                quote! {
                    #alias => Self::#variant_name
                }
            }
            None => quote!(),
        })
        .collect()
}
