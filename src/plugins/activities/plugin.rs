use crate::{
    core::i18n,
    plugins::{
        activities::{DataProvider, PluginActivitiesDetails},
        Plugin, PluginDetails,
    },
};
use gtk::prelude::*;

const NAME: &str = "activities";
const ICON_NAME: &str = "walking-thin-symbolic";

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

    fn name(&self) -> &'static str {
        NAME
    }

    fn icon_name(&self) -> &'static str {
        ICON_NAME
    }

    fn localised_name(&self) -> String {
        i18n("Activities")
    }
}
