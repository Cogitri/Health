use crate::{
    i18n,
    plugins::{
        activities::{PluginActivitiesDetails, PluginActivitiesSummaryRow},
        Plugin, PluginDetails, PluginOverviewRow, PluginSummaryRow,
    },
};
use gtk::{glib, prelude::*};

const NAME: &str = "activities";
const ICON_NAME: &str = "activities-thin-symbolic";

#[derive(Clone, Debug)]
pub struct ActivitiesPlugin {
    details: PluginActivitiesDetails,
    summary: PluginActivitiesSummaryRow,
    overview: PluginOverviewRow,
}

impl ActivitiesPlugin {
    pub fn new() -> Self {
        Self {
            details: PluginActivitiesDetails::new(),
            summary: PluginActivitiesSummaryRow::new(NAME),
            overview: PluginOverviewRow::new(NAME, ICON_NAME, &i18n("Activities")),
        }
    }
}

impl Plugin for ActivitiesPlugin {
    fn summary(&self) -> PluginSummaryRow {
        self.summary.clone().upcast()
    }

    fn overview(&self) -> PluginOverviewRow {
        self.overview.clone()
    }

    fn details(&self) -> PluginDetails {
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

    fn mock(&self) {
        self.details.mock();
    }

    fn unmock(&self) {
        self.details.unmock();
    }
}
