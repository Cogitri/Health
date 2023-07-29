/* legend_row.rs
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

use gtk::{
    gdk,
    glib::{self, prelude::*},
};

mod imp {
    use crate::widgets::ColorCircle;
    use adw::{prelude::*, subclass::prelude::*};
    use gtk::{gdk, glib, CompositeTemplate};

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/legend_row.ui")]
    pub struct LegendRow {
        #[template_child]
        pub activity_name: TemplateChild<gtk::Label>,
        #[template_child]
        pub color_circle: TemplateChild<ColorCircle>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LegendRow {
        const NAME: &'static str = "HealthLegendRow";
        type ParentType = adw::Bin;
        type Type = super::LegendRow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for LegendRow {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecString::builder("activity-name").build(),
                    glib::ParamSpecBoxed::builder::<gdk::RGBA>("color").build(),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "activity-name" => self.activity_name.set_label(value.get().unwrap()),
                "color" => self.color_circle.set_color(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "activity-name" => self.activity_name.label().to_value(),
                "color" => self.color_circle.color().to_value(),
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for LegendRow {}
    impl BinImpl for LegendRow {}
}

glib::wrapper! {
    /// [LegendRow] is a Widget that shows a colored circle next to the activity name.
    pub struct LegendRow(ObjectSubclass<imp::LegendRow>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl LegendRow {
    pub fn activity_name(&self) -> String {
        self.property("activity-name")
    }

    pub fn color(&self) -> gdk::RGBA {
        self.property("color")
    }

    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn set_activity_name(&self, activity_name: &str) {
        self.set_property("activity-name", activity_name)
    }

    pub fn set_color(&self, color: gdk::RGBA) {
        self.set_property("color", color)
    }
}

#[cfg(test)]
mod test {
    use super::LegendRow;
    use crate::utils::init_gtk;

    #[gtk::test]
    fn new() {
        init_gtk();
        LegendRow::new();
    }
}
