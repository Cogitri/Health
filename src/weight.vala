/* weight.vala
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
    public class Weight : GLib.Object {
        public GLib.Date date { get; private set; }
        public double weight { get; private set; }

        public Weight (GLib.Date date, double weight) {
            this.date = date;
            this.weight = weight;
        }

    }

    public class WeightGraphModel : GraphModel<Weight> {

        public override bool reload () {
            var db = new SqliteDatabase ();
            try {
                db.open ();
                this.arr = db.get_weights_after (get_date_in_n_days (-30));
                return true;
            } catch (DatabaseError e) {
                warning (_ ("Failed to load weights from database due to error %s"), e.message);
                return false;
            }
        }

        public override void to_arrays (out double[] days, out double[] values) {
            base.to_arrays (out days, out values);

            if (this.arr.is_empty) {
                return;
            }

            this.arr.sort ((a, b) => { return a.date.compare (b.date); });

            var first_date = this.arr.get (0).date;
            int i = 0;
            foreach (var weight in this.arr) {
                values[i] = weight.weight;
                days[i] = first_date.days_between (weight.date);
                i++;
            }
        }

        public double get_last_weight () {
            this.arr.sort ((a, b) => { return a.date.compare (b.date); });
            if (this.arr.is_empty) {
                return 0.0;
            }
            var last_weight = this.arr.get (this.arr.size - 1);
            return last_weight.weight;
        }

    }

    public class WeightGraphView : Caroline {
        public WeightGraphView (WeightGraphModel model) {
            double[] days;
            double[] weights;
            model.to_arrays (out days, out weights);
            base (days, weights, "smooth-line", true, true);
        }

    }

    [GtkTemplate (ui = "/org/gnome/Health/weight_view.ui")]
    public class WeightView : View {
        [GtkChild]
        private Gtk.Label title_label;
        [GtkChild]
        private Gtk.Box main_box;
        private WeightGraphView weight_graph_view;
        private WeightGraphModel weight_graph_model;

        public WeightView (WeightGraphModel model) {
            this.name = "Weight";
            this.title = _ ("Weight");
            this.title_label.set_text (_ ("Current weight: %4.lf KG").printf (model.get_last_weight ()));
            this.weight_graph_view = new WeightGraphView (model);
            this.weight_graph_model = model;
            this.main_box.pack_start (this.weight_graph_view, true, true, 0);
            this.main_box.show_all ();
        }

        public override void update () {
            this.weight_graph_model.reload ();
            this.title_label.set_text (_ ("Current weight: %4.lf KG").printf (this.weight_graph_model.get_last_weight ()));
            this.main_box.remove (this.weight_graph_view);
            this.weight_graph_view = new WeightGraphView (this.weight_graph_model);
            this.main_box.pack_start (this.weight_graph_view, true, true, 0);
            this.main_box.show_all ();
        }

    }
}
