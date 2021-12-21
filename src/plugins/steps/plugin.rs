use crate::{
    core::i18n,
    plugins::{
        steps::{DataProvider, PluginStepsDetails},
        Plugin, PluginDetails,
    },
};
use gtk::prelude::*;

const NAME: &str = "steps";
const ICON_NAME: &str = "steps-thin-symbolic";

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

    fn name(&self) -> &'static str {
        NAME
    }

    fn icon_name(&self) -> &'static str {
        ICON_NAME
    }

    fn localised_name(&self) -> String {
        i18n("Steps")
    }
}
