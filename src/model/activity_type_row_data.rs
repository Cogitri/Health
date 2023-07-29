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

use gtk::{glib, prelude::*};

mod imp {
    use gtk::{glib, prelude::*, subclass::prelude::*};
    use once_cell::unsync::OnceCell;

    #[derive(Debug, Default)]
    pub struct ActivityTypeRowData {
        pub id: OnceCell<String>,
        pub label: OnceCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ActivityTypeRowData {
        const NAME: &'static str = "HealthActivityTypeRowData";
        type ParentType = glib::Object;
        type Type = super::ActivityTypeRowData;
    }

    impl ObjectImpl for ActivityTypeRowData {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecString::builder("id")
                        .construct_only()
                        .readwrite()
                        .build(),
                    glib::ParamSpecString::builder("label")
                        .construct_only()
                        .readwrite()
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "id" => self.id.get().unwrap().to_value(),
                "label" => self.label.get().unwrap().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "id" => self.id.set(value.get().unwrap()).unwrap(),
                "label" => self.label.set(value.get().unwrap()).unwrap(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    /// The data belonging to a certain [ActivityTypeRow](crate::views::ActivityTypeRow).
    /// This can be fed to a [ActivityTypeRow](crate::views::ActivityTypeRow) via
    /// a [gio::ListModel].
    pub struct ActivityTypeRowData(ObjectSubclass<imp::ActivityTypeRowData>);
}

impl ActivityTypeRowData {
    pub fn new(id: &str, label: &str) -> Self {
        glib::Object::builder()
            .property("id", id)
            .property("label", label)
            .build()
    }

    pub fn id(&self) -> String {
        self.property("id")
    }

    pub fn label(&self) -> String {
        self.property("label")
    }
}

#[cfg(test)]
mod test {
    use super::ActivityTypeRowData;

    #[test]
    fn new() {
        ActivityTypeRowData::new("id", "label");
    }

    #[test]
    fn label() {
        let s = ActivityTypeRowData::new("id", "label");
        assert_eq!(s.id(), "id");
        assert_eq!(s.label(), String::from("label"));
    }
}
