use gio::prelude::*;
use glib::subclass::types::ObjectSubclass;
use uom::si::f32::Length;

mod imp {
    use crate::core::{i18n, settings::Unitsystem, utils::get_spinbutton_value, Settings};
    use adw::subclass::prelude::*;
    use glib::{
        clone,
        subclass::{self, Signal},
    };
    use gtk::{prelude::*, subclass::prelude::*, CompositeTemplate};
    use std::cell::RefCell;
    use uom::si::{
        f32::Length,
        length::{kilometer, meter, mile, yard},
    };

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
            let set_togglebutton_text = clone!(@weak obj => move || {
                let self_ = DistanceActionRow::from_instance(&obj);
                if self_.settings.get_unitsystem() == Unitsystem::Metric {
                    self_.big_unit_togglebutton.set_label (&i18n("KM"));
                    self_.small_unit_togglebutton.set_label (&i18n("Meters"));
                } else {
                    self_.big_unit_togglebutton.set_label (&i18n("Miles"));
                    self_.small_unit_togglebutton.set_label (&i18n("Yards"));
                }
            });

            set_togglebutton_text();
            self.settings
                .connect_unitsystem_changed(move |_, _| set_togglebutton_text());

            self.small_unit_togglebutton
                .connect_toggled(clone!(@weak obj => move |btn| {
                    let adjustment = &DistanceActionRow::from_instance(&obj).distance_adjustment;
                    if btn.get_active() {
                        adjustment.set_step_increment(100.0);
                        adjustment.set_page_increment(1000.0);
                    } else {
                        adjustment.set_step_increment(1.0);
                        adjustment.set_page_increment(5.0);
                    }
                }));

            self.distance_spin_button
                .connect_changed(clone!(@weak obj => move |e| {
                    let self_ = DistanceActionRow::from_instance(&obj);
                    let value = get_spinbutton_value::<f32>(e);

                    if self_.settings.get_unitsystem() == Unitsystem::Metric {
                        if self_.small_unit_togglebutton.get_active() {
                            self_.value.replace(Length::new::<meter>(value));
                        } else {
                            self_.value.replace(Length::new::<kilometer>(value));
                        }
                    } else {
                        if self_.small_unit_togglebutton.get_active() {
                                self_.value.replace(Length::new::<yard>(value));
                        } else {
                            self_.value.replace(Length::new::<mile>(value));
                        }
                    }
                    obj.emit("changed", &[]).unwrap();
                }));

            self.distance_spin_button
                .connect_input(clone!(@weak obj => move |_| {
                    obj.emit("input", &[]).unwrap();
                    None
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

    impl DistanceActionRow {
        pub fn get_value(&self) -> Length {
            *self.value.borrow()
        }

        pub fn set_value(&self, value: Length) {
            // FIXME: Disallow both buttons being inactive

            if self.settings.get_unitsystem() == Unitsystem::Metric {
                if self.small_unit_togglebutton.get_active() {
                    self.distance_spin_button
                        .set_value(value.get::<meter>().into())
                } else if self.big_unit_togglebutton.get_active() {
                    self.distance_spin_button
                        .set_value(value.get::<kilometer>().into())
                }
            } else if self.small_unit_togglebutton.get_active() {
                self.distance_spin_button
                    .set_value(value.get::<yard>().into())
            } else if self.big_unit_togglebutton.get_active() {
                self.distance_spin_button
                    .set_value(value.get::<mile>().into())
            }

            self.value.replace(value);
        }
    }
}

glib::wrapper! {
    pub struct DistanceActionRow(ObjectSubclass<imp::DistanceActionRow>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::ActionRow;
}

impl DistanceActionRow {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create DistanceActionRow")
    }

    pub fn get_value(&self) -> Length {
        imp::DistanceActionRow::from_instance(self).get_value()
    }

    pub fn set_value(&self, value: Length) {
        imp::DistanceActionRow::from_instance(self).set_value(value)
    }

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
}
