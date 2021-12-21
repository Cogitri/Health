use crate::{
    core::i18n,
    plugins::{
        calories::{DataProvider, PluginCaloriesDetails},
        Plugin, PluginDetails,
    },
};
use gtk::prelude::*;

const NAME: &str = "calories";
const ICON_NAME: &str = "calories-thin-symbolic";

#[derive(Clone, Debug)]
pub struct CaloriesPlugin;

impl CaloriesPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

impl Plugin for CaloriesPlugin {
    fn details(&self, mocked: bool) -> PluginDetails {
        let data_provider = if mocked {
            DataProvider::mocked()
        } else {
            DataProvider::actual()
        };

        PluginCaloriesDetails::new(data_provider).upcast()
    }

    fn name(&self) -> &'static str {
        NAME
    }

    fn icon_name(&self) -> &'static str {
        ICON_NAME
    }

    fn localised_name(&self) -> String {
        i18n("Calories")
    }
}
