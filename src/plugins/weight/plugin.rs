use crate::{
    core::i18n,
    plugins::{
        weight::{DataProvider, PluginWeightDetails},
        Plugin, PluginDetails, PluginName,
    },
};
use gtk::prelude::*;

#[derive(Clone, Debug)]
pub struct WeightPlugin;

impl WeightPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

impl Plugin for WeightPlugin {
    fn details(&self, mocked: bool) -> PluginDetails {
        let data_provider = if mocked {
            DataProvider::mocked()
        } else {
            DataProvider::actual()
        };

        PluginWeightDetails::new(data_provider).upcast()
    }

    fn name(&self) -> PluginName {
        PluginName::Weight
    }

    fn icon_name(&self) -> &'static str {
        "weight-scale-thin-symbolic"
    }

    fn localised_name(&self) -> String {
        i18n("Weight")
    }
}
