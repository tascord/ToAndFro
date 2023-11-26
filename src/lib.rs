use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data};

#[proc_macro_derive(ToAndFro)]
pub fn tf_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    let name = input.ident;
    let data = match input.data {
        Data::Enum(data) => data,
        _ => panic!("Display can only be implemented for enums"),
    };

    let debug_arms = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        quote! {
            #name::#variant_name => write!(f, stringify!(#variant_name)),
        }
    });

    let display_arms = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        quote! {
            #name::#variant_name => write!(f, stringify!(#variant_name)),
        }
    });

    let from_str_arms = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        quote! {
            stringify!(#variant_name) => Ok(#name::#variant_name),
        }
    });

    let expanded = quote! {

        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                    #(#debug_arms)*
                }
            }
        }

        impl std::cmp::PartialEq for #name {
            fn eq(&self, other: &Self) -> bool {
                std::mem::discriminant(self) == std::mem::discriminant(other)
            }
        }

        // -- //

        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                    #(#display_arms)*
                }
            }
        }

        impl std::str::FromStr for #name {
            type Err = anyhow::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    #(#from_str_arms)*
                    _ => Err(anyhow::anyhow!("Invalid variant {} for enum {}", s, stringify!(#name))),
                }
            }
        }

    };    

    TokenStream::from(expanded)
}