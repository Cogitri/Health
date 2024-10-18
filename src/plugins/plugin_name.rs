use gtk::glib;
use gtk::prelude::*;

#[derive(
    PartialEq,
    Eq,
    Debug,
    Clone,
    Copy,
    strum::EnumString,
    strum::AsRefStr,
    num_derive::ToPrimitive,
    serde::Deserialize,
    serde::Serialize,
)]
#[strum(serialize_all = "snake_case")]
pub enum PluginName {
    Activities,
    Calories,
    Steps,
    Weight,
}

impl ToValue for PluginName {
    fn to_value(&self) -> glib::Value {
        self.as_ref().to_value()
    }

    fn value_type(&self) -> glib::Type {
        <String as StaticType>::static_type()
    }
}
