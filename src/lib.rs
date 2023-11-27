use heck::{
    AsKebabCase, AsLowerCamelCase, AsPascalCase, AsShoutyKebabCase, AsShoutySnakeCase, AsSnakeCase,
    AsTitleCase, AsTrainCase, AsUpperCamelCase,
};

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::rc::Rc;
use syn::{parse_macro_input, Data, DeriveInput};

type Caser = Rc<Box<dyn Fn(&str) -> String + Send + Sync + 'static>>;
static CASES: [&str; 11] = [
    "kebab",
    "lower_camel",
    "pascal",
    "shouty_kebab",
    "shouty_snake",
    "snake",
    "title",
    "train",
    "upper_camel",
    "upper",
    "lower",
];

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

fn should_reject(attrs: &Vec<syn::Attribute>) -> bool {
    attrs
        .iter()
        .find(|attr| attr.path().is_ident("reject"))
        .is_some()
}

fn check_case(args: TokenStream) {
    let mut args = args.into_iter();
    if args.clone().count() != 1 {
        panic!("Expected one argument");
    }

    let arg = args.next().unwrap();
    if !CASES.contains(&arg.to_string().as_str()) {
        panic!("Invalid casing {}", arg);
    }
}

/// Generate automatic implementations of `FromStr`, `Display`, `Debug`, and `PartialEq` for an enum.
#[proc_macro_derive(ToAndFro, attributes(input_case, output_case, default, reject))]
pub fn tf_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let data = match input.data {
        Data::Enum(data) => data,
        _ => panic!("Display can only be implemented for enums"),
    };

    let d_caser: Caser = Rc::new(Box::new(|s| s.to_string()));
    let i_caser = match_supplied_casing("input_case", &input.attrs).unwrap_or(d_caser.clone());
    let o_caser = match_supplied_casing("output_case", &input.attrs).unwrap_or(d_caser.clone());

    let from_str_failure = match input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("default"))
        .map(|attr| attr.parse_args::<syn::LitStr>().unwrap().value())
    {
        Some(x) => {
            let ident = format_ident!("{}", x);
            quote!(Ok(#name::#ident))
        }
        None => quote!(Err(anyhow::anyhow!(
            "Invalid variant {} for enum {}",
            s,
            stringify!(#name)
        ))),
    };

    let from_str_arms = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        if should_reject(&variant.attrs) {
            return quote!();
        }

        let caser = match_supplied_casing("input_case", &variant.attrs).unwrap_or(i_caser.clone());
        let cased_variant = caser(variant_name.to_string().as_str());

        quote! {
            #cased_variant => Ok(#name::#variant_name),
        }
    });

    let display_arms = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let caser = match_supplied_casing("output_case", &variant.attrs).unwrap_or(o_caser.clone());
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
                    _ => #from_str_failure
                }
            }
        }

    };

    TokenStream::from(expanded)
}

/// Define the case to expect when parsing a variant from a string.
/// Valid values are:
/// - `kebab` [(heck)](https://docs.rs/heck/latest/heck/struct.AsKebabCase.html)
/// - `pascal` [(heck)](https://docs.rs/heck/latest/heck/struct.AsPascalCase.html)
/// - `snake` [(heck)](https://docs.rs/heck/latest/heck/struct.AsSnakeCase.html)
/// - `title` [(heck)](https://docs.rs/heck/latest/heck/struct.AsTitleCase.html)
/// - `train` [(heck)](https://docs.rs/heck/latest/heck/struct.AsTrainCase.html)
/// - `lower_camel` [(heck)](https://docs.rs/heck/latest/heck/struct.AsLowerCamelCase.html)
/// - `upper_camel` [(heck)](https://docs.rs/heck/latest/heck/struct.AsUpperCamelCase.html)
/// - `shouty_kebab` [(heck)](https://docs.rs/heck/latest/heck/struct.AsShoutyKebabCase.html)
/// - `shouty_snake` [(heck)](https://docs.rs/heck/latest/heck/struct.AsShoutySnakeCase.html)
/// - `upper` (UPPERCASE)
/// - `lower` (lowercase)
#[proc_macro_attribute]
pub fn input_case(args: TokenStream, input: TokenStream) -> TokenStream {
    check_case(args);
    input
}

/// Define the case to stringify to through Display, or Debug.
/// Valid values are:
/// - `kebab` [(heck)](https://docs.rs/heck/latest/heck/struct.AsKebabCase.html)
/// - `pascal` [(heck)](https://docs.rs/heck/latest/heck/struct.AsPascalCase.html)
/// - `snake` [(heck)](https://docs.rs/heck/latest/heck/struct.AsSnakeCase.html)
/// - `title` [(heck)](https://docs.rs/heck/latest/heck/struct.AsTitleCase.html)
/// - `train` [(heck)](https://docs.rs/heck/latest/heck/struct.AsTrainCase.html)
/// - `lower_camel` [(heck)](https://docs.rs/heck/latest/heck/struct.AsLowerCamelCase.html)
/// - `upper_camel` [(heck)](https://docs.rs/heck/latest/heck/struct.AsUpperCamelCase.html)
/// - `shouty_kebab` [(heck)](https://docs.rs/heck/latest/heck/struct.AsShoutyKebabCase.html)
/// - `shouty_snake` [(heck)](https://docs.rs/heck/latest/heck/struct.AsShoutySnakeCase.html)
/// - `upper` (UPPERCASE)
/// - `lower` (lowercase)
#[proc_macro_attribute]
pub fn output_case(args: TokenStream, input: TokenStream) -> TokenStream {
    check_case(args);
    input
}

/// Defines the field to default to when parsing fails.
/// ```rs
/// #[derive(ToAndFro)]
/// #[default("Load")]
/// pub enum TestEnum {
///   Generation,
///   Load,
///   Customers,
/// }
///
/// assert_eq!(TestEnum::from_str("Uncaught Case").unwrap(), TestEnum::Load);
/// ```
#[proc_macro_attribute]
pub fn default(args: TokenStream, input: TokenStream) -> TokenStream {
    if args.clone().into_iter().count() != 1 {
        panic!("#[default(\"...\")] takes one argument");
    }

    input
}

/// Rejects the variant from being parsed from a String.
/// This either throws an Error on parse, or defaults to the variant specified with `default`.
#[proc_macro_attribute]
pub fn reject(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        panic!("#[reject] does not take arguments");
    }

    input
}
