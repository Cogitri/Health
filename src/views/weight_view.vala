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
     * An implementation of {@link GraphView} that visualizes the user's weight measurements over time.
     */
    public class WeightGraphView : GraphView {
        public WeightGraphView (WeightGraphModel model, double weightgoal) {
            base (model.to_points (), _ ("Weightgoal"), weightgoal);
            this.x_lines_interval = 10;
            this.hover_func = (point) => {
                string unit = "KG";

                if (Settings.get_instance ().unitsystem == Unitsystem.IMPERIAL) {
                    unit = "PB";
                }

                /* TRANSLATORS: This is shown on-hover of points where %u is the weight, the first %s is the unit and the second %s is the already localised date (e.g. 2020-09-11) */
                return _ ("%u %s on %s").printf ((uint) point.value, unit, Util.datetime_from_date (point.date).format ("%x"));
            };
        }
    }

    /**
     * An implementation of {@link View} visualizes BMI and weight development.
     */
    [GtkTemplate (ui = "/dev/Cogitri/Health/ui/weight_view.ui")]
    public class WeightView : View {
        private Settings settings;
        private WeightGraphView? weight_graph_view;
        private WeightGraphModel weight_graph_model;

        public WeightView (WeightGraphModel model, TrackerDatabase db) {
            this.settings = Settings.get_instance ();
            this.weight_graph_model = model;

            this.update_weightgoal_label ();

            if (!this.weight_graph_model.is_empty) {
                this.weight_graph_view = new WeightGraphView (model, this.settings.user_weightgoal.value);
                this.scrolled_window.child = (!) this.weight_graph_view;
                this.stack.visible_child_name = "data_page";
            }

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
            this.update ();
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
                this.goal_label.set_text (_ ("Your weightgoal is %.2lf %s. Add a first weight measurement to see how close you are to reaching it."). printf (this.settings.user_weightgoal.value, unitsystem));
            } else if (weight_goal.get_in_kg () > 0.01 && !this.weight_graph_model.is_empty) {
                var goal_diff = ((!) ((!) this.weight_graph_model).get_last_weight ()).value - weight_goal.value;

                if (goal_diff < 0) {
                    goal_diff *= -1;
                }

                if (goal_diff == 0) {
                    this.goal_label.set_text (_ ("You've reached your weightgoal. Great job!"));
                } else {
                    string unitsystem;
                    if (this.settings.unitsystem == Unitsystem.IMPERIAL) {
                        unitsystem = _ ("pounds");
                    } else {
                        unitsystem = _ ("kilogram");
                    }
                    /* TRANSLATORS: the two %s format strings are the weight unit, e.g. kilogram */
                    this.goal_label.set_text (_ ("%.2lf %s left to reach your weightgoal of %.2lf %s").printf (goal_diff, unitsystem, weight_goal.value, unitsystem));
                }
            } else {
                this.goal_label.set_text (_ ("No weightgoal set yet. You can set it in Health's preferences."));
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
                        this.weight_graph_view = new WeightGraphView (this.weight_graph_model, this.settings.user_weightgoal.value);
                        this.scrolled_window.child = (!) this.weight_graph_view;
                        this.stack.visible_child_name = "data_page";
                    } else if (this.weight_graph_view != null) {
                        ((!) this.weight_graph_view).points = this.weight_graph_model.to_points ();
                        ((!) this.weight_graph_view).limit = this.settings.user_weightgoal.value;
                    }
                }
            });
        }

    }
}
