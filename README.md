# To, and Fro

Automatic implimentations for `Display` and `FromStr` for things.


<div align="center">
    <a href="https://choosealicense.com/licenses/mit/">
        <img alt="MIT License" src="https://img.shields.io/badge/License-MIT-green.svg">
    </a>
    <a href="https://crates.io/crates/to_and_fro">
      <img alt="Crates.io" src="https://img.shields.io/crates/d/to_and_fro">
    </a>
    <a href="https://github.com/tascord/ToAndFro/actions/runs/6998808999/">
      <img alt="GitHub Workflow Status (with event)" src="https://img.shields.io/github/actions/workflow/status/tascord/toandfro/rust.yml">
    </a>
</div>

### Package available through cargo
```sh
cargo add to_and_fro
```

### Implimentation
```rs
#[derive(ToAndFro)]
pub enum TestEnum {
  ValueOne,
  ValueTwo,
  ValueThree
}

TestEnum::ValueOne.to_string()  // "ValueOne"
TestEnum::from_str("ValueTwo")  //  TestEnum::ValueTwo

TestEnum::from_str("ValueFour") // anyhow::Error("Invalid variant ValueFour for enum TestEnum")
```

### Casing
```rs
#[derive(ToAndFro)]
pub enum TestEnum {
  #[input_case("snake")]        // FromStr will parse only snake_case input
  ValueOne,
  #[output_case("kebab")]       // Display methods will produce a kebab-case output
  ValueTwo,
  ValueThree                    // Defaults to as written input, and as-written output
}
```

## Feedback
I appreciate all feedback, in whatever forms they might take.  
If you're looking to specifically make a [Bug Report](https://github.com/tascord/ToAndFro/issues/new?template=bug_report.md), or [Suggest a Feature](https://github.com/tascord/ToAndFro/issues/new?template=feature_request.md), please do so through their templates in the issues section.
## Related
- [**Synstructure**](https://github.com/mystor/synstructure), a crate that *provides helper types for matching against enum variants, and extracting bindings to each of the fields in the deriving Struct or Enum in a generic way.*
