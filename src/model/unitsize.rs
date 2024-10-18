use gtk::glib;
use gtk::prelude::*;

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

impl ToValue for Unitsize {
    fn to_value(&self) -> glib::Value {
        self.as_ref().to_value()
    }

    fn value_type(&self) -> glib::Type {
        <String as StaticType>::static_type()
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
        assert_eq!(Unitsize::Big.as_ref(), "big");
        assert_eq!(Unitsize::Small.as_ref(), "small");
    }
}
