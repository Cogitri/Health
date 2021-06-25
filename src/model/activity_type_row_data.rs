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

use gtk::{gio::subclass::prelude::*, glib};

mod imp {
    use gtk::{glib, subclass::prelude::*};
    use std::cell::RefCell;

    #[derive(Debug)]
    pub struct ActivityTypeRowDataMut {
        pub id: &'static str,
        pub label: String,
    }

    #[derive(Debug, Default)]
    pub struct ActivityTypeRowData {
        pub inner: RefCell<Option<ActivityTypeRowDataMut>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ActivityTypeRowData {
        const NAME: &'static str = "HealthActivityTypeRowData";
        type ParentType = glib::Object;
        type Type = super::ActivityTypeRowData;
    }

    impl ObjectImpl for ActivityTypeRowData {}
}

glib::wrapper! {
    /// The data belonging to a certain [ActivityTypeRow](crate::views::ActivityTypeRow).
    /// This can be fed to a [ActivityTypeRow](crate::views::ActivityTypeRow) via
    /// a [gio::ListModel].
    pub struct ActivityTypeRowData(ObjectSubclass<imp::ActivityTypeRowData>);
}

impl ActivityTypeRowData {
    pub fn new(id: &'static str, label: &str) -> Self {
        let o: Self = glib::Object::new(&[]).expect("Failed to create ActivityTypeRowData");

        o.imp().inner.replace(Some(imp::ActivityTypeRowDataMut {
            id,
            label: label.to_string(),
        }));

        o
    }

    pub fn id(&self) -> &'static str {
        self.imp().inner.borrow().as_ref().unwrap().id
    }

    pub fn label(&self) -> String {
        self.imp().inner.borrow().as_ref().unwrap().label.clone()
    }

    fn imp(&self) -> &imp::ActivityTypeRowData {
        imp::ActivityTypeRowData::from_instance(self)
    }
}

#[cfg(test)]
mod test {
    use super::ActivityTypeRowData;

    #[test]
    fn label() {
        let s = ActivityTypeRowData::new("id", "label");
        assert_eq!(s.id(), "id");
        assert_eq!(s.label(), String::from("label"));
    }
}
