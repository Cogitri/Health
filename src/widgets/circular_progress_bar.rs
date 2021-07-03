/* graph_view.rs
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

use gtk::gio::subclass::prelude::*;
use gtk::glib::{self};
use gtk::prelude::*;

mod imp {
    use gtk::gdk::prelude::GdkCairoContextExt;
    use gtk::glib::{self};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use std::{cell::RefCell, f64::consts::PI};

    pub struct CircularProgressBarMut {
        pub step_goal: i64,
        pub step_count: i64,
    }

    pub struct CircularProgressBar {
        pub inner: RefCell<CircularProgressBarMut>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CircularProgressBar {
        const NAME: &'static str = "HealthCircularProgressBar";
        type ParentType = gtk::Widget;
        type Type = super::CircularProgressBar;

        fn new() -> Self {
            Self {
                inner: RefCell::new(CircularProgressBarMut {
                    step_goal: 0,
                    step_count: 0,
                }),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }
    }
    impl ObjectImpl for CircularProgressBar {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }
    impl WidgetImpl for CircularProgressBar {
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
            let radius = width * 0.21;

            cr.set_line_width(2.5);
            let style_context = widget.style_context();
            let unshaded = style_context.lookup_color("light-blue").unwrap();
            GdkCairoContextExt::set_source_rgba(&cr, &unshaded);
            cr.move_to(width / 2.0, height / 2.0 - radius);
            cr.arc(width / 2.0, height / 2.0, radius, -0.5 * PI, 1.5 * PI);
            cr.stroke().expect("Couldn't stroke on Cairo Context");
            let shaded = style_context.lookup_color("blue").unwrap();
            GdkCairoContextExt::set_source_rgba(&cr, &shaded);
            cr.arc(
                width / 2.0,
                height / 2.0,
                radius,
                -0.5 * PI,
                ((self.inner.borrow().step_count as f64) / self.inner.borrow().step_goal as f64)
                    * 2.0
                    * PI
                    - 0.5 * PI,
            );
            cr.stroke().expect("Couldn't stroke on Cairo Context");
            cr.save().unwrap();
        }
    }
}
glib::wrapper! {
    /// A View for visualizing the development of data over time.
    pub struct CircularProgressBar(ObjectSubclass<imp::CircularProgressBar>)
        @extends gtk::Widget;
}

impl CircularProgressBar {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create CircularProgressBar")
    }

    pub fn set_step_goal(&self, step_goal: i64) {
        let self_ = self.imp();
        self_.inner.borrow_mut().step_goal = step_goal;
        self.queue_draw();
    }

    pub fn set_step_count(&self, step_count: i64) {
        let self_ = self.imp();
        self_.inner.borrow_mut().step_count = step_count;
        self.queue_draw();
    }

    fn imp(&self) -> &imp::CircularProgressBar {
        imp::CircularProgressBar::from_instance(self)
    }
}
