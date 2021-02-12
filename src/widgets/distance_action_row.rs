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

use crate::core::{i18n, settings::Unitsystem, utils::get_spinbutton_value};
use glib::subclass::types::ObjectSubclass;
use gtk::prelude::*;
use uom::si::{
    f32::Length,
    length::{kilometer, meter, mile, yard},
};

mod imp {
    use crate::core::Settings;
    use adw::subclass::prelude::*;
    use glib::{
        clone,
        subclass::{self, Signal},
    };
    use gtk::{prelude::*, subclass::prelude::*, CompositeTemplate};
    use std::cell::RefCell;
    use uom::si::{f32::Length, length::meter};

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/dev/Cogitri/Health/ui/distance_action_row.ui")]
    pub struct DistanceActionRow {
        pub settings: Settings,
        pub value: RefCell<Length>,
        #[template_child]
        pub distance_adjustment: TemplateChild<gtk::Adjustment>,
        #[template_child]
        pub distance_spin_button: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub big_unit_togglebutton: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub small_unit_togglebutton: TemplateChild<gtk::ToggleButton>,
    }

    impl ObjectSubclass for DistanceActionRow {
        const NAME: &'static str = "HealthDistanceActionRow";
        type ParentType = adw::ActionRow;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::DistanceActionRow;
        type Interfaces = ();

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                settings: Settings::new(),
                value: RefCell::new(Length::new::<meter>(0.0)),
                distance_adjustment: TemplateChild::default(),
                distance_spin_button: TemplateChild::default(),
                big_unit_togglebutton: TemplateChild::default(),
                small_unit_togglebutton: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self::Type>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for DistanceActionRow {
        fn constructed(&self, obj: &Self::Type) {
            obj.set_togglebutton_text();
            self.settings.connect_unitsystem_changed(
                clone!(@weak obj => move |_, _| obj.set_togglebutton_text()),
            );

            self.small_unit_togglebutton
                .connect_toggled(clone!(@weak obj => move |btn| {
                    obj.handle_small_unit_togglebutton_toggle(btn)
                }));

            self.distance_spin_button
                .connect_changed(clone!(@weak obj => move |s| {
                    obj.handle_distance_spin_button_changed(s);
                }));

            self.distance_spin_button
                .connect_input(clone!(@weak obj => move |_| {
                    obj.handle_distance_spin_button_input()
                }));
        }

        fn signals() -> &'static [Signal] {
            use once_cell::sync::Lazy;
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("input", &[], glib::Type::Unit).build(),
                    Signal::builder("changed", &[], glib::Type::Unit).build(),
                ]
            });

            SIGNALS.as_ref()
        }
    }
    impl WidgetImpl for DistanceActionRow {}
    impl ListBoxRowImpl for DistanceActionRow {}
    impl ActionRowImpl for DistanceActionRow {}
}

glib::wrapper! {
    pub struct DistanceActionRow(ObjectSubclass<imp::DistanceActionRow>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::ActionRow;
}

impl DistanceActionRow {
    pub fn connect_changed<F: Fn() + 'static>(&self, callback: F) -> glib::SignalHandlerId {
        self.connect_local("changed", false, move |_| {
            callback();
            None
        })
        .unwrap()
    }

    pub fn connect_input<F: Fn() + 'static>(&self, callback: F) -> glib::SignalHandlerId {
        self.connect_local("input", false, move |_| {
            callback();
            None
        })
        .unwrap()
    }

    pub fn get_value(&self) -> Length {
        *self.get_priv().value.borrow()
    }

    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create DistanceActionRow")
    }

    pub fn set_value(&self, value: Length) {
        // FIXME: Disallow both buttons being inactive
        let self_ = self.get_priv();

        if self_.settings.get_unitsystem() == Unitsystem::Metric {
            if self_.small_unit_togglebutton.get_active() {
                self_
                    .distance_spin_button
                    .set_value(value.get::<meter>().into())
            } else if self_.big_unit_togglebutton.get_active() {
                self_
                    .distance_spin_button
                    .set_value(value.get::<kilometer>().into())
            }
        } else if self_.small_unit_togglebutton.get_active() {
            self_
                .distance_spin_button
                .set_value(value.get::<yard>().into())
        } else if self_.big_unit_togglebutton.get_active() {
            self_
                .distance_spin_button
                .set_value(value.get::<mile>().into())
        }

        self_.value.replace(value);
    }

    fn get_priv(&self) -> &imp::DistanceActionRow {
        imp::DistanceActionRow::from_instance(self)
    }

    fn handle_distance_spin_button_changed(&self, spinbutton: &gtk::SpinButton) {
        let self_ = self.get_priv();
        let value = get_spinbutton_value::<f32>(spinbutton);

        if self_.settings.get_unitsystem() == Unitsystem::Metric {
            if self_.small_unit_togglebutton.get_active() {
                self_.value.replace(Length::new::<meter>(value));
            } else {
                self_.value.replace(Length::new::<kilometer>(value));
            }
        } else if self_.small_unit_togglebutton.get_active() {
            self_.value.replace(Length::new::<yard>(value));
        } else {
            self_.value.replace(Length::new::<mile>(value));
        }
        self.emit("changed", &[]).unwrap();
    }

    fn handle_distance_spin_button_input(&self) -> Option<Result<f64, ()>> {
        self.emit("input", &[]).unwrap();
        None
    }

    fn handle_small_unit_togglebutton_toggle(&self, btn: &gtk::ToggleButton) {
        let adjustment = &self.get_priv().distance_adjustment;
        if btn.get_active() {
            adjustment.set_step_increment(100.0);
            adjustment.set_page_increment(1000.0);
        } else {
            adjustment.set_step_increment(1.0);
            adjustment.set_page_increment(5.0);
        }
    }

    fn set_togglebutton_text(&self) {
        let self_ = self.get_priv();
        if self_.settings.get_unitsystem() == Unitsystem::Metric {
            self_.big_unit_togglebutton.set_label(&i18n("KM"));
            self_.small_unit_togglebutton.set_label(&i18n("Meters"));
        } else {
            self_.big_unit_togglebutton.set_label(&i18n("Miles"));
            self_.small_unit_togglebutton.set_label(&i18n("Yards"));
        }
    }
}
