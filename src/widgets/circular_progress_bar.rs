/* circular_progress_bar.rs
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

mod imp {
    use adw::{prelude::*, subclass::prelude::*};
    use gtk::glib;
    use std::{cell::RefCell, f64::consts::PI};

    pub struct CircularProgressBarMut {
        pub step_goal: u32,
        pub step_count: u32,
    }
    pub struct CircularProgressBar {
        pub inner: RefCell<CircularProgressBarMut>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CircularProgressBar {
        const NAME: &'static str = "HealthCircularProgressBar";
        type ParentType = adw::Bin;
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
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().set_size_request(75, 75);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecUInt::builder("step-count").build(),
                    glib::ParamSpecUInt::builder("step-goal").build(),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let obj = self.obj();

            match pspec.name() {
                "step-count" => {
                    self.inner.borrow_mut().step_count = value.get().unwrap();
                    obj.queue_draw();
                }
                "step-goal" => {
                    self.inner.borrow_mut().step_goal = value.get().unwrap();
                    obj.queue_draw();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "step-count" => self.inner.borrow().step_count.to_value(),
                "step-goal" => self.inner.borrow().step_goal.to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for CircularProgressBar {
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
            let radius = width * 0.21;

            cr.set_line_width(2.5);
            let style_manager = adw::StyleManager::default();
            let shaded = style_manager.accent_color_rgba();
            let mut unshaded = shaded;
            unshaded.set_alpha(0.25);
            GdkCairoContextExt::set_source_color(&cr, &unshaded);
            cr.move_to(width / 2.0, height / 2.0 - radius);
            cr.arc(width / 2.0, height / 2.0, radius, -0.5 * PI, 1.5 * PI);
            cr.stroke().expect("Couldn't stroke on Cairo Context");
            GdkCairoContextExt::set_source_color(&cr, &shaded);
            if self.inner.borrow().step_goal != 0 {
                cr.arc(
                    width / 2.0,
                    height / 2.0,
                    radius,
                    -0.5 * PI,
                    (f64::from(self.inner.borrow().step_count)
                        / f64::from(self.inner.borrow().step_goal))
                        * 2.0
                        * PI
                        - 0.5 * PI,
                );
            }
            cr.stroke().expect("Couldn't stroke on Cairo Context");
            cr.save().unwrap();
        }
    }
    impl BinImpl for CircularProgressBar {}
}

glib::wrapper! {
    /// A View for visualizing the development of data over time.
    pub struct CircularProgressBar(ObjectSubclass<imp::CircularProgressBar>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl CircularProgressBar {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn set_step_count(&self, step_count: u32) {
        self.set_property("step-count", step_count)
    }

    pub fn set_step_goal(&self, step_goal: u32) {
        self.set_property("step-goal", step_goal)
    }

    pub fn step_count(&self) -> u32 {
        self.property("step-count")
    }

    pub fn step_goal(&self) -> u32 {
        self.property("step-goal")
    }
}

#[cfg(test)]
mod test {
    use super::CircularProgressBar;
    use crate::utils::init_gtk;

    #[gtk::test]
    fn new() {
        init_gtk();
        CircularProgressBar::new();
    }
}
