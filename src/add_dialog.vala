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
     * AddDialog is a generic dialog used for adding new data to the DB via user input.
     */
    [GtkTemplate (ui = "/dev/Cogitri/Health/add_dialog.ui")]
    public class AddDialog : Gtk.Dialog {
        [GtkChild]
        protected Gtk.Label dialog_label;
        [GtkChild]
        protected Gtk.Entry dialog_entry;

        public AddDialog (Gtk.Window? parent) {
            Object (use_header_bar: 1);
            this.set_transient_for (parent);
        }

        [GtkCallback]
        private void dialog_entry_changed (Gtk.Editable editable) {
            this.set_response_sensitive (Gtk.ResponseType.OK, editable.text.length != 0);
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

        /**
         * save() should save the user's input to the DB.
         */
        public async virtual void save () throws GLib.Error {
        }

    }

    /**
     * An {@link AddDialog} for adding a new step record.
     */
    public class StepsAddDialog : AddDialog {
        private TrackerDatabase db;

        public StepsAddDialog (Gtk.Window? parent, TrackerDatabase db) {
            base (parent);

            this.db = db;


            db.check_steps_exist_on_date.begin (get_today_date (), null, (obj, res) => {
                var update = false;
                try {
                    update = db.check_steps_exist_on_date.end (res);
                } catch (GLib.Error e) {
                    warning (e.message);
                }


                if (update) {
                    this.dialog_label.set_text (_ ("Update today's step record"));
                } else {
                    this.dialog_label.set_text (_ ("Add new step record"));
                }
            });

            this.dialog_entry.set_max_length (6);
        }

        /**
         * Saves the data that has been entered into the dialog to the database.
         */
        public async override void save () throws GLib.Error {
            var db = TrackerDatabase.get_instance ();

            uint64 steps = 0;
            try {
                uint64.from_string (this.dialog_entry.get_text (), out steps);
            } catch (NumberParserError e) {
                warning (_("Failed to parse steps due to error %s"), e.message);
            }

            yield db.save_steps (new Steps (get_today_date (), (uint32) steps), null);
        }

    }

    /**
     * An {@link AddDialog} for adding a new weight record.
     */
    public class WeightAddDialog : AddDialog {
        private Settings settings;
        private TrackerDatabase db;

        public WeightAddDialog (Gtk.Window? parent, Settings settings, TrackerDatabase db) {
            base (parent);

            this.db = db;
            this.settings = settings;

            db.check_weight_exist_on_date.begin (get_today_date (), null, (obj, res) => {
                var update = false;
                try {
                    update = db.check_weight_exist_on_date.end (res);
                } catch (GLib.Error e) {
                    warning (e.message);
                }

                if (update) {
                    this.dialog_label.set_text (_ ("Update today's weight measurement"));
                } else {
                    this.dialog_label.set_text (_ ("Add new weight measurement"));
                }
            });

            this.dialog_entry.set_max_length (6);
        }

        /**
         * Saves the data that has been entered into the dialog to the database.
         */
        public async override void save () throws GLib.Error {
            var db = TrackerDatabase.get_instance ();

            double weight = 0;
            if (!double.try_parse (this.dialog_entry.get_text (), out weight)) {
                warning (_ ("Failed to parse weight '%s' as floating point number"), this.dialog_entry.get_text ());
            }

            yield db.save_weight (new Weight (get_today_date (), new WeightUnitContainer.from_user_value (weight, this.settings)), null);
        }

    }
}
