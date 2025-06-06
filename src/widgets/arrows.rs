/* arrows.rs
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

use gtk::{glib, prelude::*};

use crate::model::WeightChange;

mod imp {
    use crate::model::WeightChange;
    use adw::{prelude::*, subclass::prelude::*};
    use gtk::glib;
    use std::{cell::Cell, str::FromStr};

    #[derive(Debug, Default)]
    pub struct Arrows {
        pub weight_change: Cell<WeightChange>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Arrows {
        const NAME: &'static str = "HealthArrows";
        type ParentType = adw::Bin;
        type Type = super::Arrows;
    }

    impl ObjectImpl for Arrows {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecString::builder("weight-change")
                    .default_value(Some("no_change"))
                    .construct()
                    .readwrite()
                    .build()]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let obj = self.obj();

            match pspec.name() {
                "weight-change" => {
                    self.weight_change
                        .set(WeightChange::from_str(value.get::<&str>().unwrap()).unwrap());
                    obj.queue_draw();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "weight-change" => self.weight_change.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for Arrows {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();
            let cr = snapshot.append_cairo(&gtk::graphene::Rect::new(
                0.0,
                0.0,
                widget.width() as f32,
                widget.height() as f32,
            ));

            let width = f64::from(widget.width());
            let height = f64::from(widget.height());
            let weight_change = self.weight_change.get();

            const STROKE_WIDTH: f64 = 4.5;
            cr.set_line_width(STROKE_WIDTH);

            let style_manager = adw::StyleManager::default();
            let shaded = style_manager.accent_color_rgba();
            GdkCairoContextExt::set_source_color(&cr, &shaded);

            const HEAD_LENGTH: f64 = 12.0_f64;
            let (arrowhead_position, arrowhead_size, tip) = match weight_change {
                WeightChange::Down => (height * 0.85, -width / HEAD_LENGTH, STROKE_WIDTH / 4.0),
                WeightChange::Up => (height * 0.1, width / HEAD_LENGTH, -STROKE_WIDTH / 4.0),
                WeightChange::NoChange => (width * 0.9, -width / HEAD_LENGTH, STROKE_WIDTH / 4.0),
            };
            match weight_change {
                WeightChange::Down | WeightChange::Up => {
                    // body
                    cr.move_to(width / 2.0, height * 0.1);
                    cr.line_to(width / 2.0, height * 0.85);
                    // left head
                    cr.move_to(width / 2.0 - tip, arrowhead_position + tip);
                    cr.line_to(
                        width / 2.0 - arrowhead_size,
                        arrowhead_position + arrowhead_size,
                    );
                    // right head
                    cr.move_to(width / 2.0 + tip, arrowhead_position + tip);
                    cr.line_to(
                        width / 2.0 + arrowhead_size,
                        arrowhead_position + arrowhead_size,
                    );
                }
                WeightChange::NoChange => {
                    // body
                    cr.move_to(width - width * 0.85, height / 2.0);
                    cr.line_to(width - width * 0.1, height / 2.0);
                    // upper head
                    cr.move_to(arrowhead_position, height / 2.0);
                    cr.line_to(
                        arrowhead_size + arrowhead_position,
                        height / 2.0 - arrowhead_size,
                    );
                    // lower head
                    cr.move_to(arrowhead_position, height / 2.0);
                    cr.line_to(
                        arrowhead_position + arrowhead_size,
                        height / 2.0 + arrowhead_size,
                    );
                }
            }
            cr.stroke().expect("Couldn't stroke on Cairo Context");
            cr.save().unwrap();
        }
    }
    impl BinImpl for Arrows {}
}

glib::wrapper! {
    /// A View for visualizing the development of data over time.
    pub struct Arrows(ObjectSubclass<imp::Arrows>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Arrows {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn set_weight_change(&self, change: WeightChange) {
        self.set_property("weight-change", &change)
    }
}

#[cfg(test)]
mod test {
    use super::Arrows;
    use crate::utils::init_gtk;

    #[gtk::test]
    fn new() {
        init_gtk();
        Arrows::new();
    }
}
