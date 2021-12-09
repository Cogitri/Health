use crate::{
    i18n,
    plugins::{
        calories::{PluginCaloriesDetails, PluginCaloriesSummaryRow},
        Plugin, PluginOverviewRow, PluginSummaryRow,
    },
};
use gtk::{glib, prelude::*};

const NAME: &str = "calories";
const ICON_NAME: &str = "calories-thin-symbolic";

#[derive(Clone, Debug)]
pub struct CaloriesPlugin {
    details: PluginCaloriesDetails,
    summary: PluginCaloriesSummaryRow,
    overview: PluginOverviewRow,
}

impl CaloriesPlugin {
    pub fn new() -> Self {
        Self {
            details: PluginCaloriesDetails::new(),
            summary: PluginCaloriesSummaryRow::new(NAME),
            overview: PluginOverviewRow::new(ICON_NAME, &i18n("Calories")),
        }
    }
}

impl Plugin for CaloriesPlugin {
    fn summary(&self) -> PluginSummaryRow {
        self.summary.clone().upcast()
    }

    fn overview(&self) -> adw::ActionRow {
        self.overview.clone().upcast()
    }

    fn details(&self) -> gtk::Widget {
        self.details.clone().upcast()
    }

    fn name(&self) -> &'static str {
        NAME
    }

    fn icon_name(&self) -> &'static str {
        ICON_NAME
    }

    fn update(&self) {
        gtk_macros::spawn!(glib::clone!(@strong self as obj => async move {
            obj.details.update().await;
            obj.summary.update().await;
        }));
    }
}
