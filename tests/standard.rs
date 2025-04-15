extern crate to_and_fro;

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, str::FromStr};
    use to_and_fro::ToAndFro;

    #[derive(ToAndFro, Debug)]
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

    #[test]
    pub fn fail_from_str() {
        assert!(TestEnum::from_str("Not a variant").is_err())
    }

    #[test]
    fn test_hash() {
        let mut set = HashSet::new();

        // ensure hashes are disjoint
        for i in TestEnum::list() {
            set.insert(i);
        }
        assert_eq!(set.len(), TestEnum::list().len());

        // second round of adding shouldn't change anything
        for i in TestEnum::list() {
            set.insert(i);
        }
        assert_eq!(set.len(), TestEnum::list().len());
    }

    #[test]
    pub fn test_eq() {
        fn ensure_eq<T: Eq>(a: T, b: T) -> bool {
            a == b
        }

        for i in TestEnum::list() {
            assert!(ensure_eq(i, i));
        }
    }
}
