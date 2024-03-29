/* view.rs
 *
 * Copyright 2020-2021 Rasmus Thomsen <oss@cogitri.dev>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

use crate::plugins::PluginName;
use gtk::{
    glib::{self, prelude::*},
    subclass::prelude::*,
};
use std::str::FromStr;

mod imp {
    use crate::plugins::PluginName;
    use adw::subclass::prelude::*;
    use gtk::{glib, prelude::*, CompositeTemplate};
    use once_cell::unsync::OnceCell;
    use std::str::FromStr;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/plugins/overview.ui")]
    pub struct PluginOverviewRow {
        #[template_child]
        pub icon: TemplateChild<gtk::Image>,
        pub plugin_name: OnceCell<PluginName>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PluginOverviewRow {
        const NAME: &'static str = "HealthPluginOverviewRow";
        type ParentType = adw::ActionRow;
        type Type = super::PluginOverviewRow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PluginOverviewRow {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecString::builder("icon-name")
                        .construct_only()
                        .readwrite()
                        .build(),
                    glib::ParamSpecString::builder("plugin-name")
                        .construct_only()
                        .readwrite()
                        .build(),
                    glib::ParamSpecObject::builder::<gtk::Image>("icon-widget")
                        .read_only()
                        .build(),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "icon-name" => self.icon.set_icon_name(value.get().unwrap()),
                "plugin-name" => {
                    self.plugin_name
                        .set(PluginName::from_str(value.get::<&str>().unwrap()).unwrap())
                        .unwrap();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "icon-name" => self.icon.icon_name().to_value(),
                "plugin-name" => self.plugin_name.get().unwrap().to_value(),
                "icon-widget" => self.icon.to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for PluginOverviewRow {}
    impl ListBoxRowImpl for PluginOverviewRow {}
    impl PreferencesRowImpl for PluginOverviewRow {}
    impl ActionRowImpl for PluginOverviewRow {}
}

glib::wrapper! {
    /// The [PluginOverviewRow] displays the title of a plugin and its icon so they can enable currently disabled plugins.
    pub struct PluginOverviewRow(ObjectSubclass<imp::PluginOverviewRow>)
    @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PluginOverviewRow {
    pub fn new(plugin_name: PluginName, icon_name: &str, title: &str) -> Self {
        glib::Object::builder()
            .property("icon-name", icon_name)
            .property("title", title)
            .property("plugin-name", &plugin_name)
            .property("activatable", true)
            .build()
    }
}

pub trait PluginOverviewRowExt {
    fn icon_name(&self) -> String;
    fn icon_widget(&self) -> gtk::Image;
    fn plugin_name(&self) -> PluginName;
}

impl<O: IsA<PluginOverviewRow>> PluginOverviewRowExt for O {
    fn icon_name(&self) -> String {
        self.property("icon-name")
    }

    fn icon_widget(&self) -> gtk::Image {
        self.property("icon-widget")
    }

    fn plugin_name(&self) -> PluginName {
        PluginName::from_str(&self.property::<String>("plugin-name")).unwrap()
    }
}

unsafe impl<T: adw::subclass::action_row::ActionRowImpl> IsSubclassable<T> for PluginOverviewRow {
    fn class_init(class: &mut glib::Class<Self>) {
        <adw::ActionRow as IsSubclassable<T>>::class_init(class.upcast_ref_mut());
    }

    fn instance_init(instance: &mut glib::subclass::InitializingObject<T>) {
        <adw::ActionRow as IsSubclassable<T>>::instance_init(instance);
    }
}

#[cfg(test)]
mod test {
    use super::{PluginName, PluginOverviewRow};
    use crate::utils::init_gtk;

    #[gtk::test]
    fn new() {
        init_gtk();
        PluginOverviewRow::new(PluginName::Activities, "", "");
    }
}
