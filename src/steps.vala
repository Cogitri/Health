/* steps.vala
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
    public class Steps : GLib.Object {
        public GLib.Date date { get; private set; }
        public uint32 steps { get; private set; }

        public Steps (GLib.Date date, uint32 steps) {
            this.date = date;
            this.steps = steps;
        }

    }

    public class StepsGraphModel : GraphModel<Steps> {

        public override bool reload () {
            var db = new SqliteDatabase ();
            try {
                db.open ();
                this.arr = db.get_steps_after (get_date_in_n_days (-30));
                return true;
            } catch (DatabaseError e) {
                warning ("Failed to load steps from database due to error %s", e.message);
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
            var last_date = get_today_date ();
            var date_delta = first_date.days_between (last_date);
            values.resize (date_delta + 1);
            days.resize (date_delta + 1);
            int i = 0;
            foreach (var steps in this.arr) {
                var days_between = first_date.days_between (steps.date);

                int x;
                // fill in gaps where user didn't set a step count
                for (x = 0; x < days_between - i; x++) {
                    values[x + i] = 0;
                    days[x + i] = x + i;
                }
                i += x;

                values[i] = steps.steps;
                days[i] = days_between;
                i++;
            }
            var days_since_last_entry = this.arr.get (this.arr.size - 1).date.days_between (last_date);
            var last_entry_delta = first_date.days_between (this.arr.get (this.arr.size - 1).date);
            for (int j = 0; j < days_since_last_entry; j++) {
                values[j + i] = 0;
                days[j + i] = last_entry_delta + j + 1;
            }
        }

        public uint32 get_today_step_count () {
            var today = get_today_date ();
            var steps = this.arr.first_match ((s) => {
                return s.date.get_julian () == today.get_julian ();
            });
            return steps != null ? steps.steps : 0;
        }

        public uint32 get_streak_count (uint step_goal) {
            uint32 streak = 0;
            this.arr.sort ((a, b) => { return b.date.compare (a.date); });

            var last_date = this.arr.get (0).date;
            if (last_date.get_julian () != get_today_date ().get_julian ()) {
                return 0;
            }
            foreach (var steps in this.arr) {
                if (steps.date.days_between (last_date) == streak && steps.steps >= step_goal) {
                    streak++;
                } else {
                    break;
                }
            }
            return streak;
        }

    }

    public class StepsGraphView : Caroline {
        public StepsGraphView (StepsGraphModel model) {
            double[] days;
            double[] steps;
            model.to_arrays (out days, out steps);
            base (days, steps, "smooth-line", true, true);
            /* TRANSLATORS: "Days" is used as the descriptor for the X axis in the steps graph */
            this.dataTypeX = _ ("Days");
            /* TRANSLATORS: "Steps" is used as the descriptor for the Y axis in the steps graph */
            this.dataTypeY = _ ("Steps");
            this.margin = 6;
        }

    }


    [GtkTemplate (ui = "/org/gnome/Health/step_view.ui")]
    public class StepView : View {
        [GtkChild]
        private Gtk.Label streak_label;
        [GtkChild]
        private Gtk.Label title_label;
        [GtkChild]
        private Gtk.Box main_box;
        private StepsGraphView steps_graph_view;
        private StepsGraphModel steps_graph_model;

        public StepView (StepsGraphModel model, Settings settings) {
            this.name = "Steps";
            this.title = _ ("Steps");
            this.title_label.set_text (_ ("Today's steps: %u").printf (model.get_today_step_count ()));
            var streak_count = model.get_streak_count (settings.user_stepgoal);
            switch (streak_count) {
                case 0:
                    this.streak_label.set_text (_ ("No streak yet. Reach your stepgoal for multiple days to start a streak!"));
                    break;
                case 1:
                    this.streak_label.set_text (_ ("You've reached your stepgoal today. Keep going to start a streak!"));
                    break;
                default:
                    this.streak_label.set_text (_ ("You're on a streak for %u days. Good job!").printf (streak_count));
                    break;
            }
            this.steps_graph_view = new StepsGraphView (model);
            this.steps_graph_model = model;
            this.main_box.pack_start (this.steps_graph_view, true, true, 0);
            this.main_box.show_all ();
        }

        public override void update () {
            this.steps_graph_model.reload ();
            this.title_label.set_text (_ ("Today's steps: %u").printf (this.steps_graph_model.get_today_step_count ()));
            this.main_box.remove (this.steps_graph_view);
            this.steps_graph_view = new StepsGraphView (this.steps_graph_model);
            this.main_box.pack_start (this.steps_graph_view, true, true, 0);
            this.main_box.show_all ();
        }

    }
}
