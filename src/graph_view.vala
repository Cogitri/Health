/* graph_view.vala
 *
 * Copyright 2020 Rasmus Thomsen <oss@cogitri.dev>
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
namespace Health {
    /**
     * A Point describes a single datapoint in a {@link GraphView}.
     */
    public class Point {
        public Date date;
        public double value;

        public Point (Date date, double value) {
            this.date = date;
            this.value = value;
        }
    }

    /**
     * A View for visualizing the development of data over time.
     */
    public class GraphView : Gtk.DrawingArea {
        private double x_padding;
        private double y_padding;
        private double _limit;
        public double limit {
            get {
                return _limit;
            }
            set {
                this._limit = value;
                this.queue_draw ();
            }
        }
        private Gee.ArrayList<Point> _points;
        public Gee.ArrayList<Point> points {
            get {
                return _points;
            }
            set {
                this._points = value;
                this.queue_draw ();
            }
        }
        private string _limitlabel;
        public string limitlabel {
            get {
                return _limitlabel;
            }
            set {
                this._limitlabel = value;
                this.queue_draw ();
            }
        }

        public GraphView (Gee.ArrayList<Point> points, string limitlabel = "", double limit = -1) {
            this.points = points;
            this.limit = limit;
            this.limitlabel = limitlabel;
            this.x_padding = 60;
            this.y_padding = 60;
            this.set_size_request (400, 400);
            this.set_draw_func (this.draw);
        }

        private void draw (Gtk.DrawingArea da, Cairo.Context cr, int alloc_width, int alloc_height) {
            var self = (GraphView) da;
            double biggest_value = 0.000001;
            foreach (var point in self.points) {
                if (point.value > biggest_value) {
                    biggest_value = point.value;
                }
            }
            var height = alloc_height - self.y_padding;
            var width = alloc_width - self.x_padding;
            var scale_x = width / (self.points.size > 1 ? (self.points.size - 1) : 1);
            var scale_y = height / biggest_value;
            var style_context = self.get_style_context ();

            /*
                Draw outlines
            */
            cr.save ();

            var outline_color = style_context.get_color ();
            cr.set_source_rgba (outline_color.red, outline_color.green, outline_color.blue, 0.5);

            for (int i = 0; i < 5; i++) {
                var mul = (height) / 4.0;
                cr.move_to (width + self.x_padding / 2, mul * i + self.y_padding / 2);
                cr.line_to (self.x_padding / 2, mul * i + self.y_padding / 2);
                if (!self.points.is_empty) {
                    cr.show_text ("%u".printf ((uint) (biggest_value / 4.0) * (4 - i)));
                }
            }

            cr.stroke ();
            cr.restore ();

            /*
                Draw X ticks (dates)
            */
            cr.save ();
            var text_color = style_context.get_color ();
            cr.set_source_rgba (text_color.red, text_color.green, text_color.blue, text_color.alpha);

            for (int i = 0; i < self.points.size; i++) {
                var point = self.points.get (i);
                var font_size = 10;

                cr.set_font_size (font_size);

                cr.move_to (i * scale_x + self.x_padding / 2 - font_size, height + self.y_padding / 1.25);
                /* TRANSLATORS: self is the date as displayed in the graph, e.g. 30/9 for September 30 */
                cr.show_text (_ ("%d/%d").printf (point.date.get_day (), point.date.get_month ()));
            }

            cr.stroke ();
            cr.restore ();

            /*
                Draw limit/goal (if any)
            */
            if (self.limit > 0) {
                cr.save ();

                cr.set_source_rgba (outline_color.red, outline_color.green, outline_color.blue, 0.5);
                cr.set_dash ({10, 5}, 0);
                cr.move_to (self.x_padding / 2, height - limit * scale_y + self.y_padding / 2);
                cr.show_text (self.limitlabel);
                cr.line_to (width + self.x_padding / 2, height - limit * scale_y + self.y_padding / 2);

                cr.stroke ();
                cr.restore ();
            }

            if (self.points.is_empty) {
                return;
            }

            /*
                Draw a point for each datapoint
            */
            cr.save ();

            cr.set_source_rgba (0, 174, 174, 1.0);
            cr.set_line_width (4);
            for (int i = 0; i < self.points.size; i++) {
                var value = self.points[i].value;
                var x = i * scale_x + self.x_padding / 2;
                var y = height - value * scale_y + self.y_padding / 2;
                cr.move_to (x, y);
                cr.arc (x, y, 2, 0, 2 * GLib.Math.PI);
            }

            cr.stroke ();
            cr.restore ();

            /*
                Draw the graph itself
            */
            cr.set_source_rgba (0, 174, 174, 0.8);
            cr.move_to (self.x_padding / 2, height - points.get (0).value * scale_y + self.y_padding / 2);

            for (int i = 0; i < self.points.size - 1; i++) {
                var previous_value = self.points[i].value;
                var current_value = ((i + 1) >= self.points.size) ? self.points[i].value : self.points[i + 1].value;
                var smoothness_factor = 0.5;

                cr.curve_to (
                  (i + smoothness_factor) * scale_x + self.x_padding / 2,
                  height - previous_value * scale_y + self.y_padding / 2,
                  (i + 1 - smoothness_factor) * scale_x + self.x_padding / 2,
                  height - current_value * scale_y + self.y_padding / 2,
                  (i + 1) * scale_x + self.x_padding / 2,
                  height - current_value * scale_y + self.y_padding / 2
                );
            }

            cr.stroke ();
        }

    }
}
