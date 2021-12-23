use crate::plugins::Plugin;
use gtk::{
    glib::{self, Boxed},
    prelude::*,
};

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
