#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
extern crate to_and_fro;
#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use to_and_fro::*;
    #[default_to("Generation")]
    #[input_case("snake")]
    pub enum TestEnum {
        Generation,
        Load,
        Customers,
    }
    impl std::fmt::Debug for TestEnum {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            match self {
                TestEnum::Generation => f.write_fmt(format_args!("Generation")),
                TestEnum::Load => f.write_fmt(format_args!("Load")),
                TestEnum::Customers => f.write_fmt(format_args!("Customers")),
            }
        }
    }
    impl std::cmp::PartialEq for TestEnum {
        fn eq(&self, other: &Self) -> bool {
            std::mem::discriminant(self) == std::mem::discriminant(other)
        }
    }
    impl std::fmt::Display for TestEnum {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            match self {
                TestEnum::Generation => f.write_fmt(format_args!("Generation")),
                TestEnum::Load => f.write_fmt(format_args!("Load")),
                TestEnum::Customers => f.write_fmt(format_args!("Customers")),
            }
        }
    }
    impl std::str::FromStr for TestEnum {
        type Err = anyhow::Error;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "generation" => Ok(TestEnum::Generation),
                "load" => Ok(TestEnum::Load),
                "customers" => Ok(TestEnum::Customers),
                _ => Ok(TestEnum::Generation),
            }
        }
    }
    extern crate test;
    #[cfg(test)]
    #[rustc_test_marker = "tests::test"]
    pub const test: test::TestDescAndFn = test::TestDescAndFn {
        desc: test::TestDesc {
            name: test::StaticTestName("tests::test"),
            ignore: false,
            ignore_message: ::core::option::Option::None,
            source_file: "tests/default.rs",
            start_line: 19usize,
            start_col: 12usize,
            end_line: 19usize,
            end_col: 16usize,
            compile_fail: false,
            no_run: false,
            should_panic: test::ShouldPanic::No,
            test_type: test::TestType::IntegrationTest,
        },
        testfn: test::StaticTestFn(|| test::assert_test_result(test())),
    };
    pub fn test() {
        match (&TestEnum::from_str("Not a variant").unwrap(), &TestEnum::Generation) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    }
}
#[rustc_main]
#[no_coverage]
pub fn main() -> () {
    extern crate test;
    test::test_main_static(&[&test])
}
