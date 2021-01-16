/* activity_model.vala
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
     * An implementation of {@link GLib.ListModel} that stores {@link Activity}s. Can be used with
     * {@link ActivityView} to display past activities.
     */
     public class ActivityModel : GLib.Object, GLib.ListModel {
        private Gee.ArrayList<Activity> activities;
        private Settings settings;
        private TrackerDatabase db;

        public bool is_empty {
            get {
                    return this.activities.is_empty;
            }
        }

        public ActivityModel (TrackerDatabase db) {
            this.settings = Settings.get_instance ();
            this.db = db;
            this.activities = new Gee.ArrayList<Activity> ();
        }

        /**
         * {@inheritDoc}
         */
        public GLib.Object? get_item (uint position) {
                if (this.activities.size > position) {
                return this.activities.get ((int) position);
            } else {
                return null;
            }
        }

        /**
         * {@inheritDoc}
         */
        public GLib.Type get_item_type () {
            return typeof (Activity);
        }

        /**
         * {@inheritDoc}
         */
        public uint get_n_items () {
            return this.activities.size;
        }

        /**
         * Reload the data from the DB
         *
         * This can be used e.g. after the user added a new activity record.
         * @return true if reloading suceeded.
         */
        public async bool reload () {
            try {
                var previous_size = this.activities.size;
                this.activities = yield this.db.get_activities_after (Util.get_date_in_n_days (-30));
                this.items_changed (0, previous_size, this.activities.size);
                return true;
            } catch (GLib.Error e) {
                warning ("Failed to load activities from database due to error %s", e.message);
                return false;
            }
        }
    }
}