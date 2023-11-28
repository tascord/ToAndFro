use casing::{match_supplied_casing, Caser, CASES};
use defaults::{default_impl, fromstr_failure};
use naming::{create_alias_arms, get_name_for};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::rc::Rc;
use syn::{parse_macro_input, punctuated::Punctuated, Data, DataEnum, DeriveInput, Ident, Variant, ExprLit};

mod casing;
mod defaults;
mod naming;

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

fn map_variant(
    variants: &Punctuated<Variant, syn::token::Comma>,
    input_attrs: &Vec<syn::Attribute>,
    input_mode: bool,
    mut cb: impl FnMut(&Ident, String) -> proc_macro2::TokenStream,
) -> Vec<proc_macro2::TokenStream> {
    let case_mode = if input_mode {
        "input_case"
    } else {
        "output_case"
    };
    let name_mode = if input_mode {
        "input_name"
    } else {
        "output_name"
    };

    let default_caser: Caser = Rc::new(Box::new(|s| s.to_string()));
    let default_caser = match_supplied_casing(case_mode, &input_attrs).unwrap_or(default_caser);

    variants
        .iter()
        .map(|variant| {
            if input_mode && should_reject(&variant.attrs) {
                return quote!();
            }

            let caser =
                match_supplied_casing(case_mode, &variant.attrs).unwrap_or(default_caser.clone());

            let variant_name = &variant.ident;
            let string_name = &match get_name_for(&variant.attrs, name_mode) {
                Some(name) => format_ident!("{}", name),
                None => variant_name.clone(),
            };

            let cased_name = caser(string_name.to_string().as_str());
            cb(variant_name, cased_name)
        })
        .collect()
}

fn preamble(input: DeriveInput) -> (DeriveInput, Ident, DataEnum) {
    let name = input.clone().ident;
    let data = match input.clone().data {
        Data::Enum(data) => data,
        _ => panic!("Display can only be implemented for enums"),
    };

    (input, name, data)
}

/// Generate automatic implementations of `FromStr`, `Display`, `Debug`, and `PartialEq` for an enum.
#[proc_macro_derive(
    ToAndFro,
    attributes(
        input_case,
        output_case,
        default,
        reject,
        casing,
        alias,
        rename,
        input_name,
        output_name
    )
)]
pub fn tf_derive(input: TokenStream) -> TokenStream {
    let (input, name, data) = preamble(parse_macro_input!(input as DeriveInput));

    // Generated based on field attrs
    let from_str_failure = fromstr_failure(name.clone(), &input.attrs);
    let default_impl = default_impl(name.clone(), &input.attrs);
    let alias_arms = create_alias_arms(data.variants.clone());

    // Generated based on variants
    let from_str_arms = map_variant(
        &data.variants,
        &input.attrs,
        true,
        |variant_name, cased_name| {
            quote! {
                #cased_name => Ok(Self::#variant_name),
            }
        },
    );

    // Generated based on variants
    let display_arms = map_variant(
        &data.variants,
        &input.attrs,
        false,
        |variant_name, cased_name| {
            quote! {
                Self::#variant_name => write!(f, #cased_name),
            }
        },
    );

    // Debug uses the same arms as Display
    let debug_arms = display_arms.clone();
    let expanded = quote! {

        #default_impl

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
                    #alias_arms
                    _ => #from_str_failure
                }
            }
        }

    };

    TokenStream::from(expanded)
}

/// Define the default case to expect for both parsing, or stringifying.
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
pub fn casing(args: TokenStream, input: TokenStream) -> TokenStream {
    check_case(args);
    input
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
/// Also generates a `Default` implimentation pointing to the default variant.
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

/// Defines an alias for a variant, allowing an alternate name to be used when parsing FromStr.
/// ```rs
/// #[derive(ToAndFro)]
/// pub enum TestEnum {
///  Generation,
///  Load,
///  #[alias("People")]
///  Customers,
/// }
///
/// assert_eq!(TestEnum::from_str("People").unwrap(), TestEnum::Customers);
/// ```
#[proc_macro_attribute]
pub fn alias(args: TokenStream, input: TokenStream) -> TokenStream {
    if args.clone().into_iter().count() != 1 {
        panic!("#[alias(\"...\")] takes one argument");
    }

    input
}

/// Redefines the name of the Variant for purposes of Input and Output, ignoring casing.
/// ```rs
/// #[derive(ToAndFro)]
///   pub enum TestEnum {
///   Generation,
///   Load,
///   #[rename("People")]
///   Customers,
/// }
///
/// assert_eq!(TestEnum::from_str("People").unwrap(), TestEnum::Customers);
/// assert!(TestEnum::from_str("Load").is_err());
/// ```
#[proc_macro_attribute]
pub fn rename(args: TokenStream, input: TokenStream) -> TokenStream {
    if args.clone().into_iter().count() != 1 {
        panic!("#[rename(\"...\")] takes one argument");
    }

    input
}
