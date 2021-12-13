use crate::{
    i18n,
    plugins::{
        steps::{PluginStepsDetails, PluginStepsSummaryRow},
        Plugin, PluginDetails, PluginOverviewRow, PluginSummaryRow,
    },
};
use gtk::{glib, prelude::*};

const NAME: &str = "steps";
const ICON_NAME: &str = "steps-thin-symbolic";

#[derive(Clone, Debug)]
pub struct StepsPlugin {
    details: PluginStepsDetails,
    summary: PluginStepsSummaryRow,
    overview: PluginOverviewRow,
}

impl StepsPlugin {
    pub fn new() -> Self {
        Self {
            details: PluginStepsDetails::new(),
            summary: PluginStepsSummaryRow::new(NAME),
            overview: PluginOverviewRow::new(NAME, ICON_NAME, &i18n("Steps")),
        }
    }
}

impl Plugin for StepsPlugin {
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
