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

use gtk::{gio::subclass::prelude::*, glib};
use uom::si::{
    f32::{Length, Mass},
    length::centimeter,
    mass::kilogram,
};

static LEVEL_BAR_MIN: f32 = 13.5;
static LEVEL_BAR_MAX: f32 = 30.0;

mod imp {
    use super::{LEVEL_BAR_MAX, LEVEL_BAR_MIN};
    use gtk::{
        glib,
        {prelude::*, subclass::prelude::*, CompositeTemplate},
    };
    use std::cell::RefCell;
    use uom::si::f32::{Length, Mass};

    #[derive(Debug, Default)]
    pub struct BmiLevelBarMut {
        pub height: Length,
        pub weight: Mass,
    }

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/bmi_level_bar.ui")]
    pub struct BmiLevelBar {
        pub inner: RefCell<BmiLevelBarMut>,
        #[template_child]
        pub bmi_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub level_bar: TemplateChild<gtk::LevelBar>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BmiLevelBar {
        const NAME: &'static str = "HealthBMILevelBar";
        type ParentType = gtk::Widget;
        type Type = super::BmiLevelBar;

        fn class_init(klass: &mut Self::Class) {
            klass.set_layout_manager_type::<gtk::BoxLayout>();
            klass.set_accessible_role(gtk::AccessibleRole::Meter);
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for BmiLevelBar {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.layout_manager()
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
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }
    impl WidgetImpl for BmiLevelBar {}
}

glib::wrapper! {
    /// A widget to visualise the BMI of the user.
    pub struct BmiLevelBar(ObjectSubclass<imp::BmiLevelBar>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl BmiLevelBar {
    /// Get the height of the user.
    pub fn height(&self) -> Length {
        self.imp().inner.borrow().height
    }

    /// Get the weight of the user.
    pub fn weight(&self) -> Mass {
        self.imp().inner.borrow().weight
    }

    /// Create a new [BmiLevelBar].
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create BmiLevelBar")
    }

    /// Set the height of the user.
    pub fn set_height(&self, value: Length) {
        let self_ = self.imp();
        self_.inner.borrow_mut().height = value;
        self.recalcualte_bmi();
    }

    /// Set the height of the user.
    pub fn set_weight(&self, value: Mass) {
        let self_ = self.imp();
        self_.inner.borrow_mut().weight = value;
        self.recalcualte_bmi();
    }

    fn imp(&self) -> &imp::BmiLevelBar {
        imp::BmiLevelBar::from_instance(self)
    }

    fn recalcualte_bmi(&self) {
        let self_ = self.imp();

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

#[cfg(test)]
mod test {
    use super::BmiLevelBar;
    use uom::si::{
        f32::{Length, Mass},
        length::meter,
        mass::kilogram,
    };

    #[test]
    fn recalcualte_bmi() {
        crate::utils::init_gtk();

        let bar = BmiLevelBar::new();
        bar.set_height(Length::new::<meter>(1.85));
        bar.set_weight(Mass::new::<kilogram>(70.0));

        let self_ = bar.imp();
        assert_eq!(self_.level_bar.value(), 0.4213869571685791);
        assert_eq!(
            self_.bmi_label.label().as_str(),
            crate::core::i18n_f(
                "<small>Current BMI: {}</small>",
                &[&format!("{bmi:.2}", bmi = 20.45)],
            )
            .as_str()
        );
    }
}
