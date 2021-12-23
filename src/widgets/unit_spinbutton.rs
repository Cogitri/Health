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

use crate::{
    core::{i18n, UnitKind, UnitSystem},
    prelude::*,
};
use gtk::{
    glib::{
        subclass::prelude::*,
        {self, clone},
    },
    prelude::*,
};
use uom::si::{
    f32::{Length, Mass},
    length::{centimeter, foot, inch, kilometer, meter, mile},
    mass::{kilogram, pound},
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
        pub current_unit_kind: Option<UnitKind>,
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
                spin_button: gtk::SpinButton::new(None::<&gtk::Adjustment>, 10.0, 2),
            }
        }
    }

    impl ObjectImpl for UnitSpinButton {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            self.spin_button.init_delegate();

            obj.set_property("child", &self.spin_button);
            obj.connect_handlers();
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.spin_button.finish_delegate();
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::new(
                        "adjustment",
                        "adjustment",
                        "adjustment",
                        gtk::Adjustment::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT,
                    ),
                    glib::ParamSpecBoolean::new(
                        "auto-update-unit-system",
                        "auto-update-unit-system",
                        "auto-update-unit-system",
                        true,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT,
                    ),
                    glib::ParamSpecUInt::new(
                        "digits",
                        "digits",
                        "digits",
                        0,
                        20,
                        1,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT,
                    ),
                    glib::ParamSpecBoolean::new(
                        "has-default-value",
                        "has-default-value",
                        "has-default-value",
                        true,
                        glib::ParamFlags::READABLE,
                    ),
                    glib::ParamSpecString::new(
                        "unit-kind",
                        "unit-kind",
                        "unit-kind",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT,
                    ),
                    glib::ParamSpecString::new(
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
                "unit-kind" => obj.handle_settings_unit_kind_changed(
                    UnitKind::from_str(value.get().unwrap()).unwrap(),
                ),
                "unit-system" => obj.handle_settings_unit_system_changed(
                    UnitSystem::from_str(value.get().unwrap()).unwrap(),
                ),
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
                "has-default-value" => self
                    .spin_button
                    .text()
                    .split(' ')
                    .next()
                    .map_or_else(|| false, |val| (val == "0"))
                    .to_value(),
                "unit-kind" => self
                    .inner
                    .borrow()
                    .current_unit_kind
                    .map_or_else(|| None::<String>.to_value(), |u| u.to_value()),
                "unit-system" => self
                    .inner
                    .borrow()
                    .current_unit_system
                    .map_or_else(|| None::<String>.to_value(), |u| u.to_value()),
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

enum Value {
    Length(Length),
    Mass(Mass),
}

macro_rules! get_value {
    ($name:expr, $type:path, $unit:ty) => {
        match $name {
            $type(v) => v.get::<$unit>().into(),
            _ => unimplemented!(),
        }
    };
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
    }

    pub fn new(
        adjustment: &gtk::Adjustment,
        auto_update_unit_system: bool,
        unit_kind: UnitKind,
    ) -> Self {
        glib::Object::new(&[
            ("adjustment", adjustment),
            ("auto-update-unit-system", &auto_update_unit_system),
            ("unit-kind", &unit_kind),
        ])
        .expect("Failed to create UnitSpinButton")
    }

    pub fn has_default_value(&self) -> bool {
        self.property::<bool>("has-default-value")
    }

    pub fn set_unit_kind(&self, unit_kind: UnitKind) {
        self.set_property("unit-kind", unit_kind);
    }

    pub fn set_unit_system(&self, unit_system: UnitSystem) {
        self.set_property("unit-system", unit_system);
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
                obj.emit_by_name::<()>("changed", &[]);
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

    fn handle_conversion(
        &self,
        previous_unit_kind: Option<UnitKind>,
        previous_unit_system: Option<UnitSystem>,
    ) -> bool {
        let self_ = self.imp();

        if let Some(value) = self.raw_value() {
            if let Some(old_value) = match (previous_unit_system, previous_unit_kind) {
                (Some(UnitSystem::Metric), Some(UnitKind::LikeCentimeters)) => {
                    Some(Value::Length(Length::new::<centimeter>(value)))
                }
                (Some(UnitSystem::Metric), Some(UnitKind::LikeMeters)) => {
                    Some(Value::Length(Length::new::<meter>(value)))
                }
                (Some(UnitSystem::Metric), Some(UnitKind::LikeKilometers)) => {
                    Some(Value::Length(Length::new::<kilometer>(value)))
                }
                (Some(UnitSystem::Metric), Some(UnitKind::LikeKilogram)) => {
                    Some(Value::Mass(Mass::new::<kilogram>(value)))
                }
                (Some(UnitSystem::Imperial), Some(UnitKind::LikeCentimeters)) => {
                    Some(Value::Length(Length::new::<inch>(value)))
                }
                (Some(UnitSystem::Imperial), Some(UnitKind::LikeMeters)) => {
                    Some(Value::Length(Length::new::<foot>(value)))
                }
                (Some(UnitSystem::Imperial), Some(UnitKind::LikeKilometers)) => {
                    Some(Value::Length(Length::new::<mile>(value)))
                }
                (Some(UnitSystem::Imperial), Some(UnitKind::LikeKilogram)) => {
                    Some(Value::Mass(Mass::new::<pound>(value)))
                }
                _ => None,
            } {
                match (
                    self_.inner.borrow().current_unit_system,
                    self_.inner.borrow().current_unit_kind,
                ) {
                    (Some(UnitSystem::Metric), Some(UnitKind::LikeCentimeters)) => {
                        self.set_value(get_value!(old_value, Value::Length, centimeter))
                    }
                    (Some(UnitSystem::Metric), Some(UnitKind::LikeMeters)) => {
                        self.set_value(get_value!(old_value, Value::Length, meter))
                    }
                    (Some(UnitSystem::Metric), Some(UnitKind::LikeKilometers)) => {
                        self.set_value(get_value!(old_value, Value::Length, kilometer))
                    }
                    (Some(UnitSystem::Metric), Some(UnitKind::LikeKilogram)) => {
                        self.set_value(get_value!(old_value, Value::Mass, kilogram))
                    }
                    (Some(UnitSystem::Imperial), Some(UnitKind::LikeCentimeters)) => {
                        self.set_value(get_value!(old_value, Value::Length, inch))
                    }
                    (Some(UnitSystem::Imperial), Some(UnitKind::LikeMeters)) => {
                        self.set_value(get_value!(old_value, Value::Length, foot))
                    }
                    (Some(UnitSystem::Imperial), Some(UnitKind::LikeKilometers)) => {
                        self.set_value(get_value!(old_value, Value::Length, mile))
                    }
                    (Some(UnitSystem::Imperial), Some(UnitKind::LikeKilogram)) => {
                        self.set_value(get_value!(old_value, Value::Mass, pound))
                    }
                    _ => {}
                }
                return true;
            }
        }

        false
    }

    fn handle_settings_unit_kind_changed(&self, unit_kind: UnitKind) {
        let self_ = self.imp();
        let (current_unit_kind, current_unit_system) = {
            let inner = self_.inner.borrow();
            (inner.current_unit_kind, inner.current_unit_system)
        };
        self_.inner.borrow_mut().current_unit_kind = Some(unit_kind);

        if !self.handle_conversion(current_unit_kind, current_unit_system) {
            self_.spin_button.update();
        }
    }

    fn handle_settings_unit_system_changed(&self, unit_system: UnitSystem) {
        let self_ = self.imp();
        let (current_unit_system, current_unit_kind) = {
            let inner = self_.inner.borrow();
            (inner.current_unit_system, inner.current_unit_kind)
        };
        self_.inner.borrow_mut().current_unit_system = Some(unit_system);

        if !self.handle_conversion(current_unit_kind, current_unit_system) {
            self_.spin_button.update();
        }
    }

    fn handle_spin_button_input(&self) -> Option<Result<f64, ()>> {
        self.emit_by_name::<()>("input", &[]);

        self.raw_value().map(Ok)
    }

    fn handle_spin_button_output(&self) -> gtk::Inhibit {
        let self_ = self.imp();
        let inner = self_.inner.borrow();

        if let Some(unit_string) = match (inner.current_unit_system, inner.current_unit_kind) {
            // TRANSLATORS: Unit abbreviation (centimeters)
            (Some(UnitSystem::Metric), Some(UnitKind::LikeCentimeters)) => Some(i18n("cm")),
            // TRANSLATORS: Unit abbreviation (meters)
            (Some(UnitSystem::Metric), Some(UnitKind::LikeMeters)) => Some(i18n("m")),
            // TRANSLATORS: Unit abbreviation (kilometers)
            (Some(UnitSystem::Metric), Some(UnitKind::LikeKilometers)) => Some(i18n("km")),
            // TRANSLATORS: Unit abbreviation (kilograms)
            (Some(UnitSystem::Metric), Some(UnitKind::LikeKilogram)) => Some(i18n("kg")),
            // TRANSLATORS: Unit abbreviation (inch)
            (Some(UnitSystem::Imperial), Some(UnitKind::LikeCentimeters)) => Some(i18n("in")),
            // TRANSLATORS: Unit abbreviation (feet)
            (Some(UnitSystem::Imperial), Some(UnitKind::LikeMeters)) => Some(i18n("ft")),
            // TRANSLATORS: Unit abbreviation (miles)
            (Some(UnitSystem::Imperial), Some(UnitKind::LikeKilometers)) => Some(i18n("mi")),
            // TRANSLATORS: Unit abbreviation (pounds)
            (Some(UnitSystem::Imperial), Some(UnitKind::LikeKilogram)) => Some(i18n("lb")),
            _ => None,
        } {
            let text = format!(
                "{} {}",
                self_
                    .spin_button
                    .adjustment()
                    .value()
                    .round_decimal_places(self_.spin_button.digits()),
                unit_string,
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

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! test_from_to {
        ($from:expr, $to:expr, $kind:expr, $expected:literal) => {
            crate::utils::init_gtk();

            let btn = UnitSpinButton::new(
                &gtk::Adjustment::new(10.0, 0.0, 1000.0, 10.0, 100.0, 0.0),
                false,
                $kind,
            );
            assert_eq!(btn.value(), 10.0);
            btn.set_unit_system($from);
            assert_eq!(
                btn.value(),
                10.0,
                "Changed value when setting initial unit system when it shouldn't"
            );

            btn.set_unit_system($from);
            assert_eq!(
                btn.value(),
                10.0,
                "Changed value when setting same unit system when it shouldn't"
            );

            btn.set_unit_system($to);
            assert_eq!(btn.value(), $expected);
        };
    }

    #[test]
    fn test_change_unit_system_small() {
        test_from_to!(
            UnitSystem::Metric,
            UnitSystem::Imperial,
            UnitKind::LikeCentimeters,
            3.9370076656341553
        );
    }

    #[test]
    fn test_change_unit_system_big() {
        test_from_to!(
            UnitSystem::Metric,
            UnitSystem::Imperial,
            UnitKind::LikeMeters,
            32.80839920043945
        );
    }

    #[test]
    fn test_change_unit_system_very_big() {
        test_from_to!(
            UnitSystem::Metric,
            UnitSystem::Imperial,
            UnitKind::LikeKilometers,
            6.213711738586426
        );
    }

    #[test]
    fn test_change_unit_system_mass_big() {
        test_from_to!(
            UnitSystem::Metric,
            UnitSystem::Imperial,
            UnitKind::LikeKilogram,
            22.04622459411621
        );
    }

    #[test]
    fn test_change_unit_system_small_imperal() {
        test_from_to!(
            UnitSystem::Imperial,
            UnitSystem::Metric,
            UnitKind::LikeCentimeters,
            25.400001525878906
        );
    }

    #[test]
    fn test_change_unit_system_big_imperial() {
        test_from_to!(
            UnitSystem::Imperial,
            UnitSystem::Metric,
            UnitKind::LikeMeters,
            3.0480000972747803
        );
    }

    #[test]
    fn test_change_unit_system_very_big_imperial() {
        test_from_to!(
            UnitSystem::Imperial,
            UnitSystem::Metric,
            UnitKind::LikeKilometers,
            16.09343910217285
        );
    }

    #[test]
    fn test_change_unit_system_mass_big_imperial() {
        test_from_to!(
            UnitSystem::Imperial,
            UnitSystem::Metric,
            UnitKind::LikeKilogram,
            4.535923957824707
        );
    }

    #[test]
    fn test_output() {
        crate::utils::init_gtk();

        let btn = UnitSpinButton::new(
            &gtk::Adjustment::new(10.0, 0.0, 1000.0, 10.0, 100.0, 0.0),
            false,
            UnitKind::LikeCentimeters,
        );
        btn.set_unit_system(UnitSystem::Metric);
        assert_eq!(btn.text(), format!("10 {}", i18n("cm")));
        btn.set_unit_system(UnitSystem::Imperial);
        assert_eq!(btn.text(), format!("3.9 {}", i18n("in")));
    }

    #[test]
    fn test_change_unit_kind_small_big() {
        crate::utils::init_gtk();

        let btn = UnitSpinButton::new(
            &gtk::Adjustment::new(100.0, 0.0, 1000.0, 10.0, 100.0, 0.0),
            false,
            UnitKind::LikeCentimeters,
        );
        btn.set_unit_system(UnitSystem::Metric);
        btn.set_unit_kind(UnitKind::LikeMeters);
        assert_eq!(btn.value(), 1.0);
        btn.set_unit_kind(UnitKind::LikeKilometers);
        assert_eq!(btn.value(), 0.0010000000474974513);
    }

    #[test]
    #[should_panic]
    fn test_change_unit_kind_invalid() {
        crate::utils::init_gtk();

        let btn = UnitSpinButton::new(
            &gtk::Adjustment::new(100.0, 0.0, 1000.0, 10.0, 100.0, 0.0),
            false,
            UnitKind::LikeCentimeters,
        );
        btn.set_unit_system(UnitSystem::Metric);
        btn.set_unit_kind(UnitKind::LikeKilogram);
    }
}
