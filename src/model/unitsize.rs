/// A [Unitsize] is so the user can choose to enter different unit sizes (e.g. km vs meter).
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, strum::EnumString, strum::AsRefStr,
)]
#[strum(serialize_all = "snake_case")]
pub enum Unitsize {
    Big,
    Small,
}

impl Default for Unitsize {
    fn default() -> Self {
        Self::Small
    }
}

#[cfg(test)]
mod test {
    use super::Unitsize;
    use std::str::FromStr;

    #[test]
    fn deserialize() {
        assert_eq!(Unitsize::from_str("big").unwrap(), Unitsize::Big);
        assert_eq!(Unitsize::from_str("small").unwrap(), Unitsize::Small);
    }

    #[test]
    fn serialize() {
        let a: &str = Unitsize::Big.into();
        assert_eq!(a, "big");
        let b: &str = Unitsize::Small.into();
        assert_eq!(b, "small");
    }
}
