/* color_circle.rs
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

use gtk::{gdk, glib, prelude::*};

mod imp {
    use adw::{prelude::*, subclass::prelude::*};
    use gtk::{gdk, glib, subclass::prelude::*};
    use std::{cell::RefCell, f64::consts::PI};

    pub struct ColorCircle {
        pub color: RefCell<gdk::RGBA>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ColorCircle {
        const NAME: &'static str = "HealthColorCircle";
        type ParentType = adw::Bin;
        type Type = super::ColorCircle;

        fn new() -> Self {
            Self {
                color: RefCell::new(
                    gdk::RGBA::builder()
                        .red(0.0)
                        .green(0.0)
                        .blue(0.0)
                        .alpha(0.0)
                        .build(),
                ),
            }
        }
    }

    impl ObjectImpl for ColorCircle {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecBoxed::new(
                    "color",
                    "color",
                    "color",
                    gdk::RGBA::static_type(),
                    glib::ParamFlags::READWRITE,
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
                "color" => {
                    self.color.replace(value.get().unwrap());
                    obj.queue_draw();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "color" => self.color.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for ColorCircle {
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
            let radius = height * 0.3;
            cr.set_line_width(2.5);
            GdkCairoContextExt::set_source_rgba(&cr, &self.color.borrow());
            cr.arc(width / 2.0, height / 2.0, radius, 0.0, 2.0 * PI);
            cr.stroke_preserve()
                .expect("Couldn't stroke on Cairo Context");
            cr.fill().expect("Couldn't fill on Cairo Context");
            cr.stroke().expect("Couldn't stroke on Cairo Context");
            cr.save().unwrap();
        }
    }

    impl BinImpl for ColorCircle {}
}

glib::wrapper! {
    /// A Widget for visualizing the color in legend table.
    pub struct ColorCircle(ObjectSubclass<imp::ColorCircle>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ColorCircle {
    pub fn color(&self) -> gdk::RGBA {
        self.property("color")
    }

    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ColorCircle")
    }
    pub fn set_color(&self, color: gdk::RGBA) {
        self.set_property("color", color);
    }
}

#[cfg(test)]
mod test {
    use super::ColorCircle;
    use crate::utils::init_gtk;

    #[test]
    fn new() {
        init_gtk();
        ColorCircle::new();
    }
}
