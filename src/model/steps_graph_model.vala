/* steps_graph_model.vala
 *
 * Copyright 2021 Rasmus Thomsen <oss@cogitri.dev>
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
                this.arr = yield db.get_steps_after (Util.get_date_in_n_days (-30), null);
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

            var first_date = this.arr.get (0).date;
            var last_date = Util.get_today_date ();
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
            var today = Util.get_today_date ();
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
            return this.get_streak_count_on_day (step_goal, Util.get_today_date ());
        }


        /**
         * Gets streak count, excluding today.
         *
         * If no steps have been recorded today, then this can still return >0
         * if the user had a streak yesterday.
         */
        public uint32 get_streak_count_yesterday (uint step_goal) {
            var date = Util.get_today_date ();
            date.subtract_days (1);
            return this.get_streak_count_on_day (step_goal, date);
        }

    }
}
