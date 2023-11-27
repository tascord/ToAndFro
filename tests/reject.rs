extern crate to_and_fro;

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use to_and_fro::ToAndFro;

    #[derive(ToAndFro)]
    #[allow(dead_code)]
    pub enum TestEnum {
        #[reject]
        Generation,
        Load,
        Customers,
    }

    #[test]
    pub fn to_string_unaffected() {
        assert_eq!(TestEnum::Generation.to_string(), "Generation");
        assert_eq!(TestEnum::Load.to_string(), "Load");
        assert_eq!(TestEnum::Customers.to_string(), "Customers");
    }

    #[test]
    pub fn reject() {
        assert!(TestEnum::from_str("Generation").is_err());
        assert!(TestEnum::from_str("Load").is_ok());
        assert!(TestEnum::from_str("Customers").is_ok());
    }

    #[derive(ToAndFro)]
    #[allow(dead_code)]
    #[default_to("Load")]
    pub enum TestEnum2 {
        #[reject]
        Generation,
        Load,
        Customers,
    }

    #[test]
    pub fn reject_to_default() {
        assert_eq!(TestEnum2::from_str("Generation").unwrap(), TestEnum2::Load);
        assert_eq!(TestEnum2::from_str("Load").unwrap(), TestEnum2::Load);
        assert_eq!(
            TestEnum2::from_str("Customers").unwrap(),
            TestEnum2::Customers
        );
    }
}
