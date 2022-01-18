/* sync_list_box.rs
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

use gtk::glib::{self, subclass::prelude::*, SignalHandlerId};
use gtk::prelude::*;

mod imp {
    use adw::subclass::prelude::*;
    use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/password_entry.ui")]
    pub struct PasswordEntry {
        #[template_child]
        pub password_entry: TemplateChild<gtk::PasswordEntry>,
        #[template_child]
        pub password_repeat_entry: TemplateChild<gtk::PasswordEntry>,
        #[template_child]
        pub password_repeat_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub password_strength_bar: TemplateChild<gtk::LevelBar>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PasswordEntry {
        const NAME: &'static str = "HealthPasswordEntry";
        type ParentType = adw::Bin;
        type Type = super::PasswordEntry;

        fn class_init(klass: &mut Self::Class) {
            klass.set_layout_manager_type::<gtk::BinLayout>();
            Self::bind_template(klass);
            Self::Type::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PasswordEntry {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            self.password_strength_bar
                .add_offset_value(&gtk::LEVEL_BAR_OFFSET_LOW, 1.0);
            self.password_strength_bar
                .add_offset_value(&gtk::LEVEL_BAR_OFFSET_HIGH, 3.0);
            self.password_strength_bar
                .add_offset_value(&gtk::LEVEL_BAR_OFFSET_FULL, 4.0);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecString::new(
                        "password",
                        "password",
                        "password",
                        None,
                        glib::ParamFlags::READABLE,
                    ),
                    glib::ParamSpecBoolean::new(
                        "show-password-repeat",
                        "show-password-repeat",
                        "show-password-repeat",
                        true,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecBoolean::new(
                        "show-password-strength",
                        "show-password-strength",
                        "show-password-strength",
                        true,
                        glib::ParamFlags::READWRITE,
                    ),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "password" => {
                    let entry_text = self.password_entry.text();
                    let entry_text_repeated = self.password_repeat_entry.text();
                    if entry_text.is_empty()
                        || (obj.show_password_repeat() && entry_text != entry_text_repeated)
                    {
                        const S: Option<String> = None;
                        S.to_value()
                    } else {
                        Some(entry_text).to_value()
                    }
                }
                "show-password-repeat" => self.password_repeat_entry.is_visible().to_value(),
                "show-password-strength" => self.password_strength_bar.is_visible().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "show-password-repeat" => {
                    let val = value.get().unwrap();
                    self.password_repeat_entry.set_visible(val);
                    self.password_repeat_label.set_visible(val);
                }
                "show-password-strength" => {
                    self.password_strength_bar.set_visible(value.get().unwrap())
                }
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for PasswordEntry {}
    impl BinImpl for PasswordEntry {}
}

glib::wrapper! {
    /// The [PasswordEntry] is a [adw::Bin] where users enter passwords.
    pub struct PasswordEntry(ObjectSubclass<imp::PasswordEntry>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

#[gtk::template_callbacks]
impl PasswordEntry {
    /// Create a new [PasswordEntry]
    pub fn new(show_password_repeat: bool, show_password_strength: bool) -> Self {
        glib::Object::new(&[
            ("show-password-repeat", &show_password_repeat),
            ("show-password-strength", &show_password_strength),
        ])
        .expect("Failed to create PasswordEntry")
    }

    /// Connect to the entered password changing.
    ///
    /// # Arguments
    /// * `callback` - The callback to call once the ::notify signal is emitted.
    ///
    /// # Returns
    /// The [glib::SignalHandlerId] to disconnect the signal later on.
    pub fn connect_password_notify<F: Fn(&Self) + 'static>(&self, f: F) -> SignalHandlerId {
        self.connect_notify_local(Some("password"), move |s, _| f(s))
    }

    /// Get the currently entered password, or `None` if the passwords entered dont match or are empty.
    pub fn password(&self) -> Option<String> {
        self.property::<Option<String>>("password")
    }

    pub fn set_show_password_repeat(&self, value: bool) {
        self.set_property("show-password-repeat", value);
    }

    pub fn set_show_password_strength(&self, value: bool) {
        self.set_property("show-password-strength", value);
    }

    pub fn show_password_repeat(&self) -> bool {
        self.property::<bool>("show-password-repeat")
    }

    pub fn show_password_strength(&self) -> bool {
        self.property::<bool>("show-password-strength")
    }

    fn calculate_password_strength(&self) {
        let imp = self.imp();
        let level_bar = &imp.password_strength_bar;
        let password = imp.password_entry.text();
        match zxcvbn::zxcvbn(password.as_str(), &[]) {
            Ok(e) => level_bar.set_value(e.score().into()),
            Err(_) => level_bar.set_value(0.0),
        }
    }

    #[template_callback]
    fn handle_password_entry_changed(&self) {
        if self.show_password_strength() {
            self.calculate_password_strength();
        }
        self.notify("password");
    }

    #[template_callback]
    fn handle_password_repeat_entry_changed(&self) {
        self.notify("password");
    }
}

#[cfg(test)]
mod test {
    use super::PasswordEntry;
    use crate::utils::init_gtk;

    #[test]
    fn new() {
        init_gtk();
        PasswordEntry::new(false, false);
    }
}
