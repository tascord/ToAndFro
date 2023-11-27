extern crate to_and_fro;

#[cfg(test)]
mod tests {

    use std::str::FromStr;
    use to_and_fro::*;

    #[derive(ToAndFro)]
    #[default_to("Generation")]
    #[input_case("snake")]
    pub enum TestEnum {
        Generation,
        Load,
        Customers,
    }

    #[test]
    pub fn test() {
        assert_eq!(
            TestEnum::from_str("Not a variant").unwrap(),
            TestEnum::Generation
        )
    }

}