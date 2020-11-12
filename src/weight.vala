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
    /**
     * A weight measurement for a single day.
     */
    public class Weight : GLib.Object {
        public GLib.Date date { get; private set; }
        public WeightUnitContainer weight { get; set; }

        public Weight (GLib.Date date, WeightUnitContainer weight) {
            this.date = date;
            this.weight = weight;
        }

    }

    /**
     * An implementation of {@link GraphModel} that interacts with the user's weight measurements.
     */
    public class WeightGraphModel : GraphModel<Weight> {
        private Settings settings;
        private TrackerDatabase db;

        public WeightGraphModel (Settings settings, TrackerDatabase db) {
            this.settings = settings;
            this.db = db;

            this.init ();
        }

        /**
         * Reload the data from the DB
         *
         * This can be used e.g. after the user added a new weight measurement.
         * @return true if reloading suceeded.
         */
        public async override bool reload () {
            try {
                this.arr = yield db.get_weights_after (get_date_in_n_days (-30), this.settings, null);
                return true;
            } catch (GLib.Error e) {
                warning (_ ("Failed to load weights from database due to error %s"), e.message);
                return false;
            }
        }

        /**
         * {@inheritDoc}
         */
        public override Gee.ArrayList<Point> to_points () {
            var ret = new Gee.ArrayList<Point> ();

            this.arr.sort ((a, b) => { return a.date.compare (b.date); });

            foreach (var weight in this.arr) {
                ret.add (new Point (weight.date, weight.weight.value));
            }

            return ret;
        }

        public WeightUnitContainer? get_last_weight () {
            this.arr.sort ((a, b) => { return a.date.compare (b.date); });
            if (this.arr.is_empty) {
                return null;
            }
            var last_weight = this.arr.get (this.arr.size - 1);
            return last_weight.weight;
        }

    }

    /**
     * An implementation of {@link GraphView} that visualizes the user's weight measurements over time.
     */
    public class WeightGraphView : GraphView {
        public WeightGraphView (WeightGraphModel model, double weightgoal) {
            base (model.to_points (), _ ("Weightgoal"), weightgoal);
            this.x_lines_interval = 10;
        }

    }

    /**
     * An implementation of {@link View} visualizes BMI and weight development.
     */
    [GtkTemplate (ui = "/dev/Cogitri/Health/weight_view.ui")]
    public class WeightView : View {
        [GtkChild]
        private Gtk.Label title_label;
        [GtkChild]
        private Gtk.Box main_box;
        [GtkChild]
        private Gtk.Label weightgoal_label;
        private Gtk.Label no_data_label;
        private Settings settings;
        private WeightGraphView? weight_graph_view;
        private WeightGraphModel weight_graph_model;

        public WeightView (WeightGraphModel model, Settings settings, TrackerDatabase db) {
            this.name = "Weight";
            this.settings = settings;
            this.title = _ ("Weight");
            this.icon_name = "dev.Cogitri.Health-weight-scale-symbolic";
            this.weight_graph_model = model;

            this.update_weightgoal_label ();

            if (this.weight_graph_model.is_empty) {
                this.no_data_label = new Gtk.Label (_ ("No data has been added yet. Click + to add a new weight measurement."));
                this.main_box.append (this.no_data_label);
            } else {
                this.weight_graph_view = new WeightGraphView (model, this.settings.user_weightgoal.value);
                this.main_box.append ((!) this.weight_graph_view);
            }

            this.update ();
            this.settings.changed[Settings.USER_HEIGHT_KEY].connect (() => {
                this.update ();
            });
            this.settings.changed[Settings.USER_WEIGHTGOAL_KEY].connect (() => {
                this.update ();
            });
            this.settings.changed[Settings.UNITSYSTEM_KEY].connect (() => {
                this.update ();
            });
            db.weight_updated.connect (() => {
                this.update ();
            });
            this.destroy.connect (() => {
                this.main_box.unparent ();
            });
        }

        private double get_bmi () {
            var last_weight = this.weight_graph_model.get_last_weight ();
            if (last_weight == null) {
                return 0;
            }
            return ((!) last_weight).get_in_kg () / GLib.Math.pow (this.settings.user_height / 100.0, 2);
        }

        private void update_weightgoal_label () {
            var weight_goal = this.settings.user_weightgoal;
            if (weight_goal.get_in_kg () > 0.01 && this.weight_graph_model.is_empty) {
                string unitsystem;
                if (this.settings.unitsystem == Unitsystem.IMPERIAL) {
                    unitsystem = _ ("pounds");
                } else {
                    unitsystem = _ ("kilogram");
                }

                /* TRANSLATORS: the %s format strings are the weight unit, e.g. kilogram */
                this.weightgoal_label.set_text (_ ("Your weightgoal is %.2lf %s. Add a first weight measurement to see how close you are to reaching it."). printf (this.settings.user_weightgoal.value, unitsystem));
            } else if (weight_goal.get_in_kg () > 0.01 && !this.weight_graph_model.is_empty) {
                var goal_diff = ((!) ((!) this.weight_graph_model).get_last_weight ()).value - weight_goal.value;

                if (goal_diff < 0) {
                    goal_diff *= -1;
                }

                if (goal_diff == 0) {
                    this.weightgoal_label.set_text (_ ("You've reached your weightgoal. Great job!"));
                } else {
                    string unitsystem;
                    if (this.settings.unitsystem == Unitsystem.IMPERIAL) {
                        unitsystem = _ ("pounds");
                    } else {
                        unitsystem = _ ("kilogram");
                    }
                    /* TRANSLATORS: the two %s format strings are the weight unit, e.g. kilogram */
                    this.weightgoal_label.set_text (_ ("%.2lf %s left to reach your weightgoal of %.2lf %s").printf (goal_diff, unitsystem, weight_goal.value, unitsystem));
                }
            } else {
                this.weightgoal_label.set_text (_ ("No weightgoal set yet. You can set it in Health's preferences."));
            }
        }

        /**
         * Reload the {@link WeightGraphModel}'s data and refresh labels & the {@link WeightGraphView}.
         */
        public override void update () {
            this.weight_graph_model.reload.begin ((obj, res) => {
                if (this.weight_graph_model.reload.end (res)) {
                    this.title_label.set_text (_ ("Current BMI: %.2lf").printf (this.get_bmi ()));

                    this.update_weightgoal_label ();

                    if (this.weight_graph_view == null && !this.weight_graph_model.is_empty) {
                        this.main_box.remove (this.no_data_label);
                        this.weight_graph_view = new WeightGraphView (this.weight_graph_model, this.settings.user_weightgoal.value);
                        ((!) this.weight_graph_view).visible = true;
                        this.main_box.append ((!) this.weight_graph_view);
                    } else if (this.weight_graph_view != null) {
                        ((!) this.weight_graph_view).points = this.weight_graph_model.to_points ();
                        ((!) this.weight_graph_view).limit = this.settings.user_weightgoal.value;
                    }
                }
            });
        }

    }
}
