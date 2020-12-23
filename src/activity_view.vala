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
    public class ActivityModel : GLib.Object {
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

        /**
         * Reload the data from the DB
         *
         * This can be used e.g. after the user added a new activity record.
         * @return true if reloading suceeded.
         */
        public async bool reload () {
            try {
                this.activities = yield this.db.get_activities_after (get_date_in_n_days (-30), this.settings);
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
        private Gtk.Box main_box;
        [GtkChild]
        private Gtk.ListBox activities_list_box;
        [GtkChild]
        private Gtk.ScrolledWindow scrolled_window;
        private Gtk.Label no_data_label;
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

            if (this.activity_model.is_empty) {
                this.no_data_label = new Gtk.Label (_ ("No data has been added yet. Click + to add a new activity record."));
                this.no_data_label.wrap = true;
                this.no_data_label.wrap_mode = Pango.WrapMode.WORD_CHAR;
                this.no_data_label.margin_start = this.no_data_label.margin_end = 6;
                this.main_box.append (this.no_data_label);
            } else {
                this.main_box.append (this.scrolled_window);
           }

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

                    if (!this.activity_model.is_empty && this.main_box.get_last_child () == this.no_data_label) {
                        this.main_box.remove (this.no_data_label);
                        this.main_box.append (this.scrolled_window);
                        this.no_data_label = null;
                    } else if (!this.activity_model.is_empty) {
                        // FIXME: Allow adding adjusting this & loading more activities on demand
                        this.db.get_activities_after.begin (get_date_in_n_days (-30), this.settings, null, (obj, res) => {
                            try {
                                var activities = this.db.get_activities_after.end (res);
                                unowned Gtk.Widget? w;

                                while ((w = this.activities_list_box.get_first_child ()) != null) {
                                    this.activities_list_box.remove (w);
                                }

                                foreach (var activity in activities) {
                                    this.activities_list_box.append ((ActivityRow) GLib.Object.new (typeof (ActivityRow), activity: activity));
                                }
                            } catch (GLib.Error e) {
                                warning ("Failed to retrieve activities from DB due to error %s", e.message);
                            }
                        });
                    }
                }
            });
        }

    }
}
