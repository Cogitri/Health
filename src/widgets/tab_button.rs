/* tab_button.rs
 *
 * Copyright 2021 Visvesh Subramanian <visveshs.blogspot.com>
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

use gtk::glib::{self};

mod imp {
    use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/tab_button.ui")]
    pub struct TabButton {
        #[template_child]
        pub tab_name: TemplateChild<gtk::Label>,
        #[template_child]
        pub icon: TemplateChild<gtk::Image>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TabButton {
        const NAME: &'static str = "HealthTabButton";
        type ParentType = gtk::Widget;
        type Type = super::TabButton;

        fn class_init(klass: &mut Self::Class) {
            klass.set_layout_manager_type::<gtk::BinLayout>();
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl WidgetImpl for TabButton {}
    impl ObjectImpl for TabButton {
        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_string(
                        "tab-name",
                        "tab-name",
                        "tab-name",
                        None,
                        glib::ParamFlags::WRITABLE,
                    ),
                    glib::ParamSpec::new_string(
                        "icon-name",
                        "icon-name",
                        "icon-name",
                        None,
                        glib::ParamFlags::WRITABLE,
                    ),
                ]
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
                "tab-name" => self.tab_name.set_label(value.get::<&str>().unwrap_or("")),
                "icon-name" => self
                    .icon
                    .set_icon_name(Some(value.get::<&str>().unwrap_or(""))),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    /// [TabButton] is a toplevel container that is implemented by all other views of Health.
    pub struct TabButton(ObjectSubclass<imp::TabButton>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl TabButton {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create TabButton")
    }
}
