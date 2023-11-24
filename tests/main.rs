extern crate to_and_fro;

#[cfg(test)]
mod tests {

    use std::str::FromStr;
    use to_and_fro::ToAndFro;

    #[derive(ToAndFro)]
    pub enum TestEnum {
        Generation,
        Load,
        Customers,
    }

    #[test]
    pub fn test_display() {
        assert_eq!(format!("{}", TestEnum::Generation), "Generation")
    }

    #[test]
    pub fn test_from_str() {
        assert_eq!(
            TestEnum::from_str("Generation").unwrap(),
            TestEnum::Generation
        )
    }
}
