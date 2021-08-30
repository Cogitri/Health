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

use gtk::gdk::RGBA;
use gtk::glib::{self};
use gtk::subclass::prelude::*;

mod imp {
    use crate::widgets::ColorCircle;
    use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};

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
        type ParentType = gtk::Widget;
        type Type = super::LegendRow;

        fn class_init(klass: &mut Self::Class) {
            klass.set_layout_manager_type::<gtk::BinLayout>();
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl WidgetImpl for LegendRow {}
    impl BoxImpl for LegendRow {}
    impl ObjectImpl for LegendRow {
        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }
}

glib::wrapper! {
    /// [LegendRow] is a Widget that shows a colored circle next to the activity name.
    pub struct LegendRow(ObjectSubclass<imp::LegendRow>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl LegendRow {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create LegendRow")
    }

    pub fn set_legend_row(&self, color: RGBA, activity_name: String) {
        let self_ = self.imp();
        self_.activity_name.set_label(&activity_name);
        self_.color_circle.set_color(color);
    }

    fn imp(&self) -> &imp::LegendRow {
        imp::LegendRow::from_instance(self)
    }
}
