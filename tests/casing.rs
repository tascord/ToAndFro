extern crate to_and_fro;

#[cfg(test)]
mod tests {

    use std::str::FromStr;
    use to_and_fro::*;

    #[derive(ToAndFro)]
    pub enum TestEnum {
        #[input_case("snake")]
        LoremIpsum,
        #[input_case("kebab")]
        DolorSitAmet,
        #[input_case("lower")]
        #[output_case("upper")]
        ConsecteturAdipiscingElit,
    }

    #[test]
    pub fn test() {
        // Input snake case
        assert_eq!(
            TestEnum::from_str("lorem_ipsum").unwrap(),
            TestEnum::LoremIpsum
        );

        // Input kebab case
        assert_eq!(
            TestEnum::from_str("dolor-sit-amet").unwrap(),
            TestEnum::DolorSitAmet,
        );

        // Input lower case
        assert_eq!(
            TestEnum::from_str("consecteturadipiscingelit").unwrap(),
            TestEnum::ConsecteturAdipiscingElit
        );

        // Output default (as written in enum)
        assert_eq!("LoremIpsum", TestEnum::LoremIpsum.to_string());

        // Output upper case
        assert_eq!(
            "CONSECTETURADIPISCINGELIT",
            TestEnum::ConsecteturAdipiscingElit.to_string()
        );
    }

    #[derive(ToAndFro)]
    #[input_case("snake")]
    #[output_case("kebab")]
    pub enum TestEnum2 {
        LoremIpsum,
    }

    #[test]
    pub fn default_casing() {
        
        // Input
        assert_eq!(
            TestEnum2::from_str("lorem_ipsum").unwrap(),
            TestEnum2::LoremIpsum
        );

        // Output
        assert_eq!(
            "lorem-ipsum",
            TestEnum2::LoremIpsum.to_string()
        );

    }
}
