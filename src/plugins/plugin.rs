use crate::plugins::{PluginDetails, PluginName, PluginOverviewRow, PluginSummaryRow};

/// The [Plugin] trait should be implemented for additional data sources of Health.
///
/// The trait automatically provides an implementation of `Plugin::overview()`, returning a [PluginOverviewRow] containing
/// the localised name and the icon of the plugin. The user may click on this row to access the (mocked) [PluginDetails]
/// page to see what the plugin does and to enable it.
/// Once the plugin is enabled, the [PluginSummaryRow] returned by `Plugin::summary()` is shown to the user in the list
/// of enabled plugins, giving the user a quick glance over the data (e.g. "Today you've done X of Y steps"). Upon clicking
/// this [PluginSummaryRow], the user is taken to the (unmocked) [PluginDetails] page.
///
/// To create a new plugin, you should do the following things:
/// 1. Create a new subfolder in `src/plugins/$plugin_name`
/// 2. Create a `details.rs` and `summary.rs`, containing the subclass of [PluginDetails] and [PluginSummaryRow] respectively.
/// 3. Add the name of your plugin to [PluginName] (and fix compile errors from non-exhaustive match arms).
/// 4. Create a `plugin.rs` and implement the [Plugin] trait.
/// 5. Finally, add your plugin to the list in the [Registrar](crate::plugins::Registrar)
#[dyn_clonable::clonable]
pub trait Plugin: Clone + std::fmt::Debug {
    /// Returns a card view with a short overview of the data, e.g 2000/10000 steps done for the home page
    fn summary(&self) -> PluginSummaryRow {
        PluginSummaryRow::from(self.name())
    }

    /// Returns an entry for the "browse all" listbox.
    fn overview(&self) -> PluginOverviewRow {
        PluginOverviewRow::new(self.name(), self.icon_name(), &self.localised_name())
    }

    /// Returns a card view containing details,e.g. steps over some weeks. May be mocked via the `is-mocked` property.
    fn details(&self, mocked: bool) -> PluginDetails;

    /// The non-localised name of the plugin, used as ID. !Must! be unique across plugins
    fn name(&self) -> PluginName;
    /// The name of the icon that should be used for the overview row
    fn icon_name(&self) -> &'static str;
    /// The localised name of the plugin, that's displayed to the user.
    fn localised_name(&self) -> String;
}
