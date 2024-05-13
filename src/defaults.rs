use quote::{format_ident, quote};
use syn::Ident;

pub fn get_default_variant(input_attrs: &[syn::Attribute]) -> Option<String> {
    input_attrs
        .iter()
        .find(|attr| attr.path().is_ident("default"))
        .map(|attr| attr.parse_args::<syn::LitStr>().unwrap().value())
}

pub fn fromstr_failure(
    enum_name: Ident,
    input_attrs: &[syn::Attribute],
) -> proc_macro2::TokenStream {
    match get_default_variant(input_attrs) {
        Some(ref x) => {
            let ident = format_ident!("{}", x);
            quote!(Ok(#enum_name::#ident))
        }
        None => quote!(Err(anyhow::anyhow!(
            "Invalid variant {} for enum {}",
            s,
            stringify!(#enum_name)
        ))),
    }
}

pub fn default_impl(enum_name: Ident, input_attrs: &[syn::Attribute]) -> proc_macro2::TokenStream {
    match get_default_variant(input_attrs) {
        Some(ref x) => {
            let ident = format_ident!("{}", x);
            quote! {
                impl std::default::Default for #enum_name {
                    fn default() -> Self {
                        #enum_name::#ident
                    }
                }
            }
        }
        None => quote!(),
    }
}
