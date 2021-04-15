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
use glib::subclass::prelude::*;
use gtk::prelude::*;

mod imp {
    use gtk::{prelude::*, subclass::prelude::*, CompositeTemplate};
    use std::cell::RefCell;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/dev/Cogitri/Health/ui/activity_type_row.ui")]
    pub struct ActivityTypeRow {
        pub activity_type_id: RefCell<&'static str>,
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

        fn new() -> Self {
            Self {
                activity_type_id: RefCell::new(""),
                activity_type_label: TemplateChild::default(),
                selected_image: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ActivityTypeRow {}
    impl WidgetImpl for ActivityTypeRow {}
    impl ListBoxRowImpl for ActivityTypeRow {}
}

glib::wrapper! {
    /// An implementation of [gtk::ListBoxRow] that is used in a [ActivityTypeSelector](crate::widgets::ActivityTypeSelector)
    /// and displays information about a single [ActivityType](crate::model::ActivityType).
    pub struct ActivityTypeRow(ObjectSubclass<imp::ActivityTypeRow>)
        @extends gtk::Widget, gtk::ListBoxRow;
}

impl ActivityTypeRow {
    /// Get the ID of the [ActivityType](crate::model::ActivityType)
    pub fn get_id(&self) -> &'static str {
        *self.get_priv().activity_type_id.borrow()
    }

    /// Get the user visible name of the [ActivityType](crate::model::ActivityType)
    pub fn get_label(&self) -> String {
        self.get_priv().activity_type_label.text().to_string()
    }

    /// Get whether or not the row is selected.
    pub fn get_selected(&self) -> bool {
        self.get_priv().selected_image.get_visible()
    }

    /// Create a new [ActivityTypeRow].
    ///
    /// # Arguments
    /// * `data` - The [ActivityTypeRowData] to populate the [ActivityTypeRow] from.
    /// * `selected` - Whether or not the row is elected.
    pub fn new(data: &ActivityTypeRowData, selected: bool) -> Self {
        let s: Self = glib::Object::new(&[]).expect("Failed to create ActivityTypeRow");

        s.set_id(data.get_id());
        s.set_label(&data.get_label());
        s.set_selected(selected);

        s
    }

    /// Set the ID of the [ActivityType](crate::model::ActivityType)
    pub fn set_id(&self, value: &'static str) {
        self.get_priv().activity_type_id.replace(value);
    }

    /// Set the user visible name of the [ActivityType](crate::model::ActivityType)
    pub fn set_label(&self, value: &str) {
        self.get_priv().activity_type_label.set_text(value)
    }

    /// Set whether or not the row is selected.
    pub fn set_selected(&self, value: bool) {
        self.get_priv().selected_image.set_visible(value)
    }

    fn get_priv(&self) -> &imp::ActivityTypeRow {
        imp::ActivityTypeRow::from_instance(self)
    }
}
