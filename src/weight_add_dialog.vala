/* weight_add_dialog.vala
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
     * A dialog for adding a new weight record.
     */
    [GtkTemplate (ui = "/dev/Cogitri/Health/ui/weight_add_dialog.ui")]
    public class WeightAddDialog : Gtk.Dialog {
        [GtkChild]
        DateSelector date_selector;
        [GtkChild]
        Gtk.SpinButton weight_spin_button;
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

            yield db.save_weight (new Weight (Util.date_from_datetime (this.date_selector.selected_date), new WeightUnitContainer.from_user_value (this.weight_spin_button.value, this.settings)), null);
        }

        private void update_title () {
            db.check_weight_exist_on_date.begin (Util.date_from_datetime (this.date_selector.selected_date), null, (obj, res) => {
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
        private void on_weight_spin_button_changed (Gtk.Editable e) {
            this.set_response_sensitive (Gtk.ResponseType.OK, e.get_text () != "0");
        }
    }
}
