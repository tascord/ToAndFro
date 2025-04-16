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
        #[casing("percent")]
        SedDoEiusmod,
    }

    #[test]
    pub fn basic_casing() {
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

        // Percent encoding
        assert_eq!(
            TestEnum::from_str(
                &percent_encoding::utf8_percent_encode(
                    "SedDoEiusmod",
                    percent_encoding::NON_ALPHANUMERIC
                )
                .to_string()
            )
            .unwrap(),
            TestEnum::SedDoEiusmod
        );
    }

    #[derive(ToAndFro)]
    #[input_case("snake")]
    #[output_case("kebab")]
    pub enum TestEnum2 {
        LoremIpsum,
    }

    #[test]
    pub fn casing_with_defaults() {
        // Input
        assert_eq!(
            TestEnum2::from_str("lorem_ipsum").unwrap(),
            TestEnum2::LoremIpsum
        );

        // Output
        assert_eq!("lorem-ipsum", TestEnum2::LoremIpsum.to_string());
        assert_eq!(
            "TestEnum2::LoremIpsum",
            format!("{:?}", TestEnum2::LoremIpsum)
        );
    }
}
