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

use crate::properties_setter_getter;
use gtk::{
    glib::{self, prelude::*},
    subclass::prelude::*,
};

mod imp {
    use adw::subclass::prelude::*;
    use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/plugins/overview.ui")]
    pub struct PluginOverviewRow {
        #[template_child]
        pub icon: TemplateChild<gtk::Image>,
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
                vec![glib::ParamSpecString::new(
                    "icon-name",
                    "icon-name",
                    "icon-name",
                    None,
                    glib::ParamFlags::READWRITE,
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
                "icon-name" => self.icon.set_icon_name(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "icon-name" => self.icon.icon_name().to_value(),
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
    /// [PluginOverviewRow] is a toplevel container that is implemented by all other views of Health.
    pub struct PluginOverviewRow(ObjectSubclass<imp::PluginOverviewRow>)
    @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PluginOverviewRow {
    pub fn new(icon_name: &str, title: &str) -> Self {
        glib::Object::new(&[("icon-name", &icon_name), ("title", &title)])
            .expect("Failed to create PluginOverviewRow")
    }

    properties_setter_getter!("icon-name", String);
}

unsafe impl<T: adw::subclass::action_row::ActionRowImpl> IsSubclassable<T> for PluginOverviewRow {
    fn class_init(class: &mut glib::Class<Self>) {
        <adw::ActionRow as IsSubclassable<T>>::class_init(class.upcast_ref_mut());
    }

    fn instance_init(instance: &mut glib::subclass::InitializingObject<T>) {
        <adw::ActionRow as IsSubclassable<T>>::instance_init(instance);
    }
}
