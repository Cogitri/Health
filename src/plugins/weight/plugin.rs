use crate::{
    i18n,
    plugins::{
        weight::{PluginWeightDetails, PluginWeightSummaryRow},
        Plugin, PluginOverviewRow, PluginSummaryRow,
    },
};
use gtk::{glib, prelude::*};

const NAME: &str = "weight";
const ICON_NAME: &str = "weight-thin-symbolic";

#[derive(Clone, Debug)]
pub struct WeightPlugin {
    details: PluginWeightDetails,
    summary: PluginWeightSummaryRow,
    overview: PluginOverviewRow,
}

impl WeightPlugin {
    pub fn new() -> Self {
        Self {
            details: PluginWeightDetails::new(),
            summary: PluginWeightSummaryRow::new(NAME),
            overview: PluginOverviewRow::new(ICON_NAME, &i18n("Weight")),
        }
    }
}

impl Plugin for WeightPlugin {
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
