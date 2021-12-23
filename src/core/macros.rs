/* macros.rs
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

/// Automatically generate helper functions for connecting to/getting/setting GSettings key
#[macro_export]
macro_rules! settings_getter_setter {
    ($type:ty, $name:ident, $key:literal) => {
        paste::item! {
            #[doc = "Get value of GSettings key"]
            pub fn [< $name >] (&self) -> $type {
                self.get::<$type>($key)
            }
            #[doc = "Set value of GSettings key"]
            pub fn [< set_ $name >] (&self, value: $type) {
                self.set::<$type>($key, &value).unwrap();
            }
            #[doc = "Connect to value changes of this key. Keep in mind that the key has to be read once before connecting or this won't do anything!"]
            pub fn [< connect_ $name _changed >]<F: Fn(&gtk::gio::Settings, &str) + 'static>(&self, f: F) -> gtk::glib::SignalHandlerId {
                self.connect_changed(Some($key), move |s, name| {
                    f(s, name);
                })
            }
        }
    };
}

/// Automatically generate getters&setters for the private struct's inner `RefCell`
#[macro_export]
macro_rules! refcell_getter_setter {
    ($name:ident, $type:ty) => {
        paste::item! {
            #[doc = "Borrow `RefCell` and get value"]
            pub fn [< $name >] (&self) -> $type {
                self.imp().inner.borrow().$name.clone()
            }
        }
        paste::item! {
            #[doc = "Borrow `RefCell` and set value"]
            pub fn [< set_ $name >] (&self, value: $type) -> &Self {
                self.imp().inner.borrow_mut().$name = value;
                self
            }
        }
    };
}
