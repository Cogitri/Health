/* bar_graph_view.rs
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

use crate::{
    core::{date::prelude::*, i18n},
    model::{ActivityInfo, ActivityType},
};
use chrono::{Date, FixedOffset, Local};
use gtk::{gdk, gio::subclass::prelude::*, glib, pango, prelude::*};
use std::{collections::HashMap, convert::TryInto};

/// A [Tuple] describes a single segment in a [BarGraphView]
#[derive(Debug, Clone, PartialEq)]
pub struct Tuple {
    pub activity_name: String,
    pub calories: i64,
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct SplitBar {
    pub date: Date<FixedOffset>,
    pub calorie_split: HashMap<ActivityType, i64>,
}

static HALF_X_PADDING: f32 = 40.0;
static HALF_Y_PADDING: f32 = 30.0;

mod imp {
    use super::{Tuple, HALF_X_PADDING, HALF_Y_PADDING};
    use crate::{
        core::date::prelude::*,
        model::{ActivityInfo, ActivityType},
        views::SplitBar,
    };
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
        pub data: Tuple,
        pub x: f32,
        pub y: f32,
    }

    pub struct BarGraphViewMut {
        pub biggest_value: f32,
        pub height: f32,
        pub hover_func: Option<Box<dyn Fn(&Tuple) -> String>>,
        pub hover_max_pointer_deviation: u32,
        pub hover_point: Option<HoverPoint>,
        pub limit: Option<f32>,
        pub limit_label: Option<String>,
        pub scale_x: f32,
        pub scale_y: f32,
        pub width: f32,
        pub x_lines_interval: f32,
        pub split_bars: Vec<SplitBar>,
        pub rmr: f32,
    }

    pub struct BarGraphView {
        pub inner: RefCell<BarGraphViewMut>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BarGraphView {
        const NAME: &'static str = "HealthBarGraphView";
        type ParentType = gtk::Widget;
        type Type = super::BarGraphView;

        fn new() -> Self {
            Self {
                inner: RefCell::new(BarGraphViewMut {
                    biggest_value: 0.1,
                    height: 0.0,
                    hover_func: None,
                    hover_max_pointer_deviation: 8,
                    hover_point: None,
                    limit: None,
                    limit_label: None,
                    scale_x: 0.0,
                    scale_y: 0.0,
                    width: 0.0,
                    x_lines_interval: 100.0,
                    split_bars: Vec::new(),
                    rmr: 0.0,
                }),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }
    }

    impl WidgetImpl for BarGraphView {
        fn snapshot(&self, widget: &Self::Type, snapshot: &gtk::Snapshot) {
            let mut inner = self.inner.borrow_mut();

            inner.height = widget.height() as f32 - HALF_Y_PADDING * 2.0;
            inner.width = widget.width() as f32
                - HALF_X_PADDING * if inner.split_bars.len() > 1 { 5.0 } else { 2.0 };
            let biggest_value = if inner.split_bars.is_empty() {
                inner.scale_x = inner.width;
                inner.scale_y = inner.height / 10000.0;
                0.1
            } else {
                // Round up to 100, the graph looks a bit odd if we draw lines at biggest_value / 4 instead of
                // using even numbers
                let biggest_value = inner.biggest_value + inner.x_lines_interval
                    - inner.biggest_value % inner.x_lines_interval;

                // If we have more than one split_bar, we don't want an empty bar at the end of the graph
                inner.scale_x = if inner.split_bars.len() > 1 {
                    inner.width / (inner.split_bars.len() - 1) as f32
                } else {
                    inner.width as f32
                };

                inner.scale_y = inner.height / biggest_value;

                biggest_value
            };

            let cr = snapshot
                .append_cairo(&gtk::graphene::Rect::new(
                    0.0,
                    0.0,
                    widget.width() as f32,
                    widget.height() as f32,
                ))
                .unwrap();
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
                    f64::from(inner.width + 4.0 * HALF_X_PADDING * 2.0),
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

            for (i, bar) in inner.split_bars.iter().enumerate() {
                let layout = widget.create_pango_layout(Some(&bar.date.format_local()));
                let (_, extents) = layout.extents();

                cr.move_to(
                    f64::from(i as f32 * inner.scale_x + HALF_X_PADDING * 2.0)
                        - pango::units_to_double(extents.width()) / 2.0,
                    f64::from(inner.height + HALF_Y_PADDING * 1.5)
                        - pango::units_to_double(extents.height()) / 2.0,
                );
                pangocairo::show_layout(&cr, &layout);
            }

            cr.stroke().expect("Couldn't stroke on Cairo Context");
            cr.restore().unwrap();

            /*
                Draw a split bar for each datapoint
            */
            cr.save().unwrap();
            cr.set_line_width(0.1);
            for (i, split_bar) in inner.split_bars.iter().enumerate() {
                let mut sorted_by_calories = split_bar
                    .calorie_split
                    .clone()
                    .into_iter()
                    .map(|(id, calorie)| (id, calorie))
                    .collect::<Vec<(ActivityType, i64)>>();

                sorted_by_calories.sort_by(|a, b| b.1.cmp(&a.1));

                let scroll_thickness = 2.0;
                let x = f64::from(i as f32 * inner.scale_x + HALF_X_PADDING);
                let height = if inner.rmr != 0.0 {
                    f64::from(inner.rmr * inner.scale_y)
                } else {
                    20.0
                };
                let mut bar_top =
                    f64::from(inner.height + HALF_Y_PADDING) - height - scroll_thickness;
                GdkCairoContextExt::set_source_rgba(
                    &cr,
                    &gtk::gdk::RGBA::builder()
                        .red(0.0)
                        .blue(0.0)
                        .green(0.0)
                        .alpha(1.0)
                        .build(),
                );
                cr.move_to(x + f64::from(HALF_X_PADDING), bar_top);
                cr.rectangle(f64::from(HALF_X_PADDING) + x - 10.0, bar_top, 20.0, height);
                cr.stroke_preserve()
                    .expect("Couldn't stroke on Cairo Context");
                cr.fill().expect("Couldn't fill on Cairo Context");

                for (activity_id, calories) in sorted_by_calories {
                    GdkCairoContextExt::set_source_rgba(
                        &cr,
                        &ActivityInfo::from(activity_id).color,
                    );
                    let calories = calories as f32;
                    bar_top -= f64::from(calories * inner.scale_y);
                    cr.move_to(x + f64::from(HALF_X_PADDING), bar_top);
                    cr.rectangle(
                        f64::from(HALF_X_PADDING) + x - 10.0,
                        bar_top,
                        20.0,
                        f64::from(calories * inner.scale_y),
                    );
                    cr.stroke_preserve()
                        .expect("Couldn't stroke on Cairo Context");
                    cr.fill().expect("Couldn't fill on Cairo Context");
                }
            }

            cr.stroke().expect("Couldn't stroke on Cairo Context");
            cr.restore().unwrap();

            if let Some(hover_func) = &inner.hover_func {
                if let Some(hover_point) = &inner.hover_point {
                    let layout = widget.create_pango_layout(Some(&hover_func(&Tuple {
                        activity_name: hover_point.data.activity_name.to_string(),
                        calories: hover_point.data.calories,
                        message: hover_point.data.message.to_string(),
                    })));
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

    impl ObjectImpl for BarGraphView {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

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
    }
}

glib::wrapper! {
    /// A View for visualizing the development of data over time.
    pub struct BarGraphView(ObjectSubclass<imp::BarGraphView>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl BarGraphView {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create BarGraphView")
    }

    /// Set the function that should be called when the user hovers over a point.
    ///
    /// # Arguments
    /// * `hover_func` - A function that takes a `Tuple` and renders it to a string that is displayed as tooltip on the graph.
    pub fn set_hover_func(&self, hover_func: Option<Box<dyn Fn(&Tuple) -> String>>) {
        self.imp().inner.borrow_mut().hover_func = hover_func;
        self.queue_draw();
    }

    pub fn set_rmr(&self, rmr: f32) {
        self.imp().inner.borrow_mut().rmr = rmr;
    }

    pub fn set_split_bars(&self, split_bars: Vec<SplitBar>) {
        let layout = self.create_pango_layout(Some(&Local::now().format_local()));
        let (_, extents) = layout.extents();
        let datapoint_width = pango::units_to_double(extents.width()) + f64::from(HALF_X_PADDING);

        self.set_size_request(
            (datapoint_width as usize * split_bars.len())
                .try_into()
                .unwrap(),
            -1,
        );
        let mut inner = self.imp().inner.borrow_mut();
        inner.split_bars = split_bars.clone();
        inner.split_bars.sort_by(|a, b| a.date.cmp(&b.date));

        let total_calories = |calorie_split: &HashMap<ActivityType, i64>| -> i64 {
            calorie_split.iter().map(|b| b.1).sum()
        };

        inner.biggest_value = split_bars
            .iter()
            .max_by(|x, y| {
                (total_calories(&x.calorie_split)).cmp(&(total_calories(&y.calorie_split)))
            })
            .map(|b| total_calories(&b.calorie_split))
            .unwrap() as f32
            + inner.rmr;

        if inner.biggest_value < inner.limit.unwrap_or(0.0) {
            inner.biggest_value = inner.limit.unwrap();
        }

        self.queue_draw();
    }

    /// Set the interval factor in which the background lines are drawn in the graph. E.g. if you set this to `10`,
    /// lines will be drawn in `biggest_value` / 4 rounded to the next 10 multiple.
    pub fn set_x_lines_interval(&self, x_lines_interval: f32) {
        self.imp().inner.borrow_mut().x_lines_interval = x_lines_interval;
        self.queue_draw();
    }

    fn imp(&self) -> &imp::BarGraphView {
        imp::BarGraphView::from_instance(self)
    }

    fn on_motion_event(
        &self,
        x: f64,
        y: f64,
        allow_touch: bool,
        controller: &impl IsA<gtk::EventController>,
    ) {
        let mut inner = self.imp().inner.borrow_mut();
        // Don't handle touch events, we do that via Gtk.GestureClick.
        if !allow_touch {
            if let Some(device) = controller.current_event_device() {
                if device.source() == gdk::InputSource::Touchscreen {
                    return;
                }
            }
        }

        let mut segment = None;
        let mut bar_index = None;

        // find which bar we are touching, if any
        for i in 0..inner.split_bars.len() {
            let point_x = i as f32 * inner.scale_x + 2.0 * HALF_X_PADDING;
            if (point_x - x as f32 as f32).abs() <= 10.0 {
                bar_index = Some(i);
                break;
            }
        }

        // If we are touching any bar, which segment, if any (we could be above the bar)
        if let Some(index) = bar_index {
            let touched_bar = inner.split_bars[index].clone();
            let mut sorted_by_calories = touched_bar
                .calorie_split
                .into_iter()
                .map(|(id, calorie)| (id, calorie))
                .collect::<Vec<(ActivityType, i64)>>();
            sorted_by_calories.sort_by(|a, b| b.1.cmp(&a.1));

            let height = if inner.rmr != 0.0 {
                f64::from(inner.rmr * inner.scale_y)
            } else {
                20.0
            };
            let mut cursor_height = f64::from(inner.height + HALF_Y_PADDING) - y - height;
            if cursor_height >= 0.0 {
                for (id, calories) in sorted_by_calories {
                    cursor_height -= f64::from(calories as f32 * inner.scale_y);
                    if cursor_height < 0.0 {
                        segment = Some(imp::HoverPoint {
                            data: Tuple {
                                activity_name: ActivityInfo::from(id).name,
                                calories,
                                message: "".to_string(),
                            },
                            x: x as f32,
                            y: y as f32,
                        });
                        break;
                    }
                }
            } else if cursor_height >= -height {
                let message = if inner.rmr != 0.0 {
                    String::new()
                } else {
                    i18n("Enter weight record to calculate idle calories")
                };
                segment = Some(imp::HoverPoint {
                    data: Tuple {
                        activity_name: i18n("Idle calories"),
                        calories: inner.rmr as i64,
                        message,
                    },
                    x: x as f32,
                    y: y as f32,
                });
            }
        }

        inner.hover_point = segment;
        self.queue_draw();
    }
}

#[cfg(test)]
mod test {
    use super::BarGraphView;
    use crate::utils::init_gtk;

    #[test]
    fn new() {
        init_gtk();
        BarGraphView::new();
    }
}
