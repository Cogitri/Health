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

use gtk::gdk::RGBA;
use gtk::gio::subclass::prelude::*;
use gtk::glib::{self};
use gtk::prelude::*;

mod imp {
    use gtk::gdk::prelude::GdkCairoContextExt;
    use gtk::gdk::RGBA;
    use gtk::glib::{self};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use std::{cell::RefCell, f64::consts::PI};

    pub struct ColorCircleMut {
        pub color: RGBA,
    }

    pub struct ColorCircle {
        pub inner: RefCell<ColorCircleMut>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ColorCircle {
        const NAME: &'static str = "HealthColorCircle";
        type ParentType = gtk::Widget;
        type Type = super::ColorCircle;

        fn new() -> Self {
            Self {
                inner: RefCell::new(ColorCircleMut {
                    color: gtk::gdk::RGBA::builder()
                        .red(0.0)
                        .green(0.0)
                        .blue(0.0)
                        .alpha(0.0)
                        .build(),
                }),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }
    }
    impl ObjectImpl for ColorCircle {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
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
                ))
                .unwrap();
            let width = f64::from(widget.width());
            let height = f64::from(widget.height());
            let radius = height * 0.3;
            cr.set_line_width(2.5);
            GdkCairoContextExt::set_source_rgba(&cr, &self.inner.borrow().color);
            cr.arc(width / 2.0, height / 2.0, radius, 0.0, 2.0 * PI);
            cr.stroke_preserve()
                .expect("Couldn't stroke on Cairo Context");
            cr.fill().expect("Couldn't fill on Cairo Context");
            cr.stroke().expect("Couldn't stroke on Cairo Context");
            cr.save().unwrap();
        }
    }
}
glib::wrapper! {
    /// A Widget for visualizing the color in legend table.
    pub struct ColorCircle(ObjectSubclass<imp::ColorCircle>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ColorCircle {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ColorCircle")
    }

    pub fn set_color(&self, color: RGBA) {
        let self_ = self.imp();
        self_.inner.borrow_mut().color = color;
        self.queue_draw();
    }

    fn imp(&self) -> &imp::ColorCircle {
        imp::ColorCircle::from_instance(self)
    }
}
