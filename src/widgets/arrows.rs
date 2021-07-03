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

use gtk::glib::{self};
use gtk::{gio::subclass::prelude::*, prelude::*};

mod imp {
    use gtk::gdk::prelude::GdkCairoContextExt;
    use gtk::glib::{self};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use std::cell::RefCell;

    #[derive(Debug)]
    pub struct ArrowsMut {
        pub weight: f32,
        pub difference: f32,
    }

    pub struct Arrows {
        pub inner: RefCell<ArrowsMut>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Arrows {
        const NAME: &'static str = "HealthArrows";
        type ParentType = gtk::Widget;
        type Type = super::Arrows;

        fn new() -> Self {
            Self {
                inner: RefCell::new(ArrowsMut {
                    weight: 0.0,
                    difference: 0.0,
                }),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }
    }
    impl ObjectImpl for Arrows {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }
    impl WidgetImpl for Arrows {
        fn snapshot(&self, widget: &Self::Type, snapshot: &gtk::Snapshot) {
            if self.inner.borrow().difference != 0.0 {
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
                let orientation: bool = self.inner.borrow().difference > 0.0;

                cr.set_line_width(2.5);

                let style_context = widget.style_context();
                let shaded = style_context.lookup_color("blue").unwrap();
                GdkCairoContextExt::set_source_rgba(&cr, &shaded);

                let arrowhead_position = if orientation {
                    height * 0.1
                } else {
                    height * 0.85
                };
                let arrowhead_size = if orientation {
                    width / 12.0_f64
                } else {
                    -width / 12.0_f64
                };
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
                cr.stroke().expect("Couldn't stroke on Cairo Context");
                cr.save().unwrap();
            }
        }
    }
}
glib::wrapper! {
    /// A View for visualizing the development of data over time.
    pub struct Arrows(ObjectSubclass<imp::Arrows>)
        @extends gtk::Widget;
}

impl Arrows {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Arrows")
    }

    pub fn set_weight(&self, weight: f32) {
        let self_ = self.imp();
        self_.inner.borrow_mut().weight = weight;
        self.queue_draw();
    }

    pub fn set_weight_difference(&self, weight_difference: f32) {
        let self_ = self.imp();
        self_.inner.borrow_mut().difference = weight_difference;
        self.queue_draw();
    }

    fn imp(&self) -> &imp::Arrows {
        imp::Arrows::from_instance(self)
    }
}
