use casing::{match_supplied_casing, Caser, CASES};
use defaults::{default_impl, fromstr_failure};
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use std::rc::Rc;
use syn::{parse_macro_input, punctuated::Punctuated, Data, DataEnum, DeriveInput, Ident, Variant};

mod casing;
mod defaults;

fn should_reject(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("reject"))
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
    input_attrs: &[syn::Attribute],
    case_attr: &str,
    reject_if_present: bool,
    mut cb: impl FnMut(&Ident, String) -> proc_macro2::TokenStream,
) -> Vec<proc_macro2::TokenStream> {
    let default_caser: Caser = Rc::new(Box::new(|s| s.to_string()));
    let default_caser = match_supplied_casing(case_attr, input_attrs).unwrap_or(default_caser);

    variants
        .iter()
        .map(|variant| {
            if reject_if_present && should_reject(&variant.attrs) {
                return quote!();
            }

            let caser =
                match_supplied_casing(case_attr, &variant.attrs).unwrap_or(default_caser.clone());

            let variant_name = &variant.ident;
            let cased_name = caser(variant_name.to_string().as_str());

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

/// Generate automatic implementations of `FromStr`, `TryFrom<str-like>`, `Display`, `Debug`, `PartialEq`, `Eq` and `Hash` for an enum.
#[proc_macro_derive(
    ToAndFro,
    attributes(input_case, output_case, default, reject, casing, serde)
)]
pub fn tf_derive(input: TokenStream) -> TokenStream {
    let (input, name, data) = preamble(parse_macro_input!(input as DeriveInput));

    // Generated based on default attr
    let from_str_failure = fromstr_failure(name.clone(), &input.attrs);
    let default_impl = default_impl(name.clone(), &input.attrs);

    // Generated based on variants
    let from_str_arms = map_variant(
        &data.variants,
        &input.attrs,
        "input_case",
        true,
        |variant_name, cased_name| {
            quote! {
                #cased_name => Ok(#name::#variant_name),
            }
        },
    );

    // Generated based on variants
    let display_arms = map_variant(
        &data.variants,
        &input.attrs,
        "output_case",
        false,
        |variant_name, cased_name| {
            quote! {
                #name::#variant_name => f.write_str(#cased_name),
            }
        },
    );

    // Generated based on variants
    let str_arms = map_variant(
        &data.variants,
        &input.attrs,
        "output_case",
        false,
        |variant_name, cased_name| {
            quote! {
                #name::#variant_name => #cased_name,
            }
        },
    );

    let clone_arms = map_variant(
        &data.variants,
        &input.attrs,
        "output_case",
        false,
        |variant_name, _| {
            quote! {
                #name::#variant_name => #name::#variant_name,
            }
        },
    );

    let variant_count = data.variants.len();
    let variants = data.variants.iter().map(|v| v.ident.to_token_stream());

    // Serde impl
    let serde_impl = &input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("serde"))
        .map(|_| {
            quote! {
                impl serde::Serialize for #name {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where
                        S: serde::Serializer,
                    {
                        serializer.serialize_str(&self.to_string())
                    }
                }

                impl<'de> serde::Deserialize<'de> for #name {
                    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                    where
                        D: serde::Deserializer<'de>,
                    {
                        let s = String::deserialize(deserializer)?;
                        Self::from_str(&s).map_err(serde::de::Error::custom)
                    }
                }
            }
        })
        .unwrap_or(quote!());

    // only allow list() on enums that don't have fields
    let list = if data.variants.iter().all(|v| v.fields.is_empty()) {
        quote!(
            impl #name {
                pub fn list() -> [#name; #variant_count] {
                    [
                    #( #name::#variants, )*
                    ]
                }
            }
        )
    } else {
        quote!()
    };

    let expanded = quote! {

        #default_impl
        #serde_impl
        #list

        impl #name {
            fn as_str(&self) -> &'static str {
                match self {
                    #(#str_arms)*
                }
            }
        }

        impl Clone for #name {
            fn clone(&self) -> #name {
                match self {
                    #(#clone_arms)*
                }
            }
        }

        impl Copy for #name {}

        impl std::cmp::PartialEq for #name {
            fn eq(&self, other: &Self) -> bool {
                std::mem::discriminant(self) == std::mem::discriminant(other)
            }
        }

        impl std::cmp::Eq for #name {}

        impl std::hash::Hash for #name {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                std::mem::discriminant(self).hash(state)
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
                    _ => #from_str_failure
                }
            }
        }

        impl std::convert::TryFrom<std::rc::Rc<str>> for #name {
            type Error = anyhow::Error;

            fn try_from(s: std::rc::Rc<str>) -> Result<Self, Self::Error> {
                s.parse()
            }
        }

        impl<'a> std::convert::TryFrom<&'a std::rc::Rc<str>> for #name {
            type Error = anyhow::Error;

            fn try_from(s: &'a std::rc::Rc<str>) -> Result<Self, Self::Error> {
                s.parse()
            }
        }

        impl std::convert::TryFrom<std::sync::Arc<str>> for #name {
            type Error = anyhow::Error;

            fn try_from(s: std::sync::Arc<str>) -> Result<Self, Self::Error> {
                s.parse()
            }
        }

        impl<'a> std::convert::TryFrom<&'a std::sync::Arc<str>> for #name {
            type Error = anyhow::Error;

            fn try_from(s: &'a std::sync::Arc<str>) -> Result<Self, Self::Error> {
                s.parse()
            }
        }

        impl std::convert::TryFrom<std::boxed::Box<str>> for #name {
            type Error = anyhow::Error;

            fn try_from(s: std::boxed::Box<str>) -> Result<Self, Self::Error> {
                s.parse()
            }
        }

        impl<'a> std::convert::TryFrom<&'a std::boxed::Box<str>> for #name {
            type Error = anyhow::Error;

            fn try_from(s: &'a std::boxed::Box<str>) -> Result<Self, Self::Error> {
                s.parse()
            }
        }

        impl std::convert::TryFrom<String> for #name {
            type Error = anyhow::Error;

            fn try_from(s: String) -> Result<Self, Self::Error> {
                s.parse()
            }
        }

        impl<'a> std::convert::TryFrom<&'a String> for #name {
            type Error = anyhow::Error;

            fn try_from(s: &'a String) -> Result<Self, Self::Error> {
                s.parse()
            }
        }

        impl<'a> std::convert::TryFrom<&'a str> for #name {
            type Error = anyhow::Error;

            fn try_from(s: &'a str) -> Result<Self, Self::Error> {
                s.parse()
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
/// - percent
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
/// - percent
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
/// - percent
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

/// Impliments `serde::Serialize` and `serde::Deserialize` for the enum.
#[proc_macro_attribute]
pub fn serde(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        panic!("#[serde] does not take arguments");
    }

    input
}
