use crate::{
    core::i18n,
    plugins::{
        weight::{DataProvider, PluginWeightDetails},
        Plugin, PluginDetails,
    },
};
use gtk::prelude::*;

const NAME: &str = "weight";
const ICON_NAME: &str = "weight-scale-thin-symbolic";

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

    fn name(&self) -> &'static str {
        NAME
    }

    fn icon_name(&self) -> &'static str {
        ICON_NAME
    }

    fn localised_name(&self) -> String {
        i18n("Weight")
    }
}
