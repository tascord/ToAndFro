extern crate to_and_fro;

#[cfg(test)]
mod tests {

    use std::str::FromStr;
    use to_and_fro::*;

    #[derive(ToAndFro)]
    #[default("Generation")]
    pub enum TestEnum {
        #[output_name("Gen")]
        Generation,
        #[rename("Load-Test")]
        Load,
        #[alias("People")]
        Customers,
    }

    #[test]
    pub fn output_name() {
        // Just changes the output name, not the input name
        assert_eq!(TestEnum::Generation.to_string(), "Gen");
        assert_eq!(TestEnum::from_str("Generation").unwrap(), TestEnum::Generation);
    }

    #[test]
    pub fn rename() {
        // Changes both
        assert_eq!(TestEnum::from_str("Load-Test").unwrap(), TestEnum::Load);
        assert_eq!(TestEnum::Load.to_string(), "Load-Test");
    }

    #[test]
    pub fn alias() {
        // Aliases input
        assert_eq!(TestEnum::from_str("People").unwrap(), TestEnum::Customers);
        assert_eq!(TestEnum::from_str("Customers").unwrap(), TestEnum::Customers);
        assert_eq!(TestEnum::Customers.to_string(), "Customers");
    }
}
