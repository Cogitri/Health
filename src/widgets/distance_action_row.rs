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
    core::{i18n, settings::Unitsystem, utils::get_spinbutton_value},
    model::Unitsize,
};
use gio::subclass::prelude::*;
use gtk::prelude::*;
use uom::si::{
    f32::Length,
    length::{kilometer, meter, mile, yard},
};

mod imp {
    use crate::{core::Settings, model::Unitsize};
    use adw::subclass::prelude::*;
    use glib::{clone, subclass::Signal};
    use gtk::{prelude::*, subclass::prelude::*, CompositeTemplate};
    use std::cell::RefCell;
    use uom::si::{f32::Length, length::meter};

    #[derive(Debug)]
    pub struct DistanceActionRowMut {
        pub unitsize: Unitsize,
        pub value: Length,
    }

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/dev/Cogitri/Health/ui/distance_action_row.ui")]
    pub struct DistanceActionRow {
        pub inner: RefCell<DistanceActionRowMut>,
        pub settings: Settings,
        pub settings_handler_id: RefCell<Option<glib::SignalHandlerId>>,
        #[template_child]
        pub distance_adjustment: TemplateChild<gtk::Adjustment>,
        #[template_child]
        pub distance_spin_button: TemplateChild<gtk::SpinButton>,
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

        fn new() -> Self {
            Self {
                inner: RefCell::new(DistanceActionRowMut {
                    unitsize: Unitsize::Small,
                    value: Length::new::<meter>(0.0),
                }),
                settings: Settings::get_instance(),
                settings_handler_id: RefCell::new(None),
                distance_adjustment: TemplateChild::default(),
                distance_spin_button: TemplateChild::default(),
                big_unit_togglebutton: TemplateChild::default(),
                small_unit_togglebutton: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for DistanceActionRow {
        fn constructed(&self, obj: &Self::Type) {
            obj.set_togglebutton_text();
            self.settings_handler_id
                .replace(Some(self.settings.connect_unitsystem_changed(
                    clone!(@weak obj => move |_, _| obj.set_togglebutton_text()),
                )));

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
        @extends gtk::Widget, gtk::ListBoxRow, adw::ActionRow;
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

    pub fn get_value(&self) -> Length {
        self.get_priv().inner.borrow().value
    }

    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create DistanceActionRow")
    }

    pub fn set_unitsize(&self, unitsize: Unitsize) {
        let adjustment = &self.get_priv().distance_adjustment;
        if unitsize == Unitsize::Small {
            adjustment.set_step_increment(100.0);
            adjustment.set_page_increment(1000.0);
        } else {
            adjustment.set_step_increment(1.0);
            adjustment.set_page_increment(5.0);
        }

        let val = {
            let mut inner = self.get_priv().inner.borrow_mut();
            inner.unitsize = unitsize;
            inner.value
        };
        self.set_value(val)
    }

    pub fn set_value(&self, value: Length) {
        // FIXME: Disallow both buttons being inactive
        let self_ = self.get_priv();
        let unitsize = self_.inner.borrow().unitsize;

        if self_.settings.get_unitsystem() == Unitsystem::Metric {
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
                .set_value(value.get::<yard>().into())
        } else if unitsize == Unitsize::Big {
            self_
                .distance_spin_button
                .set_value(value.get::<mile>().into())
        }

        self_.inner.borrow_mut().value = value;
    }

    fn get_priv(&self) -> &imp::DistanceActionRow {
        imp::DistanceActionRow::from_instance(self)
    }

    fn handle_distance_spin_button_changed(&self, spinbutton: &gtk::SpinButton) {
        let self_ = self.get_priv();
        let value = get_spinbutton_value::<f32>(spinbutton);
        let unitsize = self_.inner.borrow().unitsize;

        if self_.settings.get_unitsystem() == Unitsystem::Metric {
            if unitsize == Unitsize::Small {
                self_.inner.borrow_mut().value = Length::new::<meter>(value);
            } else {
                self_.inner.borrow_mut().value = Length::new::<kilometer>(value);
            }
        } else if unitsize == Unitsize::Small {
            self_.inner.borrow_mut().value = Length::new::<yard>(value);
        } else {
            self_.inner.borrow_mut().value = Length::new::<mile>(value);
        }
        self.emit_by_name("changed", &[]).unwrap();
    }

    fn handle_distance_spin_button_input(&self) -> Option<Result<f64, ()>> {
        self.emit_by_name("input", &[]).unwrap();
        None
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
