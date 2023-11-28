extern crate to_and_fro;

#[cfg(test)]
mod tests {

    use std::str::FromStr;
    use to_and_fro::*;

    #[derive(ToAndFro)]
    #[default("Generation")]
    pub enum TestEnum {
        Generation,
        Load,
        Customers,
    }

    #[test]
    pub fn default_on_fallback() {
        assert_eq!(
            TestEnum::from_str("Not a variant").unwrap(),
            TestEnum::Generation
        )
    }

    #[test]
    pub fn default_impl() {
        assert_eq!(TestEnum::default(), TestEnum::Generation)
    }
}
