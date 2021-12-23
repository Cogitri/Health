use crate::{
    core::i18n,
    plugins::{
        steps::{DataProvider, PluginStepsDetails},
        Plugin, PluginDetails, PluginName,
    },
};
use gtk::prelude::*;

#[derive(Clone, Debug)]
pub struct StepsPlugin;

impl StepsPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

impl Plugin for StepsPlugin {
    fn details(&self, mocked: bool) -> PluginDetails {
        let data_provider = if mocked {
            DataProvider::mocked()
        } else {
            DataProvider::actual()
        };

        PluginStepsDetails::new(data_provider).upcast()
    }

    fn name(&self) -> PluginName {
        PluginName::Steps
    }

    fn icon_name(&self) -> &'static str {
        "steps-thin-symbolic"
    }

    fn localised_name(&self) -> String {
        i18n("Steps")
    }
}
