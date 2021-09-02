/* distance_action_row.rs
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

use crate::{
    core::{i18n, utils::prelude::*, UnitKind, UnitSystem},
    model::Unitsize,
};
use gtk::{gio::subclass::prelude::*, glib, prelude::*};
use uom::si::{
    f32::Length,
    length::{foot, kilometer, meter, mile},
};

mod imp {
    use crate::{core::Settings, model::Unitsize, widgets::UnitSpinButton};
    use adw::subclass::prelude::*;
    use gtk::{
        glib::{self, clone, subclass::Signal},
        prelude::*,
        subclass::prelude::*,
        CompositeTemplate,
    };
    use std::cell::RefCell;
    use uom::si::f32::Length;

    #[derive(Debug, Default)]
    pub struct DistanceActionRowMut {
        pub unitsize: Unitsize,
        pub value: Length,
    }

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/distance_action_row.ui")]
    pub struct DistanceActionRow {
        pub inner: RefCell<DistanceActionRowMut>,
        pub settings: Settings,
        pub settings_handler_id: RefCell<Option<glib::SignalHandlerId>>,
        #[template_child]
        pub distance_adjustment: TemplateChild<gtk::Adjustment>,
        #[template_child]
        pub distance_spin_button: TemplateChild<UnitSpinButton>,
        #[template_child]
        pub big_unit_togglebutton: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub small_unit_togglebutton: TemplateChild<gtk::ToggleButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DistanceActionRow {
        const NAME: &'static str = "HealthDistanceActionRow";
        type ParentType = adw::ActionRow;
        type Type = super::DistanceActionRow;

        fn class_init(klass: &mut Self::Class) {
            UnitSpinButton::static_type();
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for DistanceActionRow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.set_togglebutton_text();
            self.settings_handler_id
                .replace(Some(self.settings.connect_unit_system_changed(
                    clone!(@weak obj => move |_, _| obj.set_togglebutton_text()),
                )));

            self.distance_spin_button
                .connect_changed(clone!(@weak obj => move |s| {
                    obj.handle_distance_spin_button_changed(s);
                }));

            self.distance_spin_button
                .connect_input(clone!(@weak obj => @default-panic, move |_| {
                    obj.emit_by_name("input", &[]).unwrap();
                }));
        }

        fn signals() -> &'static [Signal] {
            use once_cell::sync::Lazy;
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("input", &[], glib::Type::UNIT.into()).build(),
                    Signal::builder("changed", &[], glib::Type::UNIT.into()).build(),
                ]
            });

            SIGNALS.as_ref()
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.settings
                .disconnect(self.settings_handler_id.borrow_mut().take().unwrap())
        }
    }
    impl WidgetImpl for DistanceActionRow {}
    impl ListBoxRowImpl for DistanceActionRow {}
    impl ActionRowImpl for DistanceActionRow {}
}

glib::wrapper! {
    /// An implementation [adw::ActionRow] that contains a [gtk::SpinButton] and also allows switching
    /// between small&big units (e.g. kilometer vs meter) via a [gtk::ToggleButton].
    pub struct DistanceActionRow(ObjectSubclass<imp::DistanceActionRow>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl DistanceActionRow {
    /// Connect to a new value being entered (this is only emitted once the user is done editing!).
    ///
    /// # Arguments
    /// * `callback` - The callback to call once the signal is emitted.
    ///
    /// # Returns
    /// The [glib::SignalHandlerId] to disconnect the signal later on.
    pub fn connect_changed<F: Fn() + 'static>(&self, callback: F) -> glib::SignalHandlerId {
        self.connect_local("changed", false, move |_| {
            callback();
            None
        })
        .unwrap()
    }

    /// Connect to a new value being entered (this is emitted for every change (e.g. key hit) the user does!).
    ///
    /// # Arguments
    /// * `callback` - The callback to call once the signal is emitted.
    ///
    /// # Returns
    /// The [glib::SignalHandlerId] to disconnect the signal later on.
    pub fn connect_input<F: Fn() + 'static>(&self, callback: F) -> glib::SignalHandlerId {
        self.connect_local("input", false, move |_| {
            callback();
            None
        })
        .unwrap()
    }

    pub fn value(&self) -> Length {
        self.imp().inner.borrow().value
    }

    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create DistanceActionRow")
    }

    pub fn set_unitsize(&self, unitsize: Unitsize) {
        let self_ = self.imp();
        let adjustment = &self_.distance_adjustment;
        if unitsize == Unitsize::Small {
            adjustment.set_step_increment(100.0);
            adjustment.set_page_increment(1000.0);
            self_
                .distance_spin_button
                .set_unit_kind(UnitKind::LikeMeters);
        } else {
            adjustment.set_step_increment(1.0);
            adjustment.set_page_increment(5.0);
            self_
                .distance_spin_button
                .set_unit_kind(UnitKind::LikeKilometers);
        }

        if unitsize == Unitsize::Big && !self_.big_unit_togglebutton.is_active() {
            self_.big_unit_togglebutton.set_active(true);
        } else if unitsize == Unitsize::Small && !self_.small_unit_togglebutton.is_active() {
            self_.small_unit_togglebutton.set_active(true);
        }

        let val = {
            let mut inner = self.imp().inner.borrow_mut();
            inner.unitsize = unitsize;
            inner.value
        };
        self.set_value(val)
    }

    pub fn set_value(&self, value: Length) {
        // FIXME: Disallow both buttons being inactive
        let self_ = self.imp();
        let unitsize = self_.inner.borrow().unitsize;

        if self_.settings.unit_system() == UnitSystem::Metric {
            if unitsize == Unitsize::Small {
                self_
                    .distance_spin_button
                    .set_value(value.get::<meter>().into())
            } else if unitsize == Unitsize::Big {
                self_
                    .distance_spin_button
                    .set_value(value.get::<kilometer>().into())
            }
        } else if unitsize == Unitsize::Small {
            self_
                .distance_spin_button
                .set_value(value.get::<foot>().into())
        } else if unitsize == Unitsize::Big {
            self_
                .distance_spin_button
                .set_value(value.get::<mile>().into())
        }

        self_.inner.borrow_mut().value = value;
    }

    fn imp(&self) -> &imp::DistanceActionRow {
        imp::DistanceActionRow::from_instance(self)
    }

    fn handle_distance_spin_button_changed(&self, spinbutton: &gtk::SpinButton) {
        let self_ = self.imp();
        let value = spinbutton.raw_value::<f32>().unwrap_or_default();
        let unitsize = self_.inner.borrow().unitsize;

        if self_.settings.unit_system() == UnitSystem::Metric {
            if unitsize == Unitsize::Small {
                self_.inner.borrow_mut().value = Length::new::<meter>(value);
            } else {
                self_.inner.borrow_mut().value = Length::new::<kilometer>(value);
            }
        } else if unitsize == Unitsize::Small {
            self_.inner.borrow_mut().value = Length::new::<foot>(value);
        } else {
            self_.inner.borrow_mut().value = Length::new::<mile>(value);
        }
        self.emit_by_name("changed", &[]).unwrap();
    }

    fn set_togglebutton_text(&self) {
        let self_ = self.imp();
        if self_.settings.unit_system() == UnitSystem::Metric {
            self_.big_unit_togglebutton.set_label(&i18n("KM"));
            self_.small_unit_togglebutton.set_label(&i18n("Meters"));
        } else {
            self_.big_unit_togglebutton.set_label(&i18n("Miles"));
            self_.small_unit_togglebutton.set_label(&i18n("Feet"));
        }
    }
}

#[cfg(test)]
mod test {
    use super::DistanceActionRow;
    use crate::model::Unitsize;
    use uom::si::{f32::Length, length::meter};

    #[test]
    pub fn set_unitsize() {
        crate::utils::init_gtk();

        let row = DistanceActionRow::new();
        let row_ = row.imp();
        row.set_value(Length::new::<meter>(1500.0));
        assert_eq!(row_.distance_spin_button.value(), 1500.0);
        row.set_unitsize(Unitsize::Big);
        assert_eq!(row_.distance_spin_button.value(), 1.5);
    }
}
