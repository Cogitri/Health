use gtk::glib;
use gtk::prelude::*;

/// A [WeightChange] expresses how the weight of the user has changed
#[derive(
    Debug,
    Default,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    strum::EnumString,
    strum::AsRefStr,
)]
#[strum(serialize_all = "snake_case")]
pub enum WeightChange {
    Up,
    Down,
    #[default]
    NoChange,
}

impl ToValue for WeightChange {
    fn to_value(&self) -> glib::Value {
        self.as_ref().to_value()
    }

    fn value_type(&self) -> glib::Type {
        <String as StaticType>::static_type()
    }
}
