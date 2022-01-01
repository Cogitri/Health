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
    use gtk::{glib, subclass::prelude::*};
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
                vec![glib::ParamSpecString::new(
                    "weight-change",
                    "weight-change",
                    "weight-change",
                    Some("no_change"),
                    glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT,
                )]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "weight-change" => {
                    self.weight_change
                        .set(WeightChange::from_str(&value.get::<String>().unwrap()).unwrap());
                    obj.queue_draw();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "weight-change" => self.weight_change.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for Arrows {
        fn snapshot(&self, widget: &Self::Type, snapshot: &gtk::Snapshot) {
            let cr = snapshot
                .append_cairo(&gtk::graphene::Rect::new(
                    0.0,
                    0.0,
                    widget.width() as f32,
                    widget.height() as f32,
                ));

            let width = f64::from(widget.width());
            let height = f64::from(widget.height());
            let weight_change = self.weight_change.get();

            cr.set_line_width(2.5);

            let style_context = widget.style_context();
            let shaded = style_context.lookup_color("blue").unwrap();
            GdkCairoContextExt::set_source_rgba(&cr, &shaded);

            let (arrowhead_position, arrowhead_size) = match weight_change {
                WeightChange::Down => (height * 0.85, -width / 12.0_f64),
                WeightChange::Up => (height * 0.1, width / 12.0_f64),
                WeightChange::NoChange => (width * 0.9, -width / 12.0_f64),
            };
            match weight_change {
                WeightChange::Down | WeightChange::Up => {
                    cr.move_to(width / 2.0, height * 0.1);
                    cr.line_to(width / 2.0, height * 0.85);
                    cr.move_to(width / 2.0, arrowhead_position);
                    cr.line_to(
                        width / 2.0 - arrowhead_size,
                        arrowhead_position + arrowhead_size,
                    );
                    cr.move_to(width / 2.0, arrowhead_position);
                    cr.line_to(
                        width / 2.0 + arrowhead_size,
                        arrowhead_position + arrowhead_size,
                    );
                }
                WeightChange::NoChange => {
                    cr.move_to(width - width * 0.85, height / 2.0);
                    cr.line_to(width - width * 0.1, height / 2.0);
                    cr.move_to(arrowhead_position, height / 2.0);
                    cr.line_to(
                        arrowhead_size + arrowhead_position,
                        height / 2.0 - arrowhead_size,
                    );
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
        glib::Object::new(&[]).expect("Failed to create Arrows")
    }

    pub fn set_weight_change(&self, change: WeightChange) {
        self.set_property("weight-change", change)
    }
}

#[cfg(test)]
mod test {
    use super::Arrows;
    use crate::utils::init_gtk;

    #[test]
    fn new() {
        init_gtk();
        Arrows::new();
    }
}
