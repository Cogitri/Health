/* weight_graph_model.vala
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
        private TrackerDatabase db;

        public WeightGraphModel (TrackerDatabase db) {
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
                this.arr = yield db.get_weights_after (Util.get_date_in_n_days (-30), null);
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

            foreach (var weight in this.arr) {
                ret.add (new Point (weight.date, weight.weight.value));
            }

            return ret;
        }

        public WeightUnitContainer? get_last_weight () {
            if (this.arr.is_empty) {
                return null;
            }
            var last_weight = this.arr.get (this.arr.size - 1);
            return last_weight.weight;
        }

    }
}
