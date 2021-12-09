static mut REGISTRAR: Option<Registrar> = None;

use crate::plugins::PluginList;
use gtk::{glib, prelude::*, subclass::prelude::*};

mod imp {
    use crate::{
        plugins::{
            ActivitiesPlugin, CaloriesPlugin, Plugin, PluginList, StepsPlugin, WeightPlugin,
        },
        Settings,
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
                if enabled_plugins.contains(&plugin.name().to_string()) {
                    self.enabled_plugins.push(plugin);
                } else {
                    self.disabled_plugins.push(plugin);
                }
            }
        }
    }
}

glib::wrapper! {
    /// An implementation of [View] visualizes streak counts and daily step records.
    pub struct Registrar(ObjectSubclass<imp::Registrar>);
}

impl Registrar {
    pub fn disable_plugin(&self, plugin_name: &str) {
        let self_ = self.imp();
        if !self_.disabled_plugins.contains(plugin_name) {
            self_
                .disabled_plugins
                .push(self_.enabled_plugins.remove(plugin_name).unwrap());

            self.emit_by_name::<()>("plugins-changed", &[]);
        }
    }

    pub fn enable_plugin(&self, plugin_name: &str) {
        let self_ = self.imp();
        if !self_.enabled_plugins.contains(plugin_name) {
            self_
                .enabled_plugins
                .push(self_.disabled_plugins.remove(plugin_name).unwrap());

            self.emit_by_name::<()>("plugins-changed", &[]);
        }
    }

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

    pub fn disabled_plugins(&self) -> PluginList {
        self.imp().disabled_plugins.clone()
    }

    pub fn enabled_plugins(&self) -> PluginList {
        self.imp().enabled_plugins.clone()
    }

    fn imp(&self) -> &imp::Registrar {
        imp::Registrar::from_instance(self)
    }

    fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Registrar")
    }
}
