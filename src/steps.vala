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
    /**
     * A Step record for a single day.
     */
    public class Steps : GLib.Object {
        public GLib.Date date { get; private set; }
        public uint32 steps { get; set; }

        public Steps (GLib.Date date, uint32 steps) {
            this.date = date;
            this.steps = steps;
        }

    }

    /**
     * An implementation of {@link GraphModel} that interacts with the user's step record data.
     */
    public class StepsGraphModel : GraphModel<Steps> {
        private TrackerDatabase db;

        public StepsGraphModel (TrackerDatabase db) {
            this.db = db;

            this.init ();
        }

        /**
         * Reload the data from the DB
         *
         * This can be used e.g. after the user added a new step record.
         * @return true if reloading suceeded.
         */
        public async override bool reload () {
            try {
                this.arr = yield db.get_steps_after (get_date_in_n_days (-30), null);
                return true;
            } catch (GLib.Error e) {
                warning ("Failed to load steps from database due to error %s", e.message);
                return false;
            }
        }

        /**
         * {@inheritDoc}
         */
        public override Gee.ArrayList<Point> to_points () {
            var ret = new Gee.ArrayList<Point> ();

            this.arr.sort ((a, b) => { return a.date.compare (b.date); });

            var first_date = this.arr.get (0).date;
            var last_date = get_today_date ();
            var date_delta = first_date.days_between (last_date);
            var target_date = GLib.Date ();
            for (int i = 0; i <= date_delta; i++) {
                target_date.set_julian (first_date.get_julian () + i);
                Steps? item;
                item = this.arr.first_match ((s) => { return s.date.get_julian () == target_date.get_julian (); });
                if (item == null) {
                    ret.add (new Point (target_date, 0));
                } else {
                    ret.add (new Point (((!) item).date, ((!) item).steps));
                }
            }

            return ret;
        }

        public uint32 get_today_step_count () {
            var today = get_today_date ();
            Steps? steps;
            steps = this.arr.first_match ((s) => {
                return s.date.get_julian () == today.get_julian ();
            });
            return steps != null ? ((!) steps).steps : 0;
        }

        private uint32 get_streak_count_on_day (uint step_goal, GLib.Date date) {
            uint32 streak = 0;

            if (this.arr.is_empty) {
                return 0;
            }

            this.arr.sort ((a, b) => { return b.date.compare (a.date); });

            var last_date = this.arr.get (0).date;
            if (last_date.get_julian () != date.get_julian ()) {
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

        /**
         * Gets streak count, including today.
         *
         * If no steps have been recorded today, then this will return 0.
         */
        public uint32 get_streak_count_today (uint step_goal) {
            return this.get_streak_count_on_day (step_goal, get_today_date ());
        }


        /**
         * Gets streak count, excluding today.
         *
         * If no steps have been recorded today, then this can still return >0
         * if the user had a streak yesterday.
         */
        public uint32 get_streak_count_yesterday (uint step_goal) {
            var date = get_today_date ();
            date.subtract_days (1);
            return this.get_streak_count_on_day (step_goal, date);
        }

    }

    /**
     * An implementation of {@link GraphView} that visualizes the user's step records over time.
     */
    public class StepsGraphView : GraphView {
        public StepsGraphView (StepsGraphModel model, double stepgoal) {
            base (model.to_points (), _ ("Stepgoal"), stepgoal);
            this.x_lines_interval = 500;
        }

    }


    /**
     * An implementation of {@link View} visualizes streak counts and daily step records.
     */
    [GtkTemplate (ui = "/dev/Cogitri/Health/step_view.ui")]
    public class StepView : View {
        [GtkChild]
        private Gtk.Box main_box;
        [GtkChild]
        private Gtk.Label streak_label;
        [GtkChild]
        private Gtk.Label title_label;
        private Gtk.Label no_data_label;
        private Gtk.ScrolledWindow scrolled_window;
        private Settings settings;
        private StepsGraphView? steps_graph_view;
        private StepsGraphModel steps_graph_model;

        public StepView (StepsGraphModel model, Settings settings, TrackerDatabase db) {
            this.name = "Steps";
            this.title = _ ("Steps");
            this.icon_name = "dev.Cogitri.Health-steps-symbolic";
            this.settings = settings;
            this.steps_graph_model = model;

            if (this.steps_graph_model.is_empty) {
                this.no_data_label = new Gtk.Label (_ ("No data has been added yet. Click + to add a new step count."));
                this.no_data_label.wrap = true;
                this.no_data_label.wrap_mode = Pango.WrapMode.WORD_CHAR;
                this.no_data_label.margin_start = this.no_data_label.margin_end = 6;
                this.main_box.append (this.no_data_label);
            } else {
                this.scrolled_window = new Gtk.ScrolledWindow ();
                this.scrolled_window.vscrollbar_policy = Gtk.PolicyType.NEVER;
                this.scrolled_window.child = this.steps_graph_view = new StepsGraphView (model, this.settings.user_stepgoal);
                this.main_box.append (this.scrolled_window);
            }

            this.settings.changed[Settings.USER_STEPGOAL_KEY].connect (() => {
                this.update ();
            });
            db.steps_updated.connect (() => {
                this.update ();
            });
            this.update ();
        }

        ~StepView () {
            unowned Gtk.Widget child;
            while ((child = get_first_child ()) != null) {
                child.unparent ();
            }
        }

        /**
         * Reload the {@link StepsGraphModel}'s data and refresh labels & the {@link StepsGraphView}.
         */
        public override void update () {
            this.steps_graph_model.reload.begin ((obj, res) => {
                if (this.steps_graph_model.reload.end (res)) {
                    this.title_label.set_text (_ ("Today's steps: %u").printf (this.steps_graph_model.get_today_step_count ()));
                    var streak_count = this.steps_graph_model.get_streak_count_today (this.settings.user_stepgoal);
                    switch (streak_count) {
                        case 0:
                            var previous_streak = this.steps_graph_model.get_streak_count_yesterday (this.settings.user_stepgoal);
                            if (previous_streak == 0) {
                                this.streak_label.set_text (_ ("No streak yet. Reach your stepgoal for multiple days to start a streak!"));
                            } else {
                                this.streak_label.set_text (_ ("You're on a streak for %u days. Reach your stepgoal today to continue it!").printf (previous_streak));
                            }
                            break;
                        case 1:
                            this.streak_label.set_text (_ ("You've reached your stepgoal today. Keep going to start a streak!"));
                            break;
                        default:
                            this.streak_label.set_text (_ ("You're on a streak for %u days. Good job!").printf (streak_count));
                            break;
                    }
                    this.title_label.set_text (_ ("Today's steps: %u").printf (this.steps_graph_model.get_today_step_count ()));

                    if (this.steps_graph_view == null && !this.steps_graph_model.is_empty) {
                        this.scrolled_window = new Gtk.ScrolledWindow ();
                        this.scrolled_window.vscrollbar_policy = Gtk.PolicyType.NEVER;
                        this.scrolled_window.child = this.steps_graph_view = new StepsGraphView (this.steps_graph_model, this.settings.user_stepgoal);
                        this.main_box.remove (this.no_data_label);
                        this.main_box.append (this.scrolled_window);
                        this.no_data_label = null;
                        ((!) this.steps_graph_view).visible = true;
                    } else if (this.steps_graph_view != null) {
                        ((!) this.steps_graph_view).points = this.steps_graph_model.to_points ();
                        ((!) this.steps_graph_view).limit = this.settings.user_stepgoal;
                    }
                }
            });
        }

    }
}
