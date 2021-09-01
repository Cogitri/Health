/* color_circle.rs
 *
 * Copyright 2021 Visvesh Subramanian <visveshs.blogspot.com>
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

use crate::core::{i18n, UnitKind, UnitSystem};
use gtk::{
    glib::{
        subclass::prelude::*,
        {self, clone},
    },
    prelude::*,
};

mod imp {
    use crate::core::{Settings, UnitKind, UnitSystem};
    use adw::subclass::prelude::*;
    use gtk::{
        glib::{self, clone, subclass::Signal},
        prelude::*,
        subclass::prelude::*,
    };
    use std::{cell::RefCell, str::FromStr};

    #[derive(Default)]
    pub struct UnitSpinButtonMut {
        pub current_unit: Option<UnitKind>,
        pub current_unit_system: Option<UnitSystem>,
        pub settings_handler_id: Option<glib::SignalHandlerId>,
    }

    pub struct UnitSpinButton {
        pub inner: RefCell<UnitSpinButtonMut>,
        pub spin_button: gtk::SpinButton,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for UnitSpinButton {
        const NAME: &'static str = "HealthUnitSpinButton";
        type ParentType = adw::Bin;
        type Type = super::UnitSpinButton;
        type Interfaces = (gtk::Editable,);

        fn new() -> Self {
            Self {
                inner: Default::default(),
                spin_button: gtk::SpinButton::new(None::<&gtk::Adjustment>, 0.0, 0),
            }
        }
    }

    impl ObjectImpl for UnitSpinButton {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            self.spin_button.init_delegate();

            obj.set_property("child", &self.spin_button).unwrap();
            obj.set_property("auto-update-unit-system", true).unwrap();
            obj.connect_handlers();
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.spin_button.finish_delegate();
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_object(
                        "adjustment",
                        "adjustment",
                        "adjustment",
                        gtk::Adjustment::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_boolean(
                        "auto-update-unit-system",
                        "auto-update-unit-system",
                        "auto-update-unit-system",
                        true,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_uint(
                        "digits",
                        "digits",
                        "digits",
                        0,
                        20,
                        0,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_string(
                        "unit-kind",
                        "unit-kind",
                        "unit-kind",
                        None,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_string(
                        "unit-system",
                        "unit-system",
                        "unit-system",
                        None,
                        glib::ParamFlags::READWRITE,
                    ),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            if self.delegate_set_property(obj, id, value, pspec) {
                return;
            }

            match pspec.name() {
                "adjustment" => self
                    .spin_button
                    .set_adjustment(&value.get::<gtk::Adjustment>().unwrap()),
                "auto-update-unit-system" => {
                    if value.get::<bool>().unwrap() {
                        if self.inner.borrow().settings_handler_id.is_some() {
                            return;
                        }

                        let settings = Settings::instance();
                        self.inner.borrow_mut().settings_handler_id = Some(
                        settings.connect_unit_system_changed(clone!(@weak obj => move |_, _| {
                                obj.handle_settings_unit_system_changed(Settings::instance().unit_system())
                        })));
                        obj.set_unit_system(settings.unit_system());
                    } else if let Some(id) = self.inner.borrow_mut().settings_handler_id.take() {
                        Settings::instance().disconnect(id);
                    }
                }
                "digits" => self.spin_button.set_digits(value.get().unwrap()),
                "text" => self.spin_button.set_text(value.get().unwrap()),
                "unit-kind" => {
                    self.inner.borrow_mut().current_unit =
                        Some(UnitKind::from_str(value.get().unwrap()).unwrap())
                }
                "unit-system" => obj.handle_settings_unit_system_changed(
                    UnitSystem::from_str(value.get().unwrap()).unwrap(),
                ),
                "width-chars" => self.spin_button.set_width_chars(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            if let Some(value) = self.delegate_get_property(obj, id, pspec) {
                return value;
            }

            match pspec.name() {
                "adjustment" => self.spin_button.adjustment().to_value(),
                "auto-update-unit-system" => {
                    self.inner.borrow().settings_handler_id.is_some().to_value()
                }
                "digits" => self.spin_button.digits().to_value(),
                "text" => self.spin_button.text().to_value(),
                "unit-kind" => self.inner.borrow().current_unit.map_or_else(
                    || None::<String>.to_value(),
                    |u| {
                        let unit: &str = u.into();
                        unit.to_value()
                    },
                ),
                "unit-system" => self.inner.borrow().current_unit_system.map_or_else(
                    || None::<String>.to_value(),
                    |unit_system| {
                        let unit_system: &str = unit_system.into();
                        unit_system.to_value()
                    },
                ),
                "width-chars" => self.spin_button.width_chars().to_value(),
                _ => unimplemented!(),
            }
        }

        fn signals() -> &'static [Signal] {
            use once_cell::sync::Lazy;
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("input", &[], glib::Type::UNIT.into()).build()]);

            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for UnitSpinButton {}
    impl BinImpl for UnitSpinButton {}
    impl EditableImpl for UnitSpinButton {
        fn delegate(&self, _editable: &Self::Type) -> Option<gtk::Editable> {
            Some(self.spin_button.clone().upcast())
        }
    }

    impl CellEditableImpl for UnitSpinButton {
        fn editing_done(&self, _cell_editable: &Self::Type) {
            self.spin_button.editing_done()
        }
        fn remove_widget(&self, _cell_editable: &Self::Type) {
            self.spin_button.remove_widget()
        }
        fn start_editing(&self, _cell_editable: &Self::Type, event: Option<&gtk::gdk::Event>) {
            self.spin_button.start_editing(event)
        }
    }

    impl OrientableImpl for UnitSpinButton {}
}
glib::wrapper! {
    /// A Widget for visualizing the color in legend table.
    pub struct UnitSpinButton(ObjectSubclass<imp::UnitSpinButton>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::CellEditable, gtk::ConstraintTarget, gtk::Editable, gtk::Orientable;
}

impl UnitSpinButton {
    /// Connect to a new value being entered (this is only emitted once the user is done editing!).
    ///
    /// # Arguments
    /// * `callback` - The callback to call once the signal is emitted.
    ///
    /// # Returns
    /// The [glib::SignalHandlerId] to disconnect the signal later on.
    pub fn connect_changed<F: Fn(&gtk::SpinButton) + 'static>(
        &self,
        callback: F,
    ) -> glib::SignalHandlerId {
        self.connect_local(
            "changed",
            false,
            clone!(@weak self as obj => @default-panic, move |_| {
                callback(&obj.imp().spin_button);
                None
            }),
        )
        .unwrap()
    }

    /// Connect to a new value being entered (this is emitted for every change (e.g. key hit) the user does!).
    ///
    /// # Arguments
    /// * `callback` - The callback to call once the signal is emitted.
    ///
    /// # Returns
    /// The [glib::SignalHandlerId] to disconnect the signal later on.
    pub fn connect_input<F: Fn(&gtk::SpinButton) + 'static>(
        &self,
        callback: F,
    ) -> glib::SignalHandlerId {
        self.connect_local(
            "input",
            false,
            clone!(@weak self as obj => @default-panic, move |_| {
                callback(&obj.imp().spin_button);
                None
            }),
        )
        .unwrap()
    }

    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create UnitSpinButton")
    }

    pub fn set_unit_kind(&self, unit: UnitKind) {
        let str: &'static str = unit.into();
        self.set_property("unit-kind", str).unwrap();
    }

    pub fn set_unit_system(&self, unit_system: UnitSystem) {
        let str: &'static str = unit_system.into();
        self.set_property("unit-system", str).unwrap();
    }

    pub fn set_value(&self, value: f64) {
        self.imp().spin_button.set_value(value);
    }
    pub fn value(&self) -> f64 {
        self.imp().spin_button.value()
    }

    fn connect_handlers(&self) {
        let self_ = self.imp();

        self_
            .spin_button
            .connect_changed(clone!(@weak self as obj => move |_| {
                obj.emit_by_name("changed", &[]).unwrap();
            }));

        self_
            .spin_button
            .connect_input(clone!(@weak self as obj => @default-panic, move |_| {
                obj.handle_spin_button_input()
            }));

        self_
            .spin_button
            .connect_output(clone!(@weak self as obj => @default-panic, move |_| {
                obj.handle_spin_button_output()
            }));
    }

    fn handle_settings_unit_system_changed(&self, unit_system: UnitSystem) {
        let self_ = self.imp();
        self_.inner.borrow_mut().current_unit_system = Some(unit_system);
        self_.spin_button.update();
    }

    fn handle_spin_button_input(&self) -> Option<Result<f64, ()>> {
        self.emit_by_name("input", &[]).unwrap();

        self.imp()
            .spin_button
            .text()
            .split(' ')
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .map(Ok)
    }

    fn handle_spin_button_output(&self) -> gtk::Inhibit {
        let self_ = self.imp();
        let inner = self_.inner.borrow();

        if let Some(unit_string) = match (inner.current_unit_system, inner.current_unit) {
            // TRANSLATORS: Unit abbreviation (centimeters)
            (Some(UnitSystem::Metric), Some(UnitKind::LengthSmall)) => Some(i18n("cm")),
            // TRANSLATORS: Unit abbreviation (meters)
            (Some(UnitSystem::Metric), Some(UnitKind::LengthBig)) => Some(i18n("m")),
            // TRANSLATORS: Unit abbreviation (kilograms)
            (Some(UnitSystem::Metric), Some(UnitKind::WeightBig)) => Some(i18n("kg")),
            // TRANSLATORS: Unit abbreviation (inch)
            (Some(UnitSystem::Imperial), Some(UnitKind::LengthSmall)) => Some(i18n("in")),
            // TRANSLATORS: Unit abbreviation (feet)
            (Some(UnitSystem::Imperial), Some(UnitKind::LengthBig)) => Some(i18n("ft")),
            // TRANSLATORS: Unit abbreviation (pounds)
            (Some(UnitSystem::Imperial), Some(UnitKind::WeightBig)) => Some(i18n("lb")),
            _ => None,
        } {
            let text = format!(
                "{} {}",
                self_.spin_button.adjustment().value().to_string(),
                unit_string
            );
            if text != self_.spin_button.text() {
                self_.spin_button.set_text(&text);
            }
            gtk::Inhibit(true)
        } else {
            gtk::Inhibit(false)
        }
    }

    fn imp(&self) -> &imp::UnitSpinButton {
        imp::UnitSpinButton::from_instance(self)
    }
}

impl Default for UnitSpinButton {
    fn default() -> Self {
        Self::new()
    }
}
