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

/// Automatically implement Rust getters and setters for the property `name` of type `type`
#[macro_export]
macro_rules! properties_setter_getter {
    ($name:literal, $type:ty) => {
        paste::item! {
            #[doc = "Get value of property"]
            pub fn [< get_ $name >] (&self) -> Option<$type> {
                self.get_property($name).unwrap().get().unwrap()
            }
        }
        paste::item! {
            #[doc = "Set value of property"]
            pub fn [< set_ $name >] (&self, value: $type) {
                self.set_property($name, &value).unwrap()
            }
        }
    };
}

/// Automatically generate helper functions for connecting to/getting/setting GSettings key
#[macro_export]
macro_rules! settings_getter_setter {
    ($type:ty, $name:ident, $key:literal) => {
        paste::item! {
            #[easy_ext::ext([<HealthSettings $name:camel Ext>])]
            impl gio::Settings {
                #[doc = "Get value of GSettings key"]
                pub fn [< get_ $name >] (&self) -> $type {
                    self.get::<$type>($key)
                }
                #[doc = "Set value of GSettings key"]
                pub fn [< set_ $name >] (&self, value: $type) {
                    self.set::<$type>($key, &value).unwrap();
                }
                #[doc = "Connect to value changes of this key. Keep in mind that the key has to be read once before connecting or this won't do anything!"]
                pub fn [< connect_ $name _changed >]<F: Fn(&gio::Settings, &str) + 'static>(&self, f: F) -> glib::SignalHandlerId {
                    self.connect_changed(Some(stringify!($name)), move |s, name| {
                        f(s, name);
                    })
                }
            }
        }
    };
}

/// Automatically generate generate getters&setters for members of the inner struct (where inner is a `RefCell`).
#[macro_export]
macro_rules! inner_refcell_getter_setter {
    ($name:ident, $type:ty) => {
        paste::item! {
            #[doc = "Borrow `RefCell` and get value"]
            pub fn [< get_ $name >] (&self) -> $type {
                self.inner.borrow().$name.clone()
            }
        }
        paste::item! {
            #[doc = "Mutably borrow `RefCell` and set value."]
            pub fn [< set_ $name >] (&self, value: $type) -> &Self {
                self.inner.borrow_mut().$name = value;
                self
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
            pub fn [< get_ $name >] (&self) -> $type {
                self.get_priv().inner.borrow().$name.clone()
            }
        }
        paste::item! {
            #[doc = "Borrow `RefCell` and set value"]
            pub fn [< set_ $name >] (&self, value: $type) -> &Self {
                self.get_priv().inner.borrow_mut().$name = value;
                self
            }
        }
    };
}
