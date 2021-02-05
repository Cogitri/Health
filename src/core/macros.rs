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

#[macro_export]
macro_rules! properties_setter_getter {
    ($name:literal, $type:ty) => {
        paste::item! {
            pub fn [< get_ $name >] (&self) -> Option<$type> {
                self.get_property($name).unwrap().get().unwrap()
            }
        }
        paste::item! {
            pub fn [< set_ $name >] (&self, value: $type) {
                self.set_property($name, &value).unwrap()
            }
        }
    };
}

#[macro_export]
macro_rules! settings_getter_setter {
    ($type:ty, $name:ident, $key:literal) => {
        paste::item! {
            pub fn [< get_ $name >] (&self) -> $type {
                self.settings.get::<$type>($key)
            }
        }
        paste::item! {
            pub fn [< set_ $name >] (&self, value: $type) {
                self.settings.set::<$type>($key, &value).unwrap();
            }
        }
        paste::item! {
            pub fn [< connect_ $name _changed >]<F: Fn(&gio::Settings, &str) + 'static>(&self, f: F) -> glib::SignalHandlerId {
                self.settings.connect_changed(move |s, name| {
                    if name == stringify!($name) {
                        f(s, name);
                    }
                })
            }
        }
    };
}

#[macro_export]
macro_rules! inner_refcell_getter_setter {
    ($name:ident, $type:ty) => {
        paste::item! {
            pub fn [< get_ $name >] (&self) -> $type {
                self.inner.borrow().$name.clone()
            }
        }
        paste::item! {
            pub fn [< set_ $name >] (&self, value: $type) -> &Self {
                self.inner.borrow_mut().$name = value;
                self
            }
        }
    };
}

#[macro_export]
macro_rules! imp_getter_setter {
    ($name:ident, $type:ty) => {
        paste::item! {
            pub fn [< get_ $name >] (&self) -> $type {
                self.get_priv().[< get_ $name >]()
            }
        }
        paste::item! {
            pub fn [< set_ $name >] (&self, value: $type) -> &Self {
                self.get_priv().[< set_ $name >](value);
                self
            }
        }
    };
}
