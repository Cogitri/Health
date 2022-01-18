static mut REGISTRAR: Option<Registrar> = None;

use crate::plugins::{Plugin, PluginList, PluginName};
use gtk::{glib, prelude::*, subclass::prelude::*};

mod imp {
    use crate::{
        core::Settings,
        plugins::{
            ActivitiesPlugin, CaloriesPlugin, Plugin, PluginList, StepsPlugin, WeightPlugin,
        },
    };
    use gtk::glib::{
        self,
        subclass::{prelude::*, Signal},
    };

    #[derive(Default)]
    pub struct Registrar {
        pub enabled_plugins: PluginList,
        pub disabled_plugins: PluginList,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Registrar {
        const NAME: &'static str = "HealthRegistrar";
        type Type = super::Registrar;
    }

    impl ObjectImpl for Registrar {
        fn signals() -> &'static [glib::subclass::Signal] {
            use once_cell::sync::Lazy;
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("plugins-changed", &[], glib::Type::UNIT.into()).build()]
            });

            SIGNALS.as_ref()
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let enabled_plugins = Settings::instance().enabled_plugins();
            for plugin in [
                Box::new(ActivitiesPlugin::new()) as Box<dyn Plugin>,
                Box::new(CaloriesPlugin::new()) as Box<dyn Plugin>,
                Box::new(StepsPlugin::new()) as Box<dyn Plugin>,
                Box::new(WeightPlugin::new()) as Box<dyn Plugin>,
            ] {
                if enabled_plugins.contains(&plugin.name()) {
                    self.enabled_plugins.push(plugin);
                } else {
                    self.disabled_plugins.push(plugin);
                }
            }
        }
    }
}

glib::wrapper! {
    /// The Registrar is a static class, holding information about enabled and disabled plugins.
    pub struct Registrar(ObjectSubclass<imp::Registrar>);
}

impl Registrar {
    /// Disable a currently enabled [Plugin], if it's currently enabled.
    ///
    /// This emits `plugin-changed` if the list of plugins was changed.
    pub fn disable_plugin(&self, plugin_name: PluginName) {
        let imp = self.imp();
        if !imp.disabled_plugins.contains(plugin_name) {
            let plugin = imp.enabled_plugins.remove(plugin_name).unwrap();
            imp.disabled_plugins.push(plugin);

            self.emit_by_name::<()>("plugins-changed", &[]);
        }
    }

    /// Enable a currently disabled [Plugin], if it's current disabled.
    ///
    /// This emits `plugin-changed` if the list of plugins was changed.
    pub fn enable_plugin(&self, plugin_name: PluginName) {
        let imp = self.imp();
        if !imp.enabled_plugins.contains(plugin_name) {
            let plugin = imp.disabled_plugins.remove(plugin_name).unwrap();
            imp.enabled_plugins.push(plugin);

            self.emit_by_name::<()>("plugins-changed", &[]);
        }
    }

    /// Get a list of disabled [Plugin]s
    pub fn disabled_plugins(&self) -> PluginList {
        self.imp().disabled_plugins.clone()
    }

    /// Get a list of enabled [Plugin]s
    pub fn enabled_plugins(&self) -> PluginList {
        self.imp().enabled_plugins.clone()
    }

    /// Get a plugin from the list of disabled [Plugin]s by the plugin's unlocalised name (ID)
    pub fn disabled_plugin_by_name(&self, name: PluginName) -> Option<Box<dyn Plugin>> {
        self.imp()
            .disabled_plugins
            .iter()
            .find(|x| x.name() == name)
            .map(|o| o.clone())
    }

    /// Get a plugin from the list of enabled [Plugin]s by the plugin's unlocalised name (ID)
    pub fn enabled_plugin_by_name(&self, name: PluginName) -> Option<Box<dyn Plugin>> {
        self.imp()
            .enabled_plugins
            .iter()
            .find(|x| x.name() == name)
            .map(|o| o.clone())
    }

    /// Get an instance of the [Registrar]
    pub fn instance() -> Self {
        unsafe {
            REGISTRAR.as_ref().map_or_else(
                || {
                    let reg = Self::new();
                    REGISTRAR = Some(reg.clone());
                    reg
                },
                std::clone::Clone::clone,
            )
        }
    }

    fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Registrar")
    }
}

#[cfg(test)]
mod test {
    use super::Registrar;

    #[test]
    fn new() {
        Registrar::new();
    }
}
