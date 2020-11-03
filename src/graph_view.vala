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
    public class GraphView : Gtk.Widget {
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
        private uint _x_lines_interval = 500;
        public uint x_lines_interval {
            get {
                return _x_lines_interval;
            }
            set {
                this._x_lines_interval = value;
                this.queue_draw ();
            }
        }

        public GraphView (Gee.ArrayList<Point> points, string limitlabel = "", double limit = -1) {
            this.points = points;
            this.limit = limit;
            this.limitlabel = limitlabel;
            this.x_padding = 60;
            this.y_padding = 60;
            this.hexpand = this.vexpand = true;
        }

        protected override void snapshot (Gtk.Snapshot snapshot) {
            double biggest_value = 0.000001;
            foreach (var point in this.points) {
                if (point.value > biggest_value) {
                    biggest_value = point.value;
                }
            }
            // Round up to 500, the graph looks a bit odd if we draw lines at biggest_value / 4 instead of
            // using even numbers
            biggest_value = biggest_value + this.x_lines_interval - biggest_value % this.x_lines_interval;
            var height = this.get_height () - this.y_padding;
            var width = this.get_width () - this.x_padding;
            var scale_x = width / (this.points.size > 1 ? (this.points.size - 1) : 1);
            var scale_y = height / biggest_value;
            var style_context = this.get_style_context ();

            /*
                Get cairo context
             */
            var cr = snapshot.append_cairo ({{0, 0}, {(float) this.get_width (), (float) this.get_height ()}});

            /*
                Draw outlines
             */
            cr.save ();

            var outline_color = style_context.get_color ();
            cr.set_source_rgba (outline_color.red, outline_color.green, outline_color.blue, 0.5);

            for (int i = 0; i < 5; i++) {
                var mul = (height) / 4.0;
                cr.move_to (width + this.x_padding / 2, mul * i + this.y_padding / 2);
                cr.line_to (this.x_padding / 2, mul * i + this.y_padding / 2);
                if (!this.points.is_empty) {
                    Pango.Rectangle extents;
                    var layout = this.create_pango_layout ("%u".printf ((uint) (biggest_value / 4.0) * (4 - i)));
                    layout.get_extents (null, out extents);

                    cr.rel_move_to (0, Pango.units_to_double (extents.height) * -1);
                    Pango.cairo_show_layout (cr, layout);
                }
            }

            cr.stroke ();
            cr.restore ();

            /*
                Draw X ticks (dates)
             */
            cr.save ();
            cr.set_source_rgba (outline_color.red, outline_color.green, outline_color.blue, 0.5);

            for (int i = 0; i < this.points.size; i++) {
                var point = this.points.get (i);
                Pango.Rectangle extents;
                /* TRANSLATORS: this is the date as displayed in the graph, e.g. 30/9 for September 30 */
                var layout = this.create_pango_layout (_ ("%d/%d").printf (point.date.get_day (), point.date.get_month ()));
                layout.get_extents (null, out extents);

                cr.move_to (i * scale_x + (this.x_padding - Pango.units_to_double (extents.width)) / 2, height + this.y_padding / 1.5 - Pango.units_to_double (extents.height) / 2);
                Pango.cairo_show_layout (cr, layout);
            }

            cr.stroke ();
            cr.restore ();

            /*
                Draw limit/goal (if any)
             */
            if (this.limit > 0) {
                cr.save ();

                cr.set_source_rgba (outline_color.red, outline_color.green, outline_color.blue, 0.5);
                cr.set_dash ({ 10, 5 }, 0);
                cr.move_to (this.x_padding / 2, height - limit * scale_y + this.y_padding / 2);
                cr.show_text (this.limitlabel);
                cr.line_to (width + this.x_padding / 2, height - limit * scale_y + this.y_padding / 2);

                cr.stroke ();
                cr.restore ();
            }

            if (this.points.is_empty) {
                return;
            }

            /*
                Draw a point for each datapoint
             */
            cr.save ();

            cr.set_source_rgba (0, 174, 174, 1.0);
            cr.set_line_width (4);
            for (int i = 0; i < this.points.size; i++) {
                var value = this.points[i].value;
                var x = i * scale_x + this.x_padding / 2;
                var y = height - value * scale_y + this.y_padding / 2;
                cr.move_to (x, y);
                cr.arc (x, y, 2, 0, 2 * GLib.Math.PI);
            }

            cr.stroke ();
            cr.restore ();

            /*
                Draw the graph itself
             */
            cr.set_source_rgba (0, 174, 174, 0.8);
            cr.move_to (this.x_padding / 2, height - points.get (0).value * scale_y + this.y_padding / 2);

            for (int i = 0; i < this.points.size - 1; i++) {
                var previous_value = this.points[i].value;
                var current_value = ((i + 1) >= this.points.size) ? this.points[i].value : this.points[i + 1].value;
                var smoothness_factor = 0.5;

                cr.curve_to (
                    (i + smoothness_factor) * scale_x + this.x_padding / 2,
                    height - previous_value * scale_y + this.y_padding / 2,
                    (i + 1 - smoothness_factor) * scale_x + this.x_padding / 2,
                    height - current_value * scale_y + this.y_padding / 2,
                    (i + 1) * scale_x + this.x_padding / 2,
                    height - current_value * scale_y + this.y_padding / 2
                );
            }

            cr.stroke ();
        }
    }
}
