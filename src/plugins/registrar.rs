static mut REGISTRAR: Option<Registrar> = None;

use crate::{
    core::{Database, Settings},
    plugins::{
        ActivitiesPlugin, CaloriesPlugin, Plugin, PluginList, PluginName, StepsPlugin, WeightPlugin,
    },
};
use gtk::{glib, prelude::*, subclass::prelude::*};

mod imp {
    use crate::plugins::PluginList;
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
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("plugins-changed").build()]);

            SIGNALS.as_ref()
        }

        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            gtk_macros::spawn!(glib::clone!(
                #[weak]
                obj,
                async move {
                    obj.load_plugins().await;
                }
            ));
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

    /// Connect to the `plugins-changed` signal.
    ///
    /// # Arguments
    /// * `callback` - The callback which should be invoked when `plugins-changed` is emitted.
    ///
    /// # Returns
    /// A [glib::SignalHandlerId] that can be used for disconnecting the signal if so desired.
    pub fn connect_plugins_updated<F: Fn(&Self) + 'static>(
        &self,
        callback: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("plugins-changed", false, move |values| {
            callback(&values[0].get().unwrap());
            None
        })
    }

    /// Push enabled plugins and disabled plugins.
    pub async fn load_plugins(&self) {
        let imp = self.imp();
        let user_id = i64::from(Settings::instance().active_user_id());
        let user = &Database::instance().user(user_id).await.unwrap();
        let enabled_plugins = user.enabled_plugins().unwrap();
        for plugin in [
            Box::new(ActivitiesPlugin::new()) as Box<dyn Plugin>,
            Box::new(CaloriesPlugin::new()) as Box<dyn Plugin>,
            Box::new(StepsPlugin::new()) as Box<dyn Plugin>,
            Box::new(WeightPlugin::new()) as Box<dyn Plugin>,
        ] {
            if enabled_plugins.contains(&plugin.name()) {
                imp.enabled_plugins.push(plugin);
            } else {
                imp.disabled_plugins.push(plugin);
            }
        }

        self.emit_by_name::<()>("plugins-changed", &[]);
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
        glib::Object::new()
    }
}

#[cfg(test)]
mod test {
    use super::Registrar;

    #[gtk::test]
    fn new() {
        Registrar::new();
    }
}
