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
            pub fn [< connect_ $name _changed >]<F: Fn(&Settings, &str) + 'static>(&self, f: F) -> glib::SignalHandlerId {
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
