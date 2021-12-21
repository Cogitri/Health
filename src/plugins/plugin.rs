use crate::plugins::{PluginDetails, PluginOverviewRow, PluginSummaryRow};
use gtk::{
    glib::{self, Boxed},
    prelude::ObjectExt,
};

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
/// 3. Create a `plugin.rs` and implement the [Plugin] trait.
/// 4. Finally, add your plugin to the list in the [Registrar](crate::plugins::Registrar)
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
    fn name(&self) -> &'static str;
    /// The name of the icon that should be used for the overview row
    fn icon_name(&self) -> &'static str;
    /// The localised name of the plugin, that's displayed to the user.
    fn localised_name(&self) -> String;
}

#[derive(Boxed, Clone, Debug)]
#[boxed_type(name = "PluginBoxed")]
pub struct PluginBoxed(pub Box<dyn Plugin>);

mod imp {
    use super::PluginBoxed;
    use gtk::glib::{self, prelude::*, subclass::prelude::*};
    use once_cell::unsync::OnceCell;

    #[derive(Default)]
    pub struct PluginObject {
        pub plugin: OnceCell<PluginBoxed>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PluginObject {
        const NAME: &'static str = "HealthPluginObject";
        type ParentType = glib::Object;
        type Type = super::PluginObject;
    }

    impl ObjectImpl for PluginObject {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecBoxed::new(
                    "plugin",
                    "plugin",
                    "plugin",
                    PluginBoxed::static_type(),
                    glib::ParamFlags::READABLE
                        | glib::ParamFlags::WRITABLE
                        | glib::ParamFlags::CONSTRUCT_ONLY,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "plugin" => self.plugin.set(value.get().unwrap()).unwrap(),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "plugin" => self.plugin.get().unwrap().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    /// A wrapper around a [Plugin] so we can store it in a [PluginList].
    pub struct PluginObject(ObjectSubclass<imp::PluginObject>);
}

impl PluginObject {
    pub fn new(plugin: Box<dyn Plugin>) -> Self {
        glib::Object::new(&[("plugin", &PluginBoxed(plugin))])
            .expect("Failed to create PluginObject")
    }

    pub fn plugin(&self) -> Box<dyn Plugin> {
        self.property::<PluginBoxed>("plugin").0
    }
}

#[cfg(test)]
mod test {
    use super::PluginObject;

    #[test]
    fn new() {
        PluginObject::new(Box::new(crate::plugins::steps::StepsPlugin::new()));
    }
}
