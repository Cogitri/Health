/* activity_type_row.rs
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

use crate::model::ActivityTypeRowData;
use gtk::{
    glib::{self},
    prelude::*,
};

mod imp {
    use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};
    use once_cell::unsync::OnceCell;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/activity_type_row.ui")]
    pub struct ActivityTypeRow {
        pub activity_type_id: OnceCell<String>,
        #[template_child]
        pub activity_type_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub selected_image: TemplateChild<gtk::Image>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ActivityTypeRow {
        const NAME: &'static str = "HealthActivityTypeRow";
        type ParentType = gtk::ListBoxRow;
        type Type = super::ActivityTypeRow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ActivityTypeRow {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecString::builder("id")
                        .construct_only()
                        .readwrite()
                        .build(),
                    glib::ParamSpecString::builder("label")
                        .construct()
                        .readwrite()
                        .build(),
                    glib::ParamSpecBoolean::builder("selected")
                        .construct()
                        .readwrite()
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "id" => self.activity_type_id.get().unwrap().to_value(),
                "label" => self.activity_type_label.label().to_string().to_value(),
                "selected" => self.selected_image.is_visible().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "id" => self.activity_type_id.set(value.get().unwrap()).unwrap(),
                "label" => self.activity_type_label.set_label(value.get().unwrap()),
                "selected" => self.selected_image.set_visible(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for ActivityTypeRow {}
    impl ListBoxRowImpl for ActivityTypeRow {}
}

glib::wrapper! {
    /// An implementation of [gtk::ListBoxRow] that is used in a [ActivityTypeSelector](crate::widgets::ActivityTypeSelector)
    /// and displays information about a single [ActivityType](crate::model::ActivityType).
    pub struct ActivityTypeRow(ObjectSubclass<imp::ActivityTypeRow>)
        @extends gtk::Widget, gtk::ListBoxRow,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl ActivityTypeRow {
    /// Get the ID of the [ActivityType](crate::model::ActivityType)
    pub fn id(&self) -> String {
        self.property("id")
    }

    /// Get the user visible name of the [ActivityType](crate::model::ActivityType)
    pub fn label(&self) -> String {
        self.property("label")
    }

    /// Get whether or not the row is selected.
    pub fn selected(&self) -> bool {
        self.property("selected")
    }

    /// Create a new [ActivityTypeRow].
    ///
    /// # Arguments
    /// * `data` - The [ActivityTypeRowData] to populate the [ActivityTypeRow] from.
    /// * `selected` - Whether or not the row is elected.
    pub fn new(data: &ActivityTypeRowData, selected: bool) -> Self {
        glib::Object::builder()
            .property("id", &data.id())
            .property("label", &data.label())
            .property("selected", selected)
            .build()
    }

    pub fn set_selected(&self, selected: bool) {
        self.set_property("selected", selected);
    }
}

#[cfg(test)]
mod test {
    use super::{ActivityTypeRow, ActivityTypeRowData};
    use crate::utils::init_gtk;

    #[gtk::test]
    fn new() {
        init_gtk();
        ActivityTypeRow::new(&ActivityTypeRowData::new("", ""), false);
    }
}
