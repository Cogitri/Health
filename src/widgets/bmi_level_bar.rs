use crate::{core::settings::Unitsystem, imp_getter_setter};
use gdk::subclass::prelude::ObjectSubclass;
use gtk::prelude::*;
use gtk::{glib, CompositeTemplate};
use uom::si::f32::{Length, Mass};

mod imp {
    use super::*;
    use crate::core::HealthSettings;
    use glib::subclass;
    use gtk::subclass::prelude::*;
    use std::cell::RefCell;
    use uom::si::{length::centimeter, mass::kilogram};

    static LEVEL_BAR_MIN: f32 = 13.5;
    static LEVEL_BAR_MAX: f32 = 30.0;

    #[derive(Debug)]
    pub struct HealthBMILevelBarMut {
        height: Length,
        weight: Mass,
        unitsystem: Unitsystem,
    }

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/dev/Cogitri/Health/ui/bmi_level_bar.ui")]
    pub struct HealthBMILevelBar {
        pub inner: RefCell<HealthBMILevelBarMut>,
        #[template_child]
        pub bmi_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub level_bar: TemplateChild<gtk::LevelBar>,
    }

    impl ObjectSubclass for HealthBMILevelBar {
        const NAME: &'static str = "HealthBMILevelBar";
        type ParentType = gtk::Widget;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::HealthBMILevelBar;
        type Interfaces = ();

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                inner: RefCell::new(HealthBMILevelBarMut {
                    height: Length::new::<centimeter>(0.0),
                    weight: Mass::new::<kilogram>(0.0),
                    unitsystem: Unitsystem::Metric,
                }),
                bmi_label: TemplateChild::default(),
                level_bar: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            klass.set_layout_manager_type::<gtk::BoxLayout>();
            klass.set_accessible_role(gtk::AccessibleRole::Meter);
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self::Type>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for HealthBMILevelBar {
        fn constructed(&self, obj: &Self::Type) {
            let settings = HealthSettings::new();
            self.inner.borrow_mut().unitsystem = settings.get_unitsystem();
            settings.connect_unitsystem_changed(glib::clone!(@weak obj, @strong settings => move |_, _| {
                HealthBMILevelBar::from_instance(&obj).inner.borrow_mut().unitsystem = settings.get_unitsystem();
            }));

            obj.get_layout_manager()
                .unwrap()
                .dynamic_cast_ref::<gtk::Orientable>()
                .unwrap()
                .set_orientation(gtk::Orientation::Vertical);

            self.level_bar
                .remove_offset_value(Some(&gtk::LEVEL_BAR_OFFSET_LOW));
            self.level_bar
                .remove_offset_value(Some(&gtk::LEVEL_BAR_OFFSET_HIGH));
            self.level_bar
                .remove_offset_value(Some(&gtk::LEVEL_BAR_OFFSET_FULL));

            self.level_bar.add_offset_value(
                "severly-underweight-bmi",
                ((18.5 - LEVEL_BAR_MIN) / (LEVEL_BAR_MAX - LEVEL_BAR_MIN)).into(),
            );
            self.level_bar.add_offset_value(
                "underweight-bmi",
                ((20.0 - LEVEL_BAR_MIN) / (LEVEL_BAR_MAX - LEVEL_BAR_MIN)).into(),
            );
            self.level_bar.add_offset_value(
                "optimal-bmi",
                ((25.0 - LEVEL_BAR_MIN) / (LEVEL_BAR_MAX - LEVEL_BAR_MIN)).into(),
            );
            self.level_bar.add_offset_value(
                "overweight-bmi",
                ((29.9 - LEVEL_BAR_MIN) / (LEVEL_BAR_MAX - LEVEL_BAR_MIN)).into(),
            );
            self.level_bar.add_offset_value("obese-bmi", 1.0);
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.get_first_child() {
                child.unparent();
            }
        }
    }
    impl WidgetImpl for HealthBMILevelBar {}

    impl HealthBMILevelBar {
        pub fn get_height(&self) -> Length {
            self.inner.borrow().height
        }

        pub fn get_weight(&self) -> Mass {
            self.inner.borrow().weight
        }

        pub fn get_unitsystem(&self) -> Unitsystem {
            self.inner.borrow().unitsystem
        }

        pub fn set_height(&self, value: Length) {
            self.inner.borrow_mut().height = value;
            self.recalcualte_bmi();
        }

        pub fn set_weight(&self, value: Mass) {
            self.inner.borrow_mut().weight = value;
            self.recalcualte_bmi();
        }

        pub fn set_unitsystem(&self, value: Unitsystem) {
            self.inner.borrow_mut().unitsystem = value;
            self.recalcualte_bmi();
        }

        fn recalcualte_bmi(&self) {
            let height = self.inner.borrow().height.get::<centimeter>() as f32 / 100.0;
            let weight = self.inner.borrow().weight.get::<kilogram>();
            if height != 0.0 && weight != 0.0 {
                let current_bmi = weight / (height * height);
                let fraction = (current_bmi - LEVEL_BAR_MIN) / (LEVEL_BAR_MAX - LEVEL_BAR_MIN);
                if fraction < 0.0 {
                    self.level_bar.set_value(0.0);
                } else if fraction > 1.0 {
                    self.level_bar.set_value(1.0);
                } else {
                    self.level_bar.set_value(fraction.into());
                }

                self.bmi_label.set_markup(&crate::core::i18n_f(
                    "<small>Current BMI: {}</small>",
                    &[&format!("{bmi:.2}", bmi = current_bmi)],
                ));
            }
        }
    }
}

glib::wrapper! {
    pub struct HealthBMILevelBar(ObjectSubclass<imp::HealthBMILevelBar>)
        @extends gtk::Widget;
}

impl HealthBMILevelBar {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create HealthBMILevelBar")
    }

    fn get_priv(&self) -> &imp::HealthBMILevelBar {
        imp::HealthBMILevelBar::from_instance(self)
    }

    imp_getter_setter!(height, Length);
    imp_getter_setter!(unitsystem, Unitsystem);
    imp_getter_setter!(weight, Mass);
}
