/* add_dialog.vala
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
    [GtkTemplate (ui = "/dev/Cogitri/Health/add_dialog_activity.ui")]
    public class ActivityAddDialog : Gtk.Dialog {
        [GtkChild]
        DateSelector date_selector;
        [GtkChild]
        private Gtk.ComboBox activity_type_combobox;
        [GtkChild]
        private Gtk.ListStore activity_type_model;
        [GtkChild]
        private Gtk.SpinButton steps_spinner;
        [GtkChild]
        private Gtk.SpinButton minutes_spinner;

        private TrackerDatabase db;

        public ActivityAddDialog (Gtk.Window? parent, TrackerDatabase db) {
            Object (use_header_bar: 1);
            this.set_transient_for (parent);
            this.db = db;

            foreach (var x in Activities.get_values ()) {
                if (x.type == Activities.Enum.WALKING) {
                    Gtk.TreeIter iter;
                    this.activity_type_model.insert_with_values (out iter, -1, 0, x.name, -1);
                    this.activity_type_combobox.set_active_iter (iter);
                } else {
                    Gtk.TreeIter iter;
                    this.activity_type_model.insert_with_values (out iter, -1, 0, x.name, -1);
                }
            }

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

            yield db.save_activity (new Activity (this.get_selected_activity ().type, date_from_datetime (this.date_selector.selected_date), (uint32) this.minutes_spinner.value, steps));
        }

        private Activities.ActivityInfo? get_selected_activity () {
            Gtk.TreeIter iter;

            if (this.activity_type_combobox.get_active_iter (out iter)) {
                GLib.Value val;
                this.activity_type_model.get_value (iter, 0, out val);

                return Activities.get_info_by_name (val.get_string ());
            }

            return null;
        }

        private void check_response_active () {
            var selected_activity = this.get_selected_activity ();

            if (selected_activity != null && selected_activity.has_steps) {
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
        private void on_activity_type_combobox_changed (Gtk.ComboBox cb) {
            var selected_activity = this.get_selected_activity ();

            if (selected_activity != null && (!) selected_activity.has_steps) {
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

    /**
     * A dialog for adding a new weight record.
     */
    [GtkTemplate (ui = "/dev/Cogitri/Health/add_dialog_weight.ui")]
    public class WeightAddDialog : Gtk.Dialog {
        [GtkChild]
        DateSelector date_selector;
        [GtkChild]
        Gtk.SpinButton weight_spinner;
        private Settings settings;
        private TrackerDatabase db;

        public WeightAddDialog (Gtk.Window? parent, Settings settings, TrackerDatabase db) {
            Object (use_header_bar: 1);
            this.set_transient_for (parent);
            this.db = db;
            this.settings = settings;

            this.update_title ();
            this.date_selector.notify["selected_date"].connect (() => {
                this.update_title ();
            });
            this.set_response_sensitive (Gtk.ResponseType.OK, false);
        }

        /**
         * Saves the data that has been entered into the dialog to the database.
         */
        public async void save () throws GLib.Error {
            var db = TrackerDatabase.get_instance ();

            yield db.save_weight (new Weight (date_from_datetime (this.date_selector.selected_date), new WeightUnitContainer.from_user_value (this.weight_spinner.value, this.settings)), null);
        }

        private void update_title () {
            db.check_weight_exist_on_date.begin (date_from_datetime (this.date_selector.selected_date), null, (obj, res) => {
                var update = false;
                try {
                    update = db.check_weight_exist_on_date.end (res);
                } catch (GLib.Error e) {
                    warning (e.message);
                }

                if (update) {
                    this.title = _ ("Update Weight Record");
                } else {
                    this.title = _ ("Add New weight Record");
                }
            });
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
        private void on_weight_spinner_changed (Gtk.Editable e) {
            this.set_response_sensitive (Gtk.ResponseType.OK, e.get_text () != "0");
        }
    }
}
