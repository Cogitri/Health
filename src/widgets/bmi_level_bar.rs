/* bmi_level_bar.rs
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

use gio::subclass::prelude::*;
use uom::si::{
    f32::{Length, Mass},
    length::centimeter,
    mass::kilogram,
};

static LEVEL_BAR_MIN: f32 = 13.5;
static LEVEL_BAR_MAX: f32 = 30.0;

mod imp {
    use super::{LEVEL_BAR_MAX, LEVEL_BAR_MIN};
    use gtk::{prelude::*, subclass::prelude::*, CompositeTemplate};
    use std::cell::RefCell;
    use uom::si::{
        f32::{Length, Mass},
        length::centimeter,
        mass::kilogram,
    };

    #[derive(Debug)]
    pub struct BMILevelBarMut {
        pub height: Length,
        pub weight: Mass,
    }

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/dev/Cogitri/Health/ui/bmi_level_bar.ui")]
    pub struct BMILevelBar {
        pub inner: RefCell<BMILevelBarMut>,
        #[template_child]
        pub bmi_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub level_bar: TemplateChild<gtk::LevelBar>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BMILevelBar {
        const NAME: &'static str = "HealthBMILevelBar";
        type ParentType = gtk::Widget;
        type Type = super::BMILevelBar;

        fn new() -> Self {
            Self {
                inner: RefCell::new(BMILevelBarMut {
                    height: Length::new::<centimeter>(0.0),
                    weight: Mass::new::<kilogram>(0.0),
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

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for BMILevelBar {
        fn constructed(&self, obj: &Self::Type) {
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
    impl WidgetImpl for BMILevelBar {}
}

glib::wrapper! {
    /// A widget to visualise the BMI of the user.
    pub struct BMILevelBar(ObjectSubclass<imp::BMILevelBar>)
        @extends gtk::Widget;
}

impl BMILevelBar {
    pub fn get_height(&self) -> Length {
        self.get_priv().inner.borrow().height
    }

    pub fn get_weight(&self) -> Mass {
        self.get_priv().inner.borrow().weight
    }

    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create BMILevelBar")
    }

    pub fn set_height(&self, value: Length) {
        let self_ = self.get_priv();
        self_.inner.borrow_mut().height = value;
        self.recalcualte_bmi();
    }

    pub fn set_weight(&self, value: Mass) {
        let self_ = self.get_priv();
        self_.inner.borrow_mut().weight = value;
        self.recalcualte_bmi();
    }

    fn get_priv(&self) -> &imp::BMILevelBar {
        imp::BMILevelBar::from_instance(self)
    }

    fn recalcualte_bmi(&self) {
        let self_ = self.get_priv();

        let height = self_.inner.borrow().height.get::<centimeter>() as f32 / 100.0;
        let weight = self_.inner.borrow().weight.get::<kilogram>();
        if height != 0.0 && weight != 0.0 {
            let current_bmi = weight / (height * height);
            let fraction = (current_bmi - LEVEL_BAR_MIN) / (LEVEL_BAR_MAX - LEVEL_BAR_MIN);
            if fraction < 0.0 {
                self_.level_bar.set_value(0.0);
            } else if fraction > 1.0 {
                self_.level_bar.set_value(1.0);
            } else {
                self_.level_bar.set_value(fraction.into());
            }

            self_.bmi_label.set_markup(&crate::core::i18n_f(
                "<small>Current BMI: {}</small>",
                &[&format!("{bmi:.2}", bmi = current_bmi)],
            ));
        }
    }
}
