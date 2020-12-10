/* activity_add_dialog.vala
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
     * A dialog for adding a new activity record.
     */
    [GtkTemplate (ui = "/dev/Cogitri/Health/activity_add_dialog.ui")]
    public class ActivityAddDialog : Gtk.Dialog {
        [GtkChild]
        DateSelector date_selector;
        [GtkChild]
        private Gtk.StringList activity_type_model;
        [GtkChild]
        private Gtk.SpinButton steps_spinner;
        [GtkChild]
        private Gtk.SpinButton minutes_spinner;
        [GtkChild]
        private Hdy.ComboRow activity_type_comborow;

        private TrackerDatabase db;

        public ActivityAddDialog (Gtk.Window? parent, TrackerDatabase db) {
            Object (use_header_bar: 1);
            this.set_transient_for (parent);
            this.db = db;

            foreach (var x in Activities.get_values ()) {
                    this.activity_type_model.append (x.name);
            }
            this.activity_type_comborow.model = this.activity_type_model;
            this.activity_type_comborow.selected = Activities.Enum.WALKING;
            this.set_response_sensitive (Gtk.ResponseType.OK, false);
        }

        /**
         * Saves the data that has been entered into the dialog to the database.
         */
        public async void save () throws GLib.Error {
            var db = TrackerDatabase.get_instance ();

            uint32? steps = null;

            if (this.steps_spinner.text != "") {
                steps = (uint32) this.steps_spinner.value;
            }

            yield db.save_activity (new Activity (this.get_selected_activity ().type, date_from_datetime (this.date_selector.selected_date), 0, 0, 0, 0, 0, (uint32) this.minutes_spinner.value, steps));
        }

        private Activities.ActivityInfo get_selected_activity () {
            return Activities.get_values ()[this.activity_type_comborow.selected];
        }

        private void check_response_active () {
            if (ActivityDataPoints.STEP_COUNT in this.get_selected_activity ().available_data_points) {
                this.set_response_sensitive (Gtk.ResponseType.OK, steps_spinner.get_text () != "0" && minutes_spinner.get_text () != "0");
            } else {
                this.set_response_sensitive (Gtk.ResponseType.OK, minutes_spinner.get_text () != "0");
            }
        }

        [GtkCallback]
        private void on_response (int response_id) {
            switch (response_id) {
                case Gtk.ResponseType.OK:
                    this.save.begin ((obj, res) => {
                        try {
                            this.save.end (res);
                        } catch (GLib.Error e) {
                            warning (_ ("Failed to save new data due to error %s"), e.message);
                        }
                    });
                    break;
            }
            this.destroy ();
        }


        [GtkCallback]
        private void on_activity_type_comborow_selected (GLib.Object o, GLib.ParamSpec p) {
            if (ActivityDataPoints.STEP_COUNT in this.get_selected_activity ().available_data_points) {
                    this.steps_spinner.sensitive = true;
            } else {
                this.steps_spinner.sensitive = false;
            }

            this.check_response_active ();
        }

        [GtkCallback]
        private void on_spinner_changed (Gtk.Editable e) {
            this.check_response_active ();
        }
    }
}
