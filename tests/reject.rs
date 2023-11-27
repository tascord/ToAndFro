extern crate to_and_fro;

#[cfg(test)]
mod tests {

    use std::str::FromStr;
    use to_and_fro::ToAndFro;

    #[derive(ToAndFro)]
    pub enum TestEnum {
        #[reject]
        Generation,
        Load,
        Customers,
    }

    #[test]
    pub fn test() {
        assert!(TestEnum::from_str("Generation").is_err());
        assert!(TestEnum::from_str("Load").is_ok());
        assert!(TestEnum::from_str("Customers").is_ok());
    }
}
