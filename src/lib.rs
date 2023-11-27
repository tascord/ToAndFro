use heck::{
    AsKebabCase, AsLowerCamelCase, AsPascalCase, AsShoutyKebabCase, AsShoutySnakeCase, AsSnakeCase,
    AsTitleCase, AsTrainCase, AsUpperCamelCase,
};

use proc_macro::TokenStream;
use quote::quote;
use std::rc::Rc;
use syn::{parse_macro_input, Data, DeriveInput};

type Caser = Rc<Box<dyn Fn(&str) -> String + Send + Sync + 'static>>;

fn match_supplied_casing(ident: &str, attrs: &Vec<syn::Attribute>) -> Option<Caser> {
    let casing = attrs
        .iter()
        .find(|attr| attr.path().is_ident(ident))
        .map(|attr| attr.parse_args::<syn::LitStr>().unwrap().value());

    if let Some(casing) = casing {
        match casing.as_str() {
            "kebab" => Some(Rc::new(Box::new(|s| AsKebabCase(s).to_string()))),
            "lower_camel" => Some(Rc::new(Box::new(|s| AsLowerCamelCase(s).to_string()))),
            "pascal" => Some(Rc::new(Box::new(|s| AsPascalCase(s).to_string()))),
            "shouty_kebab" => Some(Rc::new(Box::new(|s| AsShoutyKebabCase(s).to_string()))),
            "shouty_snake" => Some(Rc::new(Box::new(|s| AsShoutySnakeCase(s).to_string()))),
            "snake" => Some(Rc::new(Box::new(|s| AsSnakeCase(s).to_string()))),
            "title" => Some(Rc::new(Box::new(|s| AsTitleCase(s).to_string()))),
            "train" => Some(Rc::new(Box::new(|s| AsTrainCase(s).to_string()))),
            "upper_camel" => Some(Rc::new(Box::new(|s| AsUpperCamelCase(s).to_string()))),
            "upper" => Some(Rc::new(Box::new(|s| s.to_uppercase()))),
            "lower" => Some(Rc::new(Box::new(|s| s.to_lowercase()))),
            _ => panic!("Invalid casing {}", casing),
        }
    } else {
        None
    }
}

#[proc_macro_derive(ToAndFro, attributes(input_case, output_case))]
pub fn tf_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let data = match input.data {
        Data::Enum(data) => data,
        _ => panic!("Display can only be implemented for enums"),
    };

    let i_caser = match_supplied_casing("input_case", &input.attrs)
        .unwrap_or(Rc::new(Box::new(|s| s.to_string())));

    let o_caser = match_supplied_casing("output_case", &input.attrs)
        .unwrap_or(Rc::new(Box::new(|s| s.to_string())));

    let from_str_arms = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let caser =
            match_supplied_casing("input_case", &variant.attrs).unwrap_or(i_caser.clone());
        let cased_variant = caser(variant_name.to_string().as_str());

        quote! {
            #cased_variant => Ok(#name::#variant_name),
        }
    });

    let display_arms = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let caser =
            match_supplied_casing("output_case", &variant.attrs).unwrap_or(o_caser.clone());
        let cased_variant = caser(variant_name.to_string().as_str());

        quote! {
            #name::#variant_name => write!(f, #cased_variant),
        }
    });

    let debug_arms = display_arms.clone();
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
