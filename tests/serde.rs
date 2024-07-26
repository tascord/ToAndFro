extern crate to_and_fro;

#[cfg(test)]
mod tests {

    use serde::{Deserialize, Serialize};
    use std::str::FromStr;
    use to_and_fro::*;

    #[derive(ToAndFro, Debug)]
    #[casing("kebab")]
    #[serde]
    pub enum TestEnum {
        HelloWorld,
        FooBar,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Container {
        value: TestEnum,
    }

    #[test]
    pub fn serde_kebab() {
        let v = serde_json::to_string(&Container {
            value: TestEnum::HelloWorld,
        })
        .unwrap();
        assert_eq!(v, r#"{"value":"hello-world"}"#)
    }

    #[test]
    pub fn deserialize_kebab() {
        let v: Container = serde_json::from_str(r#"{"value":"foo-bar"}"#).unwrap();
        assert_eq!(v.value, TestEnum::FooBar)
    }
}
