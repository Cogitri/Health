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

use crate::model::FnBoxedPoint;
use crate::prelude::*;
use gtk::{gdk, gio::subclass::prelude::*, glib, pango, prelude::*};
use std::convert::TryInto;

/// A [Point] describes a single datapoint in a [GraphView]
#[derive(Debug, Clone)]
pub struct Point {
    pub date: glib::DateTime,
    pub value: f32,
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.date.equals_date(&other.date)
    }
}

static HALF_X_PADDING: f32 = 40.0;
static HALF_Y_PADDING: f32 = 30.0;

mod imp {
    use super::{Point, HALF_X_PADDING, HALF_Y_PADDING};
    use crate::model::FnBoxedPoint;
    use crate::prelude::*;
    use gtk::{
        gdk::prelude::*,
        glib::{self, clone},
        pango,
        prelude::*,
        subclass::prelude::*,
    };
    use std::{cell::RefCell, convert::TryInto, f64::consts::PI};

    #[derive(Debug)]
    pub struct HoverPoint {
        pub point: Point,
        pub x: f32,
        pub y: f32,
    }

    pub struct GraphViewMut {
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
        pub inner: RefCell<GraphViewMut>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GraphView {
        const NAME: &'static str = "HealthGraphView";
        type ParentType = gtk::Widget;
        type Type = super::GraphView;

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
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let mut inner = self.inner.borrow_mut();
            let widget = self.obj();

            inner.height = widget.height() as f32 - HALF_Y_PADDING * 2.0;
            inner.width = widget.width() as f32 - HALF_X_PADDING * 2.0;

            let biggest_value = if inner.points.is_empty() {
                inner.scale_x = inner.width;
                inner.scale_y = inner.height / 10000.0;
                0.1
            } else {
                // Round up to 500, the graph looks a bit odd if we draw lines at biggest_value / 4 instead of
                // using even numbers
                let biggest_value = inner.biggest_value + inner.x_lines_interval
                    - inner.biggest_value % inner.x_lines_interval;

                // If we have more than one points, we don't want an empty point at the end of the graph
                inner.scale_x = if inner.points.len() > 1 {
                    inner.width / (inner.points.len() - 1) as f32
                } else {
                    inner.width as f32
                };
                inner.scale_y = inner.height / biggest_value;

                biggest_value
            };

            let cr = snapshot.append_cairo(&gtk::graphene::Rect::new(
                0.0,
                0.0,
                widget.width() as f32,
                widget.height() as f32,
            ));
            let style_context = widget.style_context();
            let background_color = style_context.lookup_color("insensitive_fg_color").unwrap();

            GdkCairoContextExt::set_source_rgba(&cr, &background_color);
            /*
                Draw outlines
            */
            cr.save().unwrap();
            cr.set_line_width(0.5);
            cr.set_dash(&[10.0, 5.0], 0.0);

            for i in 0..4 {
                let mul = inner.height / 4.0;
                cr.move_to(
                    f64::from(inner.width + HALF_Y_PADDING),
                    f64::from(mul * i as f32 + HALF_Y_PADDING),
                );
                cr.line_to(
                    f64::from(HALF_X_PADDING),
                    f64::from(mul * i as f32 + HALF_Y_PADDING),
                );
                let layout = widget.create_pango_layout(Some(
                    &((biggest_value / 4.0 * (4 - i) as f32) as u32).to_string(),
                ));
                let (_, extents) = layout.extents();

                cr.rel_move_to(0.0, pango::units_to_double(extents.height()) * -1.0);
                pangocairo::show_layout(&cr, &layout);
            }

            cr.stroke().expect("Couldn't stroke on Cairo Context");
            cr.restore().unwrap();

            /*
                Draw X Ticks (dates)
            */

            cr.save().unwrap();

            for (i, point) in inner.points.iter().enumerate() {
                let layout = widget.create_pango_layout(Some(&point.date.format_local()));
                let (_, extents) = layout.extents();

                cr.move_to(
                    f64::from(i as f32 * inner.scale_x + HALF_X_PADDING)
                        - pango::units_to_double(extents.width()) / 2.0,
                    f64::from(inner.height + HALF_Y_PADDING * 1.5)
                        - pango::units_to_double(extents.height()) / 2.0,
                );
                pangocairo::show_layout(&cr, &layout);
            }

            cr.stroke().expect("Couldn't stroke on Cairo Context");
            cr.restore().unwrap();

            /*
                Draw limit/goal (if any)
            */
            if let Some(limit) = inner.limit {
                cr.save().unwrap();

                let graph_color = style_context.lookup_color("success_color").unwrap();
                GdkCairoContextExt::set_source_rgba(&cr, &graph_color);

                cr.set_line_width(0.5);
                cr.set_dash(&[10.0, 5.0], 0.0);
                cr.move_to(
                    f64::from(inner.width + HALF_X_PADDING),
                    f64::from(inner.height - limit * inner.scale_y + HALF_Y_PADDING),
                );
                cr.line_to(
                    f64::from(HALF_X_PADDING),
                    f64::from(inner.height - limit * inner.scale_y + HALF_Y_PADDING),
                );

                let layout = widget.create_pango_layout(inner.limit_label.as_deref());
                let (_, extents) = layout.extents();
                cr.move_to(
                    f64::from(inner.width + HALF_X_PADDING)
                        - pango::units_to_double(extents.width()),
                    f64::from(inner.height - limit * inner.scale_y + HALF_Y_PADDING)
                        - pango::units_to_double(extents.height()),
                );
                pangocairo::show_layout(&cr, &layout);

                cr.stroke().expect("Couldn't stroke on Cairo Context");
                cr.restore().unwrap();
            }

            if inner.points.is_empty() {
                return;
            }

            /*
                Draw a point for each datapoint
            */
            cr.save().unwrap();

            let graph_color = style_context.lookup_color("accent_bg_color").unwrap();
            GdkCairoContextExt::set_source_rgba(&cr, &graph_color);
            cr.set_line_width(4.0);
            for (i, point) in inner.points.iter().enumerate() {
                let x = f64::from(i as f32 * inner.scale_x + HALF_X_PADDING);
                let y = f64::from(inner.height - point.value * inner.scale_y + HALF_Y_PADDING);

                cr.move_to(x, y);
                cr.arc(x, y, 2.0, 0.0, 2.0 * PI);
            }

            cr.stroke().expect("Couldn't stroke on Cairo Context");
            cr.restore().unwrap();

            /*
                Draw the graph itself
            */
            cr.save().unwrap();

            GdkCairoContextExt::set_source_rgba(&cr, &graph_color);
            cr.move_to(
                f64::from(HALF_X_PADDING),
                f64::from(
                    inner.height - inner.points.get(0).unwrap().value * inner.scale_y
                        + HALF_Y_PADDING,
                ),
            );

            for (i, point) in inner.points.iter().enumerate() {
                let next_value = if (i + 1) >= inner.points.len() {
                    break;
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

            cr.line_to(
                f64::from(inner.width + HALF_X_PADDING),
                f64::from(
                    inner.height - inner.points.last().unwrap().value * inner.scale_y
                        + HALF_Y_PADDING,
                ),
            );
            cr.stroke_preserve()
                .expect("Couldn't stroke on Cairo Context");

            cr.set_line_width(0.0);
            cr.line_to(
                f64::from(inner.width + HALF_X_PADDING),
                f64::from(inner.height + HALF_Y_PADDING),
            );
            cr.line_to(
                f64::from(HALF_X_PADDING),
                f64::from(inner.height + HALF_Y_PADDING),
            );
            cr.close_path();

            cr.set_source_rgba(
                f64::from(graph_color.red()),
                f64::from(graph_color.green()),
                f64::from(graph_color.blue()),
                0.65,
            );
            cr.stroke_preserve()
                .expect("Couldn't stroke on Cairo Context");
            cr.fill().expect("Couldn't fill Cairo Context");
            cr.restore().unwrap();

            if let Some(hover_func) = &inner.hover_func {
                if let Some(hover_point) = &inner.hover_point {
                    let layout = widget.create_pango_layout(Some(&hover_func(&hover_point.point)));
                    let (_, extents) = layout.extents();
                    let radius = pango::units_to_double(extents.height()) / 5.0;
                    let degrees = PI / 180.0;
                    let padding = 12.0;

                    // If the tooltip doesn't fit to the right side of the point, draw it on the left side of the point
                    let x_delta = if (hover_point.x
                        + pango::units_to_double(extents.width()) as f32
                        + padding * 2.0)
                        > inner.width
                    {
                        (pango::units_to_double(extents.width()) as f32 + padding * 3.0) * -1.0
                    } else {
                        0.0
                    };

                    cr.new_sub_path();
                    cr.arc(
                        f64::from(hover_point.x + padding * 2.0 + x_delta)
                            + pango::units_to_double(extents.width())
                            - radius,
                        f64::from(hover_point.y - padding / 2.0)
                            - pango::units_to_double(extents.height()) / 2.0
                            + radius,
                        radius,
                        -90.0 * degrees,
                        0.0,
                    );
                    cr.arc(
                        f64::from(hover_point.x + padding * 2.0 + x_delta)
                            + pango::units_to_double(extents.width())
                            - radius,
                        f64::from(hover_point.y + padding / 2.0)
                            + pango::units_to_double(extents.height()) / 2.0
                            - radius,
                        radius,
                        0.0,
                        90.0 * degrees,
                    );
                    cr.arc(
                        f64::from(hover_point.x + padding + x_delta) + radius,
                        f64::from(hover_point.y + padding / 2.0)
                            + pango::units_to_double(extents.height()) / 2.0
                            - radius,
                        radius,
                        90.0 * degrees,
                        180.0 * degrees,
                    );
                    cr.arc(
                        f64::from(hover_point.x + padding + x_delta) + radius,
                        f64::from(hover_point.y - padding / 2.0)
                            - pango::units_to_double(extents.height()) / 2.0
                            + radius,
                        radius,
                        180.0 * degrees,
                        270.0 * degrees,
                    );
                    cr.close_path();
                    cr.set_source_rgba(0.0, 0.0, 0.0, 0.65);
                    cr.fill_preserve().expect("Couldn't fill Cairo Context");

                    cr.move_to(
                        f64::from(hover_point.x + padding * 1.5 + x_delta),
                        f64::from(hover_point.y) - pango::units_to_double(extents.height()) / 2.0,
                    );
                    cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
                    pangocairo::show_layout(&cr, &layout);
                    cr.stroke().expect("Couldn't stroke on Cairo Context");
                }
            }
        }
    }

    impl ObjectImpl for GraphView {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.set_hexpand(true);
            obj.set_vexpand(true);
            let gesture_controller = gtk::GestureClick::new();
            gesture_controller.set_touch_only(true);
            gesture_controller.connect_pressed(
                clone!(@weak obj => move |c, _, x, y| obj.on_motion_event(x, y, true, c)),
            );
            obj.add_controller(&gesture_controller);

            let motion_controller = gtk::EventControllerMotion::new();
            motion_controller.connect_enter(
                clone!(@weak obj => move|c, x, y| obj.on_motion_event(x, y, false, c)),
            );
            motion_controller.connect_motion(
                clone!(@weak obj => move|c, x, y| obj.on_motion_event(x, y, false, c)),
            );
            obj.add_controller(&motion_controller);

            let mut inner = self.inner.borrow_mut();
            inner.hover_max_pointer_deviation = (8 * obj.scale_factor()).try_into().unwrap();
        }
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecBoxed::builder::<FnBoxedPoint>("hover-func")
                        .write_only()
                        .build(),
                    glib::ParamSpecFloat::builder("limit")
                        .minimum(-1.0)
                        .default_value(-1.0)
                        .build(),
                    glib::ParamSpecString::builder("limit-label").build(),
                    glib::ParamSpecFloat::builder("x-lines-interval")
                        .minimum(0.0)
                        .build(),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let obj = self.obj();
            match pspec.name() {
                "hover-func" => {
                    self.inner.borrow_mut().hover_func =
                        value.get::<FnBoxedPoint>().unwrap().0.borrow_mut().take()
                }
                "limit" => {
                    let mut inner = self.inner.borrow_mut();
                    let val = value.get::<f32>().unwrap();
                    let limit = if val < 0.0 { None } else { Some(val) };

                    inner.limit = limit;

                    if inner.biggest_value < inner.limit.unwrap_or(0.0) {
                        inner.biggest_value = inner.limit.unwrap();
                    }

                    obj.queue_draw();
                }

                "limit-label" => {
                    self.inner.borrow_mut().limit_label = value.get().unwrap();
                    obj.queue_draw();
                }
                "x-lines-interval" => {
                    self.inner.borrow_mut().x_lines_interval = value.get().unwrap();
                    obj.queue_draw();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "limit" => self.inner.borrow().limit.unwrap_or(-1.0).to_value(),
                "limit-label" => self.inner.borrow().limit_label.to_value(),
                "x-lines-interval" => self.inner.borrow().x_lines_interval.to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    /// A View for visualizing the development of data over time.
    pub struct GraphView(ObjectSubclass<imp::GraphView>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl GraphView {
    pub fn new() -> Self {
        glib::Object::new(&[])
    }

    pub fn limit(&self) -> Option<f32> {
        let val = self.property::<f32>("limit");
        if val < 0.0 {
            None
        } else {
            Some(val)
        }
    }

    pub fn limit_label(&self) -> Option<String> {
        self.property("limit-label")
    }

    /// Set the function that should be called when the user hovers over a point.
    ///
    /// # Arguments
    /// * `hover_func` - A function that takes a `Point` and renders it to a string that is displayed as tooltip on the graph.
    pub fn set_hover_func(&self, hover_func: Option<Box<dyn Fn(&Point) -> String>>) {
        self.set_property("hover-func", FnBoxedPoint::new(hover_func))
    }

    /// Set the limit (e.g. step goal) that is marked in the graph.
    pub fn set_limit(&self, limit: Option<f32>) {
        self.set_property("limit", limit.unwrap_or(-1.0))
    }

    /// Set the label that should be displayed on the limit label.
    pub fn set_limit_label(&self, limit_label: Option<String>) {
        self.set_property("limit-label", limit_label)
    }

    /// Sets the points that should be rendered in the graph view.
    pub fn set_points(&self, points: Vec<Point>) {
        let layout = self.create_pango_layout(Some(&glib::DateTime::local().format_local()));
        let (_, extents) = layout.extents();
        let datapoint_width = pango::units_to_double(extents.width()) + f64::from(HALF_X_PADDING);

        self.set_size_request(
            (datapoint_width as usize * points.len())
                .try_into()
                .unwrap(),
            -1,
        );

        let mut inner = self.imp().inner.borrow_mut();
        inner.biggest_value = points
            .iter()
            .max_by(|x, y| (x.value as u32).cmp(&(y.value as u32)))
            .map(|b| b.value)
            .unwrap();

        if inner.biggest_value < inner.limit.unwrap_or(0.0) {
            inner.biggest_value = inner.limit.unwrap();
        }

        inner.points = points;
        self.queue_draw();
    }

    /// Set the interval factor in which the background lines are drawn in the graph. E.g. if you set this to `10`,
    /// lines will be drawn in `biggest_value` / 4 rounded to the next 10 multiple.
    pub fn set_x_lines_interval(&self, x_lines_interval: f32) {
        self.set_property("x-lines-interval", x_lines_interval)
    }

    pub fn x_lines_interval(&self) -> f32 {
        self.property("x-lines-interval")
    }

    fn on_motion_event(
        &self,
        x: f64,
        y: f64,
        allow_touch: bool,
        controller: &impl IsA<gtk::EventController>,
    ) {
        let mut inner = self.imp().inner.borrow_mut();
        let hover_max_pointer_deviation = inner.hover_max_pointer_deviation;

        let approx_matches = |num: f64, approx_range: f32| {
            num > (approx_range - hover_max_pointer_deviation as f32).into()
                && num < (approx_range + hover_max_pointer_deviation as f32).into()
        };

        // Don't handle touch events, we do that via Gtk.GestureClick.
        if !allow_touch {
            if let Some(device) = controller.current_event_device() {
                if device.source() == gdk::InputSource::Touchscreen {
                    return;
                }
            }
        }

        let mut point_res = None;
        for (i, point) in inner.points.iter().enumerate() {
            let point_x = i as f32 * inner.scale_x + HALF_X_PADDING;
            let point_y = inner.height - point.value * inner.scale_y + HALF_Y_PADDING;

            if approx_matches(x, point_x) && approx_matches(y, point_y) {
                point_res = Some(imp::HoverPoint {
                    point: point.clone(),
                    x: point_x,
                    y: point_y,
                });
            }
        }

        if let Some(point) = point_res {
            inner.hover_point = Some(point);
            self.queue_draw();
        } else if inner.hover_point.is_some() {
            inner.hover_point = None;
            self.queue_draw();
        }
    }
}

#[cfg(test)]
mod test {
    use super::GraphView;
    use crate::utils::init_gtk;

    #[test]
    fn new() {
        init_gtk();
        GraphView::new();
    }

    #[test]
    fn properties() {
        init_gtk();
        let g = GraphView::new();
        g.set_limit(g.limit());
        g.set_limit_label(g.limit_label());
        g.set_x_lines_interval(g.x_lines_interval());
        g.set_hover_func(None);
    }
}
