use heck::{
    AsKebabCase, AsLowerCamelCase, AsPascalCase, AsShoutyKebabCase, AsShoutySnakeCase, AsSnakeCase,
    AsTitleCase, AsTrainCase, AsUpperCamelCase,
};
use std::rc::Rc;

pub type Caser = Rc<Box<dyn Fn(&str) -> String + Send + Sync + 'static>>;

pub static CASES: [&str; 11] = [
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

pub fn match_supplied_casing(ident: &str, attrs: &Vec<syn::Attribute>) -> Option<Caser> {
    let casing = attrs
        .iter()
        .find(|attr| attr.path().is_ident(ident))
        .map(|attr| attr.parse_args::<syn::LitStr>().unwrap().value())
        .or(attrs
            .iter()
            .find(|attr| attr.path().is_ident("casing"))
            .map(|attr| attr.parse_args::<syn::LitStr>().unwrap().value()));

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
