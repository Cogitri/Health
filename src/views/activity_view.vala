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

   /**
    * An implementation of {@link View} visualizes activities the user recently did.
    */
    [GtkTemplate (ui = "/dev/Cogitri/Health/ui/activity_view.ui")]
    public class ActivityView : View {
        [GtkChild]
        private Adw.Clamp clamp;
        [GtkChild]
        private Gtk.ListBox activities_list_box;

        private ActivityModel activity_model;
        private Settings settings;
        private TrackerDatabase db;

        public ActivityView (ActivityModel model, TrackerDatabase db) {
            this.settings = Settings.get_instance ();
            this.activity_model = model;
            this.db = db;
            this.scrolled_window.child = this.clamp;
            this.scrolled_window.vscrollbar_policy = Gtk.PolicyType.AUTOMATIC;

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
