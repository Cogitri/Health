/* activity_view.vala
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
    public class ActivityModel : GLib.Object, GLib.ListModel {
        private Gee.ArrayList<Activity> activities;
        private Settings settings;
        private TrackerDatabase db;
        public bool is_empty {
            get {
                    return this.activities.is_empty;
            }
        }

        public ActivityModel (Settings settings, TrackerDatabase db) {
            this.settings = settings;
            this.db = db;
            this.activities = new Gee.ArrayList<Activity> ();
        }

        public GLib.Object? get_item (uint position) {
                if (this.activities.size > position) {
                return this.activities.get ((int) position);
            } else {
                return null;
            }
        }

        public GLib.Type get_item_type () {
            return typeof (Activity);
        }

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
                this.activities = yield this.db.get_activities_after (get_date_in_n_days (-30), this.settings);
                this.items_changed (0, previous_size, this.activities.size);
                return true;
            } catch (GLib.Error e) {
                warning ("Failed to load activities from database due to error %s", e.message);
                return false;
            }
        }
    }

   /**
    * An implementation of {@link View} visualizes activities the user recently did.
    */
    [GtkTemplate (ui = "/dev/Cogitri/Health/ui/activity_view.ui")]
    public class ActivityView : View {
        [GtkChild]
        private Gtk.ListBox activities_list_box;
        [GtkChild]
        private Gtk.Stack stack;
        private Settings settings;
        private ActivityModel activity_model;
        TrackerDatabase db;

        public ActivityView (ActivityModel model, Settings settings, TrackerDatabase db) {
            this.name = "Activities";
            this.title = _ ("Activities");
            this.icon_name = "walking-symbolic";
            this.settings = settings;
            this.activity_model = model;
            this.db = db;

            if (!this.activity_model.is_empty) {
                this.stack.visible_child_name = "data_page";
            }

           this.activities_list_box.bind_model (this.activity_model, (o) => {
                return (Gtk.Widget) GLib.Object.new (typeof (ActivityRow), activity: o);
            });

            db.activities_updated.connect (() => {
                this.update ();
            });
            this.update ();
        }

        ~ActivityView () {
            unowned Gtk.Widget child;
            while ((child = get_first_child ()) != null) {
                child.unparent ();
            }
        }

        /**
         * Reload the {@link ActivityModel}'s data and refresh the list of activities
         */
        public override void update () {
            this.activity_model.reload.begin ((obj, res) => {
                if (this.activity_model.reload.end (res)) {
                    if (!this.activity_model.is_empty) {
                        this.stack.visible_child_name = "data_page";
                    }
                }
            });
        }

    }
}
