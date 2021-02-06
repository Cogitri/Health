/* graph_view.rs
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

use chrono::{Date, FixedOffset};
use gio::subclass::prelude::ObjectSubclass;

#[derive(Debug, Clone)]
pub struct Point {
    pub date: Date<FixedOffset>,
    pub value: f32,
}

mod imp {
    use super::Point;
    use chrono::Local;
    use glib::{clone, subclass};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use std::{cell::RefCell, convert::TryInto, f64::consts::PI};

    static HALF_X_PADDING: f32 = 40.0;
    static HALF_Y_PADDING: f32 = 30.0;

    #[derive(Debug)]
    struct HoverPoint {
        pub point: Point,
        pub x: f32,
        pub y: f32,
    }

    struct GraphViewMut {
        pub biggest_value: f32,
        pub height: f32,
        pub hover_func: Option<Box<dyn Fn(&Point) -> String>>,
        pub hover_max_pointer_deviation: u32,
        pub hover_point: Option<HoverPoint>,
        pub limit: Option<f32>,
        pub limit_label: Option<String>,
        pub points: Vec<Point>,
        pub scale_x: f32,
        pub scale_y: f32,
        pub width: f32,
        pub x_lines_interval: f32,
    }

    pub struct GraphView {
        inner: RefCell<GraphViewMut>,
    }

    impl ObjectSubclass for GraphView {
        const NAME: &'static str = "HealthGraphView";
        type ParentType = gtk::Widget;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::GraphView;
        type Interfaces = ();

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                inner: RefCell::new(GraphViewMut {
                    biggest_value: 0.1,
                    height: 0.0,
                    hover_func: None,
                    hover_max_pointer_deviation: 8,
                    hover_point: None,
                    limit: None,
                    limit_label: None,
                    points: Vec::new(),
                    scale_x: 0.0,
                    scale_y: 0.0,
                    width: 0.0,
                    x_lines_interval: 500.0,
                }),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }
    }

    impl WidgetImpl for GraphView {
        fn snapshot(&self, widget: &Self::Type, snapshot: &gtk::Snapshot) {
            let mut inner = self.inner.borrow_mut();

            inner.height = widget.get_height() as f32 - HALF_Y_PADDING * 2.0;
            inner.width = widget.get_width() as f32 - HALF_X_PADDING * 2.0;

            let biggest_value = if inner.points.is_empty() {
                inner.scale_x = inner.width;
                inner.scale_y = inner.height / 10000.0;
                0.1
            } else {
                //Round up to 500, the graph looks a bit odd if we draw lines at biggest_value / 4 instead of
                // using even numbers
                let biggest_value = inner.biggest_value + inner.x_lines_interval
                    - inner.biggest_value % inner.x_lines_interval;

                inner.scale_x = inner.width / inner.points.len() as f32;
                inner.scale_y = inner.height / biggest_value;

                biggest_value
            };

            let cr = snapshot
                .append_cairo(&gtk::graphene::Rect::new(
                    0.0,
                    0.0,
                    widget.get_width() as f32,
                    widget.get_height() as f32,
                ))
                .unwrap();
            let style_context = widget.get_style_context();
            let outline_color = style_context.get_color();
            cr.set_source_rgba(
                outline_color.red.into(),
                outline_color.green.into(),
                outline_color.blue.into(),
                0.5,
            );
            /*
                Draw outlines
            */
            cr.save().unwrap();

            for i in 0..5 {
                let mul = inner.height / 4.0;
                cr.move_to(
                    (inner.width + HALF_Y_PADDING).into(),
                    (mul * i as f32 + HALF_Y_PADDING).into(),
                );
                cr.line_to(
                    HALF_X_PADDING.into(),
                    (mul * i as f32 + HALF_Y_PADDING).into(),
                );
                let layout = widget.create_pango_layout(Some(
                    &((biggest_value / 4.0 * (4 - i) as f32) as u32).to_string(),
                ));
                let (_, extents) = layout.get_extents();

                cr.rel_move_to(0.0, pango::units_to_double(extents.height) * -1.0);
                pangocairo::show_layout(&cr, &layout);
            }

            cr.stroke();
            cr.restore().unwrap();

            /*
                Draw X Ticks (dates)
            */

            cr.save().unwrap();

            for (i, point) in inner.points.iter().enumerate() {
                let layout =
                    widget.create_pango_layout(Some(&format!("{}", point.date.format("%x"))));
                let (_, extents) = layout.get_extents();

                cr.move_to(
                    f64::from(i as f32 * inner.scale_x + HALF_X_PADDING)
                        - pango::units_to_double(extents.width) / 2.0,
                    f64::from(inner.height + HALF_Y_PADDING * 1.5)
                        - pango::units_to_double(extents.height) / 2.0,
                );
                pangocairo::show_layout(&cr, &layout);
            }

            cr.stroke();
            cr.restore().unwrap();

            /*
                Draw limit/goal (if any)
            */
            if let Some(limit) = inner.limit {
                cr.save().unwrap();

                cr.set_dash(&[10.0, 5.0], 0.0);
                cr.move_to(
                    f64::from(HALF_X_PADDING),
                    f64::from(inner.height - limit * inner.scale_y + HALF_Y_PADDING),
                );
                let layout = widget.create_pango_layout(inner.limit_label.as_deref());
                pangocairo::show_layout(&cr, &layout);
                cr.line_to(
                    f64::from(inner.width + HALF_X_PADDING),
                    f64::from(inner.height - limit * inner.scale_y + HALF_Y_PADDING),
                );

                cr.stroke();
                cr.restore().unwrap();
            }

            if inner.points.is_empty() {
                return;
            }

            /*
                Draw a point for each datapoint
            */
            cr.save().unwrap();

            cr.set_source_rgba(0.0, 174.0, 174.0, 1.0);
            cr.set_line_width(4.0);
            for (i, point) in inner.points.iter().enumerate() {
                let x = f64::from(i as f32 * inner.scale_x + HALF_X_PADDING);
                let y = f64::from(inner.height - point.value * inner.scale_y + HALF_Y_PADDING);

                cr.move_to(x, y);
                cr.arc(x, y, 2.0, 0.0, 2.0 * PI);
            }

            cr.stroke();
            cr.restore().unwrap();

            /*
                Draw the graph itself
            */
            cr.save().unwrap();
            cr.set_source_rgba(0.0, 174.0, 174.0, 0.8);
            cr.move_to(
                f64::from(HALF_X_PADDING),
                f64::from(
                    inner.height - inner.points.get(0).unwrap().value * inner.scale_y
                        + HALF_Y_PADDING,
                ),
            );

            for (i, point) in inner.points.iter().enumerate() {
                let next_value = if (i + 1) >= inner.points.len() {
                    inner.points.get(i).unwrap().value
                } else {
                    inner.points.get(i + 1).unwrap().value
                };
                let smoothness_factor = 0.5;

                cr.curve_to(
                    f64::from((i as f32 + smoothness_factor) * inner.scale_x + HALF_X_PADDING),
                    f64::from(inner.height - point.value * inner.scale_y + HALF_Y_PADDING),
                    f64::from(
                        ((i + 1) as f32 - smoothness_factor) * inner.scale_x + HALF_X_PADDING,
                    ),
                    f64::from(inner.height - next_value * inner.scale_y + HALF_Y_PADDING),
                    f64::from((i + 1) as f32 * inner.scale_x + HALF_X_PADDING),
                    f64::from(inner.height - next_value * inner.scale_y + HALF_Y_PADDING),
                );
            }

            cr.stroke();
            cr.restore().unwrap();

            if let Some(hover_func) = &inner.hover_func {
                if let Some(hover_point) = &inner.hover_point {
                    let layout = widget.create_pango_layout(Some(&hover_func(&hover_point.point)));
                    let (_, extents) = layout.get_extents();
                    let radius = pango::units_to_double(extents.height) / 5.0;
                    let degrees = PI / 180.0;
                    let padding = 12.0;

                    // If the tooltip doesn't fit to the right side of the point, draw it on the left side of the point
                    let x_delta = if (hover_point.x
                        + pango::units_to_double(extents.width) as f32
                        + padding * 2.0)
                        > inner.width
                    {
                        (pango::units_to_double(extents.width) as f32 + padding * 3.0) * -1.0
                    } else {
                        0.0
                    };

                    cr.new_sub_path();
                    cr.arc(
                        f64::from(hover_point.x + padding * 2.0 + x_delta)
                            + pango::units_to_double(extents.width)
                            - radius,
                        f64::from(hover_point.y - padding / 2.0)
                            - pango::units_to_double(extents.height) / 2.0
                            + radius,
                        radius,
                        -90.0 * degrees,
                        0.0,
                    );
                    cr.arc(
                        f64::from(hover_point.x + padding * 2.0 + x_delta)
                            + pango::units_to_double(extents.width)
                            - radius,
                        f64::from(hover_point.y + padding / 2.0)
                            + pango::units_to_double(extents.height) / 2.0
                            - radius,
                        radius,
                        0.0,
                        90.0 * degrees,
                    );
                    cr.arc(
                        f64::from(hover_point.x + padding + x_delta) + radius,
                        f64::from(hover_point.y + padding / 2.0)
                            + pango::units_to_double(extents.height) / 2.0
                            - radius,
                        radius,
                        90.0 * degrees,
                        180.0 * degrees,
                    );
                    cr.arc(
                        f64::from(hover_point.x + padding + x_delta) + radius,
                        f64::from(hover_point.y - padding / 2.0)
                            - pango::units_to_double(extents.height) / 2.0
                            + radius,
                        radius,
                        180.0 * degrees,
                        270.0 * degrees,
                    );
                    cr.close_path();
                    cr.set_source_rgba(0.0, 0.0, 0.0, 0.65);
                    cr.fill_preserve();

                    cr.move_to(
                        f64::from(hover_point.x + padding * 1.5 + x_delta),
                        f64::from(hover_point.y) - pango::units_to_double(extents.height) / 2.0,
                    );
                    cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
                    pangocairo::show_layout(&cr, &layout);
                    cr.stroke();
                }
            }
        }
    }

    impl ObjectImpl for GraphView {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.set_hexpand(true);
            obj.set_vexpand(true);
            let gesture_controller = gtk::GestureClick::new();
            gesture_controller.set_touch_only(true);
            gesture_controller.connect_pressed(clone!(@weak obj => move |c, _, x, y| GraphView::from_instance(&obj).on_motion_event(&obj, x, y, true, c)));
            obj.add_controller(&gesture_controller);

            let motion_controller = gtk::EventControllerMotion::new();
            motion_controller.connect_enter(clone!(@weak obj => move|c, x, y| GraphView::from_instance(&obj).on_motion_event(&obj, x, y, false, c)));
            motion_controller.connect_motion(clone!(@weak obj => move|c, x, y| GraphView::from_instance(&obj).on_motion_event(&obj, x, y, false, c)));
            obj.add_controller(&motion_controller);

            let mut inner = self.inner.borrow_mut();
            inner.hover_max_pointer_deviation = (8 * obj.get_scale_factor()).try_into().unwrap();
        }
    }

    impl GraphView {
        pub fn set_hover_func(
            &self,
            obj: &super::GraphView,
            hover_func: Option<Box<dyn Fn(&Point) -> String>>,
        ) {
            self.inner.borrow_mut().hover_func = hover_func;
            obj.queue_draw();
        }

        pub fn set_limit(&self, obj: &super::GraphView, limit: Option<f32>) {
            self.inner.borrow_mut().limit = limit;
            obj.queue_draw();
        }

        pub fn set_limit_label(&self, obj: &super::GraphView, label: Option<String>) {
            self.inner.borrow_mut().limit_label = label;
            obj.queue_draw();
        }

        pub fn set_x_lines_interval(&self, obj: &super::GraphView, interval: f32) {
            self.inner.borrow_mut().x_lines_interval = interval;
            obj.queue_draw();
        }

        pub fn set_points(&self, obj: &super::GraphView, points: Vec<Point>) {
            let layout = obj.create_pango_layout(Some(&format!("{}", Local::now().format("%x"))));
            let (_, extents) = layout.get_extents();
            let datapoint_width = pango::units_to_double(extents.width) + f64::from(HALF_X_PADDING);

            obj.set_size_request(
                (datapoint_width as usize * points.len())
                    .try_into()
                    .unwrap(),
                -1,
            );

            let mut inner = self.inner.borrow_mut();
            inner.biggest_value = points
                .iter()
                .max_by(|x, y| (x.value as u32).cmp(&(y.value as u32)))
                .map(|b| b.value)
                .unwrap();
            inner.points = points;
            obj.queue_draw();
        }

        fn on_motion_event(
            &self,
            obj: &super::GraphView,
            x: f64,
            y: f64,
            allow_touch: bool,
            controller: &impl IsA<gtk::EventController>,
        ) {
            let mut inner = self.inner.borrow_mut();
            let hover_max_pointer_deviation = inner.hover_max_pointer_deviation;

            let approx_matches = |num: f64, approx_range: f32| {
                num > (approx_range - hover_max_pointer_deviation as f32).into()
                    && num < (approx_range + hover_max_pointer_deviation as f32).into()
            };

            // Don't handle touch events, we do that via Gtk.GestureClick.
            if !allow_touch {
                if let Some(device) = controller.get_current_event_device() {
                    if device.get_source() == gdk::InputSource::Touchscreen {
                        return;
                    }
                }
            }

            let mut point_res = None;
            for (i, point) in inner.points.iter().enumerate() {
                let point_x = i as f32 * inner.scale_x + HALF_X_PADDING;
                let point_y = inner.height - point.value * inner.scale_y + HALF_Y_PADDING;

                if approx_matches(x, point_x) && approx_matches(y, point_y) {
                    point_res = Some(HoverPoint {
                        point: point.clone(),
                        x: point_x,
                        y: point_y,
                    });
                }
            }

            if let Some(point) = point_res {
                inner.hover_point = Some(point);
                obj.queue_draw();
            } else if inner.hover_point.is_some() {
                inner.hover_point = None;
                obj.queue_draw();
            }
        }
    }
}

glib::wrapper! {
    pub struct GraphView(ObjectSubclass<imp::GraphView>)
        @extends gtk::Widget;
}

impl GraphView {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create GraphView")
    }

    pub fn set_hover_func(&self, hover_func: Option<Box<dyn Fn(&Point) -> String>>) {
        imp::GraphView::from_instance(self).set_hover_func(self, hover_func);
    }

    pub fn set_limit(&self, limit: Option<f32>) {
        imp::GraphView::from_instance(self).set_limit(self, limit);
    }

    pub fn set_limit_label(&self, label: Option<String>) {
        imp::GraphView::from_instance(self).set_limit_label(self, label);
    }

    pub fn set_points(&self, points: Vec<Point>) {
        imp::GraphView::from_instance(self).set_points(self, points);
    }

    pub fn set_x_lines_interval(&self, interval: f32) {
        imp::GraphView::from_instance(self).set_x_lines_interval(self, interval);
    }
}
