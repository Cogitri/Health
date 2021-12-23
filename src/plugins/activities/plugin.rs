use crate::{
    core::i18n,
    plugins::{
        activities::{DataProvider, PluginActivitiesDetails},
        Plugin, PluginDetails, PluginName,
    },
};
use gtk::prelude::*;

#[derive(Clone, Debug)]
pub struct ActivitiesPlugin;

impl ActivitiesPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

impl Plugin for ActivitiesPlugin {
    fn details(&self, mocked: bool) -> PluginDetails {
        let data_provider = if mocked {
            DataProvider::mocked()
        } else {
            DataProvider::actual()
        };

        PluginActivitiesDetails::new(data_provider).upcast()
    }

    fn name(&self) -> PluginName {
        PluginName::Activities
    }

    fn icon_name(&self) -> &'static str {
        "walking-thin-symbolic"
    }

    fn localised_name(&self) -> String {
        i18n("Activities")
    }
}
