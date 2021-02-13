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

use glib::subclass::types::ObjectSubclass;

mod imp {
    use glib::subclass;
    use gtk::subclass::prelude::*;
    use std::cell::RefCell;

    #[derive(Debug)]
    pub struct ActivityTypeRowDataMut {
        pub id: &'static str,
        pub label: String,
    }

    #[derive(Debug)]
    pub struct ActivityTypeRowData {
        pub inner: RefCell<Option<ActivityTypeRowDataMut>>,
    }

    impl ObjectSubclass for ActivityTypeRowData {
        const NAME: &'static str = "HealthActivityTypeRowData";
        type ParentType = glib::Object;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::ActivityTypeRowData;
        type Interfaces = ();

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                inner: RefCell::new(None),
            }
        }
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

        o.get_priv()
            .inner
            .replace(Some(imp::ActivityTypeRowDataMut {
                id,
                label: label.to_string(),
            }));

        o
    }

    pub fn get_id(&self) -> &'static str {
        self.get_priv().inner.borrow().as_ref().unwrap().id
    }

    pub fn get_label(&self) -> String {
        self.get_priv()
            .inner
            .borrow()
            .as_ref()
            .unwrap()
            .label
            .clone()
    }

    fn get_priv(&self) -> &imp::ActivityTypeRowData {
        imp::ActivityTypeRowData::from_instance(self)
    }
}
